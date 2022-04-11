use async_graphql::{self, Context, InputObject, Object};
use diesel::{prelude::*, query_dsl::QueryDsl, sql_types::Bool};

use crate::context::GlobalCtx;
use crate::schema::chatroom;

use super::chatmessage::{ChatmessageFilter, ChatmessageOrder};
use super::*;

/// Available orders for chatroom query
#[derive(InputObject, Clone)]
pub struct ChatroomOrder {
    id: Option<Ordering>,
    created: Option<Ordering>,
    official: Option<Ordering>,
    public: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct ChatroomOrders(Vec<ChatroomOrder>);

impl Default for ChatroomOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl ChatroomOrders {
    pub fn new(orders: Vec<ChatroomOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: chatroom::BoxedQuery<'a, DB>,
    ) -> chatroom::BoxedQuery<'a, DB> {
        use crate::schema::chatroom::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
            gen_order!(obj, created, query);
            gen_order!(obj, official, query);
            gen_order!(obj, public, query);
        }

        query
    }
}

/// Available filters for chatroom query
#[derive(InputObject, Clone)]
pub struct ChatroomFilter {
    id: Option<I32Filtering>,
    name: Option<StringFiltering>,
    description: Option<StringFiltering>,
    created: Option<DateFiltering>,
    user_id: Option<I32Filtering>,
    official: Option<bool>,
    public: Option<bool>,
}

impl CindyFilter<chatroom::table, DB> for ChatroomFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<chatroom::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::chatroom::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<chatroom, DB, SqlType = Bool> + Send>> =
            None;
        let ChatroomFilter {
            id: obj_id,
            name: obj_name,
            description: obj_description,
            created: obj_created,
            user_id: obj_user_id,
            official: obj_official,
            public: obj_public,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_string_filter!(obj_name, name, filter);
        gen_string_filter!(obj_description, description, filter);
        gen_number_filter!(obj_created: DateFiltering, created, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);
        gen_bool_filter!(obj_official, official, filter);
        gen_bool_filter!(obj_public, public, filter);
        filter
    }
}

/// Available filters for chatroom_count query
#[derive(InputObject, Clone, Default)]
pub struct ChatroomCountFilter {
    name: Option<StringFiltering>,
    created: Option<DateFiltering>,
    user_id: Option<I32Filtering>,
    official: Option<bool>,
    public: Option<bool>,
}

impl CindyFilter<chatroom::table, DB> for ChatroomCountFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<chatroom::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::chatroom::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<chatroom, DB, SqlType = Bool> + Send>> =
            None;
        let ChatroomCountFilter {
            name: obj_name,
            created: obj_created,
            user_id: obj_user_id,
            official: obj_official,
            public: obj_public,
        } = self;
        gen_string_filter!(obj_name, name, filter);
        gen_number_filter!(obj_created: DateFiltering, created, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);
        gen_bool_filter!(obj_official, official, filter);
        gen_bool_filter!(obj_public, public, filter);
        filter
    }
}

/// Object for chatroom table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "chatroom"]
pub struct Chatroom {
    pub id: ID,
    pub name: String,
    pub description: String,
    pub created: Date,
    pub user_id: ID,
    pub official: bool,
    pub public: bool,
}

#[Object]
impl Chatroom {
    async fn id(&self) -> ID {
        self.id
    }
    async fn name(&self) -> &str {
        &self.name
    }
    async fn description(&self) -> &str {
        &self.description
    }
    async fn created(&self) -> Date {
        self.created
    }
    async fn user_id(&self) -> ID {
        self.user_id
    }
    async fn official(&self) -> bool {
        self.official
    }
    async fn public(&self) -> bool {
        self.public
    }

    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        use crate::schema::user;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user_inst = user::table
            .filter(user::id.eq(self.user_id))
            .limit(1)
            .first(&conn)?;

        Ok(user_inst)
    }

    async fn chatmessages(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<ChatmessageFilter>,
        order: Option<Vec<ChatmessageOrder>>,
    ) -> async_graphql::Result<Vec<Chatmessage>> {
        use crate::gql_schema::ChatmessageQuery;

        let filter = filter
            .map(|mut filter| {
                filter.chatroom_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| ChatmessageFilter {
                chatroom_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = ChatmessageQuery::default();
        query
            .chatmessages(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }
}
