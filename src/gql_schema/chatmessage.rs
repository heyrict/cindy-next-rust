use async_graphql::{self, guard::Guard, Context, InputObject, Object};
use diesel::prelude::*;

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::chatmessage::*;
use crate::models::*;
use crate::schema::chatmessage;

#[derive(Default)]
pub struct ChatMessageQuery;
#[derive(Default)]
pub struct ChatMessageMutation;

#[Object]
impl ChatMessageQuery {
    pub async fn chatmessage(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<ChatMessage> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let chatmessage = chatmessage::table
            .filter(chatmessage::id.eq(id))
            .limit(1)
            .first(&conn)?;

        Ok(chatmessage)
    }

    pub async fn chatmessages(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<ChatMessageFilter>>,
        order: Option<Vec<ChatMessageOrder>>,
    ) -> async_graphql::Result<Vec<ChatMessage>> {
        use crate::schema::chatmessage::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = chatmessage.into_boxed();
        if let Some(order) = order {
            query = ChatMessageOrders::new(order).apply_order(query);
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

        let chatmessages = query.load::<ChatMessage>(&conn)?;

        Ok(chatmessages)
    }
}

#[derive(InputObject, AsChangeset, Debug)]
#[table_name = "chatmessage"]
pub struct UpdateChatMessageInput {
    pub id: Option<ID>,
    pub content: Option<String>,
    #[column_name = "editTimes"]
    pub edit_times: Option<i32>,
    pub created: Option<Timestamptz>,
    pub modified: Option<Timestamptz>,
    pub user_id: Option<ID>,
    pub chatroom_id: Option<ID>,
}

#[derive(InputObject, Insertable)]
#[table_name = "chatmessage"]
pub struct CreateChatMessageInput {
    pub id: Option<ID>,
    pub content: String,
    #[column_name = "editTimes"]
    pub edit_times: Option<i32>,
    pub created: Option<Timestamptz>,
    pub modified: Option<Timestamptz>,
    pub user_id: Option<ID>,
    pub chatroom_id: ID,
}

#[Object]
impl ChatMessageMutation {
    pub async fn update_chatmessage(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateChatMessageInput,
    ) -> async_graphql::Result<ChatMessage> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // User should be the owner on update mutation
                let chatmessage_inst: ChatMessage = chatmessage::table
                    .filter(chatmessage::id.eq(id))
                    .limit(1)
                    .first(&conn)?;
                user_id_guard(ctx, chatmessage_inst.user_id)?;
                assert_eq_guard_msg(set.edit_times, None, "`edit_times` should not be set")?;
                assert_eq_guard_msg(set.created, None, "`created` should not be set")?;
                assert_eq_guard_msg(set.modified, None, "`modified` should not be set")?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let chatmessage: ChatMessage = diesel::update(chatmessage::table)
            .filter(chatmessage::id.eq(id))
            .set(set)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(chatmessage)
    }

    pub async fn create_chatmessage(
        &self,
        ctx: &Context<'_>,
        mut data: CreateChatMessageInput,
    ) -> async_graphql::Result<ChatMessage> {
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
                assert_eq_guard_msg(data.edit_times, None, "`edit_times` should not be set")?;
                assert_eq_guard_msg(data.created, None, "`created` should not be set")?;
                assert_eq_guard_msg(data.modified, None, "`modified` should not be set")?;
            }
            Role::Admin => {}
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        let chatmessage: ChatMessage = diesel::insert_into(chatmessage::table)
            .values(&data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(chatmessage)
    }

    // Delete chatmessage (admin only)
    #[graphql(guard(and(
        DenyRoleGuard(role = "Role::User"),
        DenyRoleGuard(role = "Role::Guest")
    )))]
    pub async fn delete_chatmessage(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<ChatMessage> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let chatmessage = diesel::delete(chatmessage::table.filter(chatmessage::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(chatmessage)
    }
}
