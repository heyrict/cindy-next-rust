use async_graphql::{self, Context, InputObject, Object};
use chrono::Utc;
use diesel::prelude::*;

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::chatroom::*;
use crate::models::*;
use crate::schema::chatroom;

#[derive(Default)]
pub struct ChatroomQuery;
#[derive(Default)]
pub struct ChatroomMutation;

#[Object]
impl ChatroomQuery {
    pub async fn chatroom(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Chatroom> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let chatroom = chatroom::table
            .filter(chatroom::id.eq(id))
            .limit(1)
            .first(&conn)?;

        Ok(chatroom)
    }

    pub async fn chatrooms(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<ChatroomFilter>>,
        order: Option<Vec<ChatroomOrder>>,
    ) -> async_graphql::Result<Vec<Chatroom>> {
        use crate::schema::chatroom::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = chatroom.into_boxed();
        if let Some(order) = order {
            query = ChatroomOrders::new(order).apply_order(query);
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

        let chatrooms = query.load::<Chatroom>(&conn)?;

        Ok(chatrooms)
    }

    pub async fn chatroom_count(
        &self,
        ctx: &Context<'_>,
        filter: Option<ChatroomCountFilter>,
    ) -> async_graphql::Result<i64> {
        use crate::schema::chatroom::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = chatroom.into_boxed();
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
#[table_name = "chatroom"]
pub struct UpdateChatroomInput {
    pub id: Option<ID>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created: Option<Date>,
    pub user_id: Option<ID>,
    pub official: Option<bool>,
    pub public: Option<bool>,
}

#[derive(InputObject, Insertable)]
#[table_name = "chatroom"]
pub struct CreateChatroomInput {
    pub id: Option<ID>,
    pub name: Option<String>,
    pub description: Option<String>,
    #[graphql(default_with = "Utc::today().naive_utc()")]
    pub created: Date,
    pub user_id: Option<ID>,
    pub official: Option<bool>,
    pub public: Option<bool>,
}

#[Object]
impl ChatroomMutation {
    pub async fn update_chatroom(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateChatroomInput,
    ) -> async_graphql::Result<Chatroom> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // User should be the owner on update mutation
                let chatroom_inst: Chatroom = chatroom::table
                    .filter(chatroom::id.eq(id))
                    .limit(1)
                    .first(&conn)?;
                user_id_guard(ctx, chatroom_inst.user_id)?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let chatroom: Chatroom = diesel::update(chatroom::table)
            .filter(chatroom::id.eq(id))
            .set(set)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(chatroom)
    }

    pub async fn create_chatroom(
        &self,
        ctx: &Context<'_>,
        mut data: CreateChatroomInput,
    ) -> async_graphql::Result<Chatroom> {
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

        let chatroom: Chatroom = diesel::insert_into(chatroom::table)
            .values(&data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(chatroom)
    }

    // Delete chatroom (admin only)
    #[graphql(guard = "DenyRoleGuard::new(Role::User).and(DenyRoleGuard::new(Role::Guest))")]
    pub async fn delete_chatroom(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Chatroom> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let chatroom = diesel::delete(chatroom::table.filter(chatroom::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(chatroom)
    }
}
