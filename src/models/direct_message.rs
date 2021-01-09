use async_graphql::{self, Context, InputObject, Object};
use diesel::{prelude::*, query_dsl::QueryDsl, sql_types::Bool};

use crate::context::GlobalCtx;
use crate::schema::direct_message;

use super::*;

/// Available orders for direct_message query
#[derive(InputObject, Clone)]
pub struct DirectMessageOrder {
    id: Option<Ordering>,
    created: Option<Ordering>,
    modified: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct DirectMessageOrders(Vec<DirectMessageOrder>);

impl Default for DirectMessageOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl DirectMessageOrders {
    pub fn new(orders: Vec<DirectMessageOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: direct_message::BoxedQuery<'a, DB>,
    ) -> direct_message::BoxedQuery<'a, DB> {
        use crate::schema::direct_message::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
            gen_order!(obj, created, query);
            gen_order!(obj, modified, query);
        }

        query
    }
}

/// Available filters for direct_message query
#[derive(InputObject, Clone, Default)]
pub struct DirectMessageFilter {
    pub id: Option<I32Filtering>,
    pub content: Option<StringFiltering>,
    pub created: Option<NullableTimestamptzFiltering>,
    pub receiver_id: Option<I32Filtering>,
    pub sender_id: Option<I32Filtering>,
    pub modified: Option<TimestamptzFiltering>,
}

impl CindyFilter<direct_message::table, DB> for DirectMessageFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<direct_message::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::direct_message::dsl::*;

        let mut filter: Option<
            Box<dyn BoxableExpression<direct_message, DB, SqlType = Bool> + Send>,
        > = None;
        let DirectMessageFilter {
            id: obj_id,
            content: obj_content,
            created: obj_created,
            receiver_id: obj_receiver_id,
            sender_id: obj_sender_id,
            modified: obj_modified,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_string_filter!(obj_content, content, filter);
        gen_nullable_number_filter!(obj_created: NullableTimestamptzFiltering, created, filter);
        gen_number_filter!(obj_receiver_id: I32Filtering, receiver_id, filter);
        gen_number_filter!(obj_sender_id: I32Filtering, sender_id, filter);
        gen_number_filter!(obj_modified: TimestamptzFiltering, modified, filter);
        filter
    }
}

#[derive(Clone)]
pub enum DirectMessageSub {
    Created(DirectMessage),
    Updated(DirectMessage, DirectMessage),
}

#[Object]
impl DirectMessageSub {
    async fn op(&self) -> DbOp {
        match &self {
            DirectMessageSub::Created(_) => DbOp::Created,
            DirectMessageSub::Updated(_, _) => DbOp::Updated,
        }
    }

    async fn data(&self) -> DirectMessage {
        match &self {
            DirectMessageSub::Created(dm) => dm.clone(),
            DirectMessageSub::Updated(_, dm) => dm.clone(),
        }
    }
}

/// Object for direct_message table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "direct_message"]
pub struct DirectMessage {
    pub id: ID,
    pub content: String,
    pub created: Timestamptz,
    pub receiver_id: ID,
    pub sender_id: ID,
    #[column_name = "editTimes"]
    pub edit_times: i32,
    pub modified: Timestamptz,
}

#[Object]
impl DirectMessage {
    async fn id(&self) -> ID {
        self.id
    }
    async fn content(&self) -> &str {
        &self.content
    }
    async fn created(&self) -> Timestamptz {
        self.created
    }
    async fn edit_times(&self) -> i32 {
        self.edit_times
    }
    async fn receiver_id(&self) -> ID {
        self.receiver_id
    }
    async fn sender_id(&self) -> ID {
        self.sender_id
    }
    async fn modified(&self) -> Timestamptz {
        self.modified
    }

    async fn sender(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        use crate::schema::user;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user = user::table
            .filter(user::id.eq(self.sender_id))
            .limit(1)
            .first(&conn)?;

        Ok(user)
    }

    async fn receiver(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        use crate::schema::user;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user = user::table
            .filter(user::id.eq(self.receiver_id))
            .limit(1)
            .first(&conn)?;

        Ok(user)
    }
}
