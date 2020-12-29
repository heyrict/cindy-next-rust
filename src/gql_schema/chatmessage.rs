use async_graphql::{
    self, guard::Guard, Context, InputObject, MaybeUndefined, Object, Subscription,
};
use chrono::Utc;
use diesel::prelude::*;
use futures::{Stream, StreamExt};

use crate::auth::Role;
use crate::broker::CindyBroker;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::chatmessage::*;
use crate::models::*;
use crate::schema::chatmessage;

#[derive(Default)]
pub struct ChatmessageQuery;
#[derive(Default)]
pub struct ChatmessageMutation;
#[derive(Default)]
pub struct ChatmessageSubscription;

#[Object]
impl ChatmessageQuery {
    pub async fn chatmessage(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> async_graphql::Result<Chatmessage> {
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
        filter: Option<Vec<ChatmessageFilter>>,
        order: Option<Vec<ChatmessageOrder>>,
    ) -> async_graphql::Result<Vec<Chatmessage>> {
        use crate::schema::chatmessage::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = chatmessage.into_boxed();
        if let Some(order) = order {
            query = ChatmessageOrders::new(order).apply_order(query);
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

        let chatmessages = query.load::<Chatmessage>(&conn)?;

        Ok(chatmessages)
    }
}

#[derive(InputObject, Debug)]
pub struct UpdateChatmessageInput {
    pub id: Option<ID>,
    pub content: Option<String>,
    pub created: MaybeUndefined<Timestamptz>,
    pub edit_times: Option<i32>,
    pub chatroom_id: Option<ID>,
    pub user_id: Option<ID>,
    #[graphql(default_with = "Utc::now()")]
    pub modified: Timestamptz,
}

#[derive(AsChangeset)]
#[table_name = "chatmessage"]
pub struct UpdateChatmessageData {
    pub id: Option<ID>,
    pub content: Option<String>,
    pub created: Option<Option<Timestamptz>>,
    #[column_name = "editTimes"]
    pub edit_times: Option<i32>,
    pub chatroom_id: Option<ID>,
    pub user_id: Option<ID>,
    pub modified: Option<Timestamptz>,
}

impl From<UpdateChatmessageInput> for UpdateChatmessageData {
    fn from(x: UpdateChatmessageInput) -> Self {
        Self {
            id: x.id,
            content: x.content,
            created: x.created.as_options(),
            edit_times: x.edit_times,
            chatroom_id: x.chatroom_id,
            user_id: x.user_id,
            modified: Some(x.modified),
        }
    }
}

#[derive(InputObject, Insertable)]
#[table_name = "chatmessage"]
pub struct CreateChatmessageInput {
    pub id: Option<ID>,
    pub content: String,
    #[graphql(default_with = "Utc::now()")]
    pub created: Timestamptz,
    #[graphql(default = 0)]
    #[column_name = "editTimes"]
    pub edit_times: i32,
    pub chatroom_id: ID,
    pub user_id: Option<ID>,
    #[graphql(default_with = "Utc::now()")]
    pub modified: Timestamptz,
}

#[Object]
impl ChatmessageMutation {
    pub async fn update_chatmessage(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateChatmessageInput,
    ) -> async_graphql::Result<Chatmessage> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        let cm_inst: Chatmessage = chatmessage::table
            .filter(chatmessage::id.eq(id))
            .limit(1)
            .first(&conn)?;

        match role {
            Role::User => {
                // User should be the owner on update mutation
                user_id_guard(ctx, cm_inst.user_id)?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let chatmessage: Chatmessage = diesel::update(chatmessage::table)
            .filter(chatmessage::id.eq(id))
            .set(UpdateChatmessageData::from(set))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        CindyBroker::publish(ChatmessageSub::Updated(cm_inst, chatmessage.clone()));

        Ok(chatmessage)
    }

    pub async fn create_chatmessage(
        &self,
        ctx: &Context<'_>,
        mut data: CreateChatmessageInput,
    ) -> async_graphql::Result<Chatmessage> {
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

        let chatmessage: Chatmessage = diesel::insert_into(chatmessage::table)
            .values(&data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        CindyBroker::publish(ChatmessageSub::Created(chatmessage.clone()));

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
    ) -> async_graphql::Result<Chatmessage> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let chatmessage = diesel::delete(chatmessage::table.filter(chatmessage::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(chatmessage)
    }
}

#[derive(InputObject, Eq, PartialEq, Clone)]
pub struct ChatmessageSubFilter {
    id: Option<I32Filtering>,
    chatroom_id: Option<I32Filtering>,
}

impl RawFilter<Chatmessage> for ChatmessageSubFilter {
    fn check(&self, item: &Chatmessage) -> bool {
        if let Some(filter) = self.id.as_ref() {
            filter.check(&item.id)
        } else if let Some(filter) = self.chatroom_id.as_ref() {
            filter.check(&item.chatroom_id)
        } else {
            true
        }
    }
}

#[Subscription]
impl ChatmessageSubscription {
    pub async fn chatmessage_sub(
        &self,
        filter: Option<ChatmessageSubFilter>,
    ) -> impl Stream<Item = Option<ChatmessageSub>> {
        CindyBroker::<ChatmessageSub>::subscribe().filter(move |cm_sub| {
            let check = if let Some(filter) = filter.as_ref() {
                match cm_sub {
                    Some(ChatmessageSub::Created(cm)) => filter.check(&cm),
                    Some(ChatmessageSub::Updated(orig, _)) => filter.check(&orig),
                    None => false,
                }
            } else {
                cm_sub.is_some()
            };

            async move { check }
        })
    }
}
