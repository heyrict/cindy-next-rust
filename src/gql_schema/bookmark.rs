use async_graphql::{self, Context, InputObject, Object};
use diesel::prelude::*;

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::bookmark::*;
use crate::models::*;
use crate::schema::bookmark;

#[derive(Default)]
pub struct BookmarkQuery;
#[derive(Default)]
pub struct BookmarkMutation;

#[Object]
impl BookmarkQuery {
    pub async fn bookmark(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Bookmark> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let bookmark = bookmark::table
            .filter(bookmark::id.eq(id))
            .limit(1)
            .first(&conn)?;

        Ok(bookmark)
    }

    pub async fn bookmarks(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<BookmarkFilter>>,
        order: Option<Vec<BookmarkOrder>>,
    ) -> async_graphql::Result<Vec<Bookmark>> {
        use crate::schema::bookmark::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = bookmark.into_boxed();
        if let Some(order) = order {
            query = BookmarkOrders::new(order).apply_order(query);
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

        let bookmarks = query.load::<Bookmark>(&conn)?;

        Ok(bookmarks)
    }

    pub async fn bookmark_count(
        &self,
        ctx: &Context<'_>,
        filter: Option<BookmarkCountFilter>,
    ) -> async_graphql::Result<i64> {
        use crate::schema::bookmark::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = bookmark.into_boxed();
        if let Some(filter) = filter {
            if let Some(filter_exp) = filter.as_expression() {
                query = query.filter(filter_exp)
            }
        }

        let result = query.count().get_result::<i64>(&conn)?;

        Ok(result)
    }
}

#[derive(InputObject, AsChangeset, Debug)]
#[table_name = "bookmark"]
pub struct UpdateBookmarkInput {
    pub id: Option<ID>,
    pub value: Option<i16>,
    pub puzzle_id: Option<ID>,
    pub user_id: Option<ID>,
}

#[derive(InputObject, Insertable)]
#[table_name = "bookmark"]
pub struct CreateBookmarkInput {
    pub id: Option<ID>,
    pub value: i16,
    pub puzzle_id: ID,
    pub user_id: Option<ID>,
}

#[Object]
impl BookmarkMutation {
    pub async fn update_bookmark(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateBookmarkInput,
    ) -> async_graphql::Result<Bookmark> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // User should be the owner on update mutation
                let bookmark_inst: Bookmark = bookmark::table
                    .filter(bookmark::id.eq(id))
                    .limit(1)
                    .first(&conn)?;
                user_id_guard(ctx, bookmark_inst.user_id)?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let bookmark: Bookmark = diesel::update(bookmark::table)
            .filter(bookmark::id.eq(id))
            .set(set)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(bookmark)
    }

    pub async fn create_bookmark(
        &self,
        ctx: &Context<'_>,
        mut data: CreateBookmarkInput,
    ) -> async_graphql::Result<Bookmark> {
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
            Role::Staff => {}
            Role::Admin => {}
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        let bookmark: Bookmark = diesel::insert_into(bookmark::table)
            .values(&data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(bookmark)
    }

    // Delete bookmark
    #[graphql(guard = "DenyRoleGuard::new(Role::Guest)")]
    pub async fn delete_bookmark(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Bookmark> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // User should be the owner
                let bookmark_inst: Bookmark = bookmark::table
                    .filter(bookmark::id.eq(id))
                    .limit(1)
                    .first(&conn)?;
                user_id_guard(ctx, bookmark_inst.user_id)?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let bookmark = diesel::delete(bookmark::table.filter(bookmark::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(bookmark)
    }
}
