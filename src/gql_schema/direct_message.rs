use async_graphql::{self, guard::Guard, Context, InputObject, Object, Subscription};
use chrono::Utc;
use diesel::prelude::*;
use futures::{Stream, StreamExt};

use crate::auth::Role;
use crate::broker::CindyBroker;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::direct_message::*;
use crate::models::*;
use crate::schema::direct_message;

#[derive(Default)]
pub struct DirectMessageQuery;
#[derive(Default)]
pub struct DirectMessageMutation;
#[derive(Default)]
pub struct DirectMessageSubscription;

#[Object]
impl DirectMessageQuery {
    pub async fn direct_message(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> async_graphql::Result<DirectMessage> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let direct_message = direct_message::table
            .filter(direct_message::id.eq(id))
            .limit(1)
            .first(&conn)?;

        Ok(direct_message)
    }

    pub async fn direct_messages(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<DirectMessageFilter>>,
        order: Option<Vec<DirectMessageOrder>>,
    ) -> async_graphql::Result<Vec<DirectMessage>> {
        use crate::schema::direct_message::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = direct_message.into_boxed();
        if let Some(order) = order {
            query = DirectMessageOrders::new(order).apply_order(query);
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

        let direct_messages = query.load::<DirectMessage>(&conn)?;

        Ok(direct_messages)
    }
}

#[derive(AsChangeset, InputObject, Debug)]
#[table_name = "direct_message"]
pub struct UpdateDirectMessageInput {
    pub id: Option<ID>,
    pub content: Option<String>,
    pub created: Option<Timestamptz>,
    #[column_name = "editTimes"]
    pub edit_times: Option<i32>,
    pub sender_id: Option<ID>,
    pub receiver_id: Option<ID>,
    #[graphql(default_with = "Utc::now()")]
    pub modified: Timestamptz,
}

#[derive(InputObject, Insertable)]
#[table_name = "direct_message"]
pub struct CreateDirectMessageInput {
    pub id: Option<ID>,
    pub content: String,
    #[graphql(default_with = "Utc::now()")]
    pub created: Timestamptz,
    #[graphql(default = 0)]
    #[column_name = "editTimes"]
    pub edit_times: i32,
    pub sender_id: Option<ID>,
    pub receiver_id: ID,
    #[graphql(default_with = "Utc::now()")]
    pub modified: Timestamptz,
}

#[Object]
impl DirectMessageMutation {
    pub async fn update_direct_message(
        &self,
        ctx: &Context<'_>,
        id: ID,
        mut set: UpdateDirectMessageInput,
    ) -> async_graphql::Result<DirectMessage> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        let cm_inst: DirectMessage = direct_message::table
            .filter(direct_message::id.eq(id))
            .limit(1)
            .first(&conn)?;

        match role {
            Role::User => {
                // User should be the owner on update mutation
                user_id_guard(ctx, cm_inst.sender_id)?;
                // Increase edit_times for user
                set.edit_times = Some(cm_inst.edit_times + 1);
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let direct_message: DirectMessage = diesel::update(direct_message::table)
            .filter(direct_message::id.eq(id))
            .set(set)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        CindyBroker::publish(DirectMessageSub::Updated(cm_inst, direct_message.clone()));

        Ok(direct_message)
    }

    pub async fn create_direct_message(
        &self,
        ctx: &Context<'_>,
        mut data: CreateDirectMessageInput,
    ) -> async_graphql::Result<DirectMessage> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // Assert the user is the owner of the puzzle.
                if let Some(user_id) = data.sender_id {
                    user_id_guard(ctx, user_id)?;
                } else {
                    data.sender_id = reqctx.get_user_id();
                };
            }
            Role::Admin => {}
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        let direct_message: DirectMessage = diesel::insert_into(direct_message::table)
            .values(&data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        CindyBroker::publish(DirectMessageSub::Created(direct_message.clone()));

        Ok(direct_message)
    }

    // Delete direct_message (admin only)
    #[graphql(guard(and(
        DenyRoleGuard(role = "Role::User"),
        DenyRoleGuard(role = "Role::Guest")
    )))]
    pub async fn delete_direct_message(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<DirectMessage> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let direct_message =
            diesel::delete(direct_message::table.filter(direct_message::id.eq(id)))
                .get_result(&conn)
                .map_err(|err| async_graphql::Error::from(err))?;

        Ok(direct_message)
    }
}

#[Subscription]
impl DirectMessageSubscription {
    pub async fn direct_message_sub(
        &self,
        user_id: ID,
    ) -> impl Stream<Item = Option<DirectMessageSub>> {
        CindyBroker::<DirectMessageSub>::subscribe().filter(move |dm_sub| {
            let check = match dm_sub {
                Some(DirectMessageSub::Created(dm)) => {
                    dm.sender_id == user_id || dm.receiver_id == user_id
                }
                Some(DirectMessageSub::Updated(orig, _)) => {
                    orig.sender_id == user_id || orig.receiver_id == user_id
                }
                None => false,
            };

            async move { check }
        })
    }
}
