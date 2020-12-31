use async_graphql::{self, guard::Guard, Context, InputObject, Object};
use diesel::prelude::*;

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::comment::*;
use crate::models::*;
use crate::schema::comment;

#[derive(Default)]
pub struct CommentQuery;
#[derive(Default)]
pub struct CommentMutation;

#[Object]
impl CommentQuery {
    pub async fn comment(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Comment> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let comment = comment::table
            .filter(comment::id.eq(id))
            .limit(1)
            .first(&conn)?;

        Ok(comment)
    }

    pub async fn comments(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<CommentFilter>>,
        order: Option<Vec<CommentOrder>>,
    ) -> async_graphql::Result<Vec<Comment>> {
        use crate::schema::comment::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = comment.into_boxed();
        if let Some(order) = order {
            query = CommentOrders::new(order).apply_order(query);
        }
        if let Some(filter) = filter {
            if let Some(filter_exp) = filter.as_expression() {
                query = query.filter(filter_exp)
            }
        }
        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let comments = query.load::<Comment>(&conn)?;

        Ok(comments)
    }

    pub async fn comments_in_solved_puzzle(
        &self,
        ctx: &Context<'_>,
        limit: i64,
        offset: i64,
    ) -> async_graphql::Result<Vec<Comment>> {
        use crate::schema::{comment, puzzle};

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let comments = comment::table
            .inner_join(puzzle::table)
            .filter(puzzle::status.ne(Status::Undergoing))
            .limit(limit)
            .offset(offset)
            .select(comment::all_columns)
            .get_results::<Comment>(&conn)?;

        Ok(comments)
    }

    pub async fn comment_count(
        &self,
        ctx: &Context<'_>,
        filter: Option<CommentCountFilter>,
    ) -> async_graphql::Result<i64> {
        use crate::schema::comment::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = comment.into_boxed();
        if let Some(filter) = filter {
            if let Some(filter_exp) = filter.as_expression() {
                query = query.filter(filter_exp)
            }
        }

        let result = query.count().get_result::<i64>(&conn)?;

        Ok(result)
    }

    pub async fn user_received_comments(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        limit: i64,
        offset: i64,
    ) -> async_graphql::Result<Vec<Comment>> {
        use crate::schema::{comment, puzzle};

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let comments: Vec<Comment> = comment::table
            .inner_join(puzzle::table)
            .filter(puzzle::user_id.eq(user_id))
            .limit(limit)
            .offset(offset)
            .select(comment::all_columns)
            .get_results::<Comment>(&conn)?;

        Ok(comments)
    }

    pub async fn user_received_comment_count(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
    ) -> async_graphql::Result<i64> {
        use crate::schema::{comment, puzzle};

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = comment::table
            .inner_join(puzzle::table)
            .filter(puzzle::user_id.eq(user_id))
            .count()
            .get_result(&conn)?;

        Ok(result)
    }
}

#[derive(InputObject, AsChangeset, Debug)]
#[table_name = "comment"]
pub struct UpdateCommentInput {
    pub id: Option<ID>,
    pub content: Option<String>,
    pub spoiler: Option<bool>,
    pub puzzle_id: Option<ID>,
    pub user_id: Option<ID>,
}

#[derive(InputObject, Insertable)]
#[table_name = "comment"]
pub struct CreateCommentInput {
    pub id: Option<ID>,
    pub content: String,
    #[graphql(default = false)]
    pub spoiler: bool,
    pub puzzle_id: ID,
    pub user_id: Option<ID>,
}

#[Object]
impl CommentMutation {
    // Update comment
    #[graphql(guard(DenyRoleGuard(role = "Role::Guest")))]
    pub async fn update_comment(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateCommentInput,
    ) -> async_graphql::Result<Comment> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // User should be the owner on update mutation
                let comment_inst: Comment = comment::table
                    .filter(comment::id.eq(id))
                    .limit(1)
                    .first(&conn)?;
                user_id_guard(ctx, comment_inst.user_id)?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let comment: Comment = diesel::update(comment::table)
            .filter(comment::id.eq(id))
            .set(set)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(comment)
    }

    // Create comment
    #[graphql(guard(DenyRoleGuard(role = "Role::Guest")))]
    pub async fn create_comment(
        &self,
        ctx: &Context<'_>,
        mut data: CreateCommentInput,
    ) -> async_graphql::Result<Comment> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // Assert the user is the owner of the puzzle.
                if let Some(user_id) = data.user_id {
                    user_id_guard(ctx, user_id)?;
                } else {
                    data.user_id = reqctx.get_user_id();
                };
            }
            Role::Admin => {}
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        let comment: Comment = diesel::insert_into(comment::table)
            .values(&data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(comment)
    }

    // Delete comment (admin only)
    #[graphql(guard(and(
        DenyRoleGuard(role = "Role::User"),
        DenyRoleGuard(role = "Role::Guest")
    )))]
    pub async fn delete_comment(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Comment> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let comment = diesel::delete(comment::table.filter(comment::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(comment)
    }
}
