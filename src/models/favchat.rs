use async_graphql::{self, Context, InputObject, Object};
use diesel::{prelude::*, query_dsl::QueryDsl, sql_types::Bool};

use crate::context::GlobalCtx;
use crate::schema::favorite_chatroom;

use super::*;

/// Available orders for favchat query
#[derive(InputObject, Clone)]
pub struct FavchatOrder {
    id: Option<Ordering>,
    chatroom_id: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct FavchatOrders(Vec<FavchatOrder>);

impl Default for FavchatOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl FavchatOrders {
    pub fn new(orders: Vec<FavchatOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: favorite_chatroom::BoxedQuery<'a, DB>,
    ) -> favorite_chatroom::BoxedQuery<'a, DB> {
        use crate::schema::favorite_chatroom::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
            gen_order!(obj, chatroom_id, query);
        }

        query
    }
}

/// Available filters for favchat query
#[derive(InputObject, Clone, Default)]
pub struct FavchatFilter {
    pub id: Option<I32Filtering>,
    pub chatroom_id: Option<I32Filtering>,
    pub user_id: Option<I32Filtering>,
}

impl CindyFilter<favorite_chatroom::table, DB> for FavchatFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<favorite_chatroom::table, DB, SqlType = Bool> + Send>>
    {
        use crate::schema::favorite_chatroom::dsl::*;

        let mut filter: Option<
            Box<dyn BoxableExpression<favorite_chatroom, DB, SqlType = Bool> + Send>,
        > = None;
        let FavchatFilter {
            id: obj_id,
            chatroom_id: obj_chatroom_id,
            user_id: obj_user_id,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_number_filter!(obj_chatroom_id: I32Filtering, chatroom_id, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);
        filter
    }
}

/// Object for favchat table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "favorite_chatroom"]
pub struct Favchat {
    pub id: ID,
    pub chatroom_id: ID,
    pub user_id: ID,
}

#[Object]
impl Favchat {
    async fn id(&self) -> ID {
        self.id
    }
    async fn chatroom_id(&self) -> ID {
        self.chatroom_id
    }
    async fn user_id(&self) -> ID {
        self.user_id
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

    async fn chatroom(&self, ctx: &Context<'_>) -> async_graphql::Result<Chatroom> {
        use crate::schema::chatroom;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let chatroom_inst = chatroom::table
            .filter(chatroom::id.eq(self.chatroom_id))
            .limit(1)
            .first(&conn)?;

        Ok(chatroom_inst)
    }
}
