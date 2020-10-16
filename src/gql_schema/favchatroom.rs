use async_graphql::{self, Context, InputObject, Object};
use diesel::prelude::*;

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::favchatroom::*;
use crate::models::*;
use crate::schema::favorite_chatroom;

#[derive(Default)]
pub struct FavChatroomQuery;
#[derive(Default)]
pub struct FavChatroomMutation;

#[Object]
impl FavChatroomQuery {
    pub async fn favchatroom(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> async_graphql::Result<FavChatroom> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let favchatroom = favorite_chatroom::table
            .filter(favorite_chatroom::id.eq(id))
            .limit(1)
            .first(&conn)?;

        Ok(favchatroom)
    }

    pub async fn favchatrooms(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<FavChatroomFilter>>,
        order: Option<Vec<FavChatroomOrder>>,
    ) -> async_graphql::Result<Vec<FavChatroom>> {
        use crate::schema::favorite_chatroom::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = favorite_chatroom.into_boxed();
        if let Some(order) = order {
            query = FavChatroomOrders::new(order).apply_order(query);
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

        let favchatrooms = query.load::<FavChatroom>(&conn)?;

        Ok(favchatrooms)
    }
}

#[derive(InputObject, AsChangeset, Debug)]
#[table_name = "favorite_chatroom"]
pub struct UpdateFavChatroomInput {
    pub id: Option<ID>,
    pub user_id: Option<ID>,
    pub chatroom_id: Option<ID>,
}

#[derive(InputObject, Insertable)]
#[table_name = "favorite_chatroom"]
pub struct CreateFavChatroomInput {
    pub id: Option<ID>,
    pub user_id: Option<ID>,
    pub chatroom_id: Option<ID>,
}

#[Object]
impl FavChatroomMutation {
    pub async fn update_favchatroom(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateFavChatroomInput,
    ) -> async_graphql::Result<FavChatroom> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // User should be the owner on update mutation
                let favchatroom_inst: FavChatroom = favorite_chatroom::table
                    .filter(favorite_chatroom::id.eq(id))
                    .limit(1)
                    .first(&conn)?;
                user_id_guard(ctx, favchatroom_inst.user_id)?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let favchatroom: FavChatroom = diesel::update(favorite_chatroom::table)
            .filter(favorite_chatroom::id.eq(id))
            .set(set)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(favchatroom)
    }

    pub async fn create_favchatroom(
        &self,
        ctx: &Context<'_>,
        mut data: CreateFavChatroomInput,
    ) -> async_graphql::Result<FavChatroom> {
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

        let favchatroom: FavChatroom = diesel::insert_into(favorite_chatroom::table)
            .values(&data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(favchatroom)
    }

    // Delete favchatroom
    pub async fn delete_favchatroom(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<FavChatroom> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        let favchatroom: FavChatroom = favorite_chatroom::table
            .filter(favorite_chatroom::id.eq(id))
            .limit(1)
            .first(&conn)?;

        match role {
            Role::User => {
                // Assert the user is the owner of the puzzle.
                user_id_guard(ctx, favchatroom.id)?;
            }
            Role::Admin => {}
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        let favchatroom =
            diesel::delete(favorite_chatroom::table.filter(favorite_chatroom::id.eq(id)))
                .get_result(&conn)
                .map_err(|err| async_graphql::Error::from(err))?;

        Ok(favchatroom)
    }
}
