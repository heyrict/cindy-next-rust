use async_graphql::{self, Context, InputObject, Object};
use diesel::prelude::*;

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::puzzle_tag::*;
use crate::models::*;
use crate::schema::puzzle_tag;

use super::tag::{CreateTagInput, TagMutation};

#[derive(Default)]
pub struct PuzzleTagQuery;
#[derive(Default)]
pub struct PuzzleTagMutation;

#[Object]
impl PuzzleTagQuery {
    pub async fn puzzle_tag(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<PuzzleTag> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let puzzle_tag = puzzle_tag::table
            .filter(puzzle_tag::id.eq(id))
            .limit(1)
            .first(&conn)?;

        Ok(puzzle_tag)
    }

    pub async fn puzzle_tags(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<PuzzleTagFilter>>,
        order: Option<Vec<PuzzleTagOrder>>,
    ) -> async_graphql::Result<Vec<PuzzleTag>> {
        use crate::schema::puzzle_tag::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = puzzle_tag.into_boxed();
        if let Some(order) = order {
            query = PuzzleTagOrders::new(order).apply_order(query);
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

        let puzzle_tags = query.load::<PuzzleTag>(&conn)?;

        Ok(puzzle_tags)
    }

    pub async fn puzzle_tag_count(
        &self,
        ctx: &Context<'_>,
        filter: Option<Vec<PuzzleTagFilter>>,
    ) -> async_graphql::Result<i64> {
        use crate::schema::puzzle_tag::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = puzzle_tag.into_boxed();
        if let Some(filter) = filter {
            if let Some(filter_exp) = filter.as_expression() {
                query = query.filter(filter_exp)
            }
        }

        let result = query.count().get_result(&conn)?;

        Ok(result)
    }
}

#[derive(InputObject, AsChangeset, Debug)]
#[table_name = "puzzle_tag"]
pub struct UpdatePuzzleTagInput {
    pub id: Option<ID>,
    pub puzzle_id: Option<ID>,
    pub tag_id: Option<ID>,
    pub user_id: Option<ID>,
}

#[derive(InputObject, Insertable)]
#[table_name = "puzzle_tag"]
pub struct CreatePuzzleTagInput {
    pub id: Option<ID>,
    pub puzzle_id: ID,
    pub tag_id: ID,
    pub user_id: Option<ID>,
}

#[derive(InputObject)]
pub struct CreatePuzzleTagWithTagInput {
    pub id: Option<ID>,
    pub puzzle_id: ID,
    pub tag: CreateTagInput,
    pub user_id: Option<ID>,
}

#[Object]
impl PuzzleTagMutation {
    // Update puzzle_tag
    pub async fn update_puzzle_tag(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdatePuzzleTagInput,
    ) -> async_graphql::Result<PuzzleTag> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // User should be the owner on update mutation
                let puzzle_tag_inst: PuzzleTag = puzzle_tag::table
                    .filter(puzzle_tag::id.eq(id))
                    .limit(1)
                    .first(&conn)?;
                user_id_guard(ctx, puzzle_tag_inst.user_id)?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let puzzle_tag: PuzzleTag = diesel::update(puzzle_tag::table)
            .filter(puzzle_tag::id.eq(id))
            .set(set)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(puzzle_tag)
    }

    // Create puzzle_tag
    pub async fn create_puzzle_tag(
        &self,
        ctx: &Context<'_>,
        mut data: CreatePuzzleTagInput,
    ) -> async_graphql::Result<PuzzleTag> {
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

        let puzzle_tag: PuzzleTag = diesel::insert_into(puzzle_tag::table)
            .values(&data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(puzzle_tag)
    }

    // A sequential mutation that creates tag first, then create a puzzle_tag
    // with tag_id associated to that tag.
    pub async fn create_puzzle_tag_with_tag(
        &self,
        ctx: &Context<'_>,
        mut data: CreatePuzzleTagWithTagInput,
    ) -> async_graphql::Result<PuzzleTag> {
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

        let tag: Tag = TagMutation.create_tag(ctx, data.tag).await?;
        let data = CreatePuzzleTagInput {
            id: data.id,
            puzzle_id: data.puzzle_id,
            user_id: data.user_id,
            tag_id: tag.id,
        };

        let puzzle_tag: PuzzleTag = diesel::insert_into(puzzle_tag::table)
            .values(&data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(puzzle_tag)
    }

    // Delete puzzle_tag
    #[graphql(guard = "DenyRoleGuard::new(Role::Guest)")]
    pub async fn delete_puzzle_tag(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<PuzzleTag> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                use crate::schema::puzzle;
                let user_id = reqctx
                    .get_user_id()
                    .ok_or(async_graphql::Error::new("No user"))?;
                // User should be the owner of the puzzle or the puzzle_tag
                let puzzle_tag_inst: PuzzleTag = puzzle_tag::table
                    .filter(puzzle_tag::id.eq(id))
                    .limit(1)
                    .first(&conn)?;
                let puzzle_inst: Puzzle = puzzle::table
                    .filter(puzzle::id.eq(puzzle_tag_inst.puzzle_id))
                    .limit(1)
                    .first(&conn)?;
                assert_eq_guard(puzzle_tag_inst.user_id, user_id)
                    .or_else(|_| assert_eq_guard(puzzle_inst.user_id, user_id))?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let puzzle_tag = diesel::delete(puzzle_tag::table.filter(puzzle_tag::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(puzzle_tag)
    }
}
