use async_graphql::{self, Context, InputObject, Object};
use diesel::{
    prelude::*,
    query_dsl::QueryDsl,
    sql_types::{Bool},
};

use crate::context::GlobalCtx;
use crate::schema::chatroom;

use super::*;

/// Available orders for chatroom query
#[derive(InputObject, Clone)]
pub struct ChatroomOrder {
    id: Option<Ordering>,
    created: Option<Ordering>,
    private: Option<Ordering>,
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
            gen_order!(obj, private, query);
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
    private: Option<bool>,
}

impl CindyFilter<chatroom::table, DB> for ChatroomFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<chatroom::table, DB, SqlType = Bool> + Send>>
    {
        use crate::schema::chatroom::dsl::*;

        let mut filter: Option<
            Box<dyn BoxableExpression<chatroom, DB, SqlType = Bool> + Send>,
        > = None;
        let ChatroomFilter {
            id: obj_id,
            name: obj_name,
            description: obj_description,
            created: obj_created,
            user_id: obj_user_id,
            private: obj_private,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_string_filter!(obj_name, name, filter);
        gen_string_filter!(obj_description, description, filter);
        gen_number_filter!(obj_created: DateFiltering, created, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);
        gen_bool_filter!(obj_private, private, filter);
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
    pub private: bool,
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
    async fn private(&self) -> bool {
        self.private
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
}
