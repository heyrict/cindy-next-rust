use async_graphql::{self, Context, InputObject, Object};
use diesel::{
    prelude::*,
    query_dsl::QueryDsl,
    sql_types::{Bool, Nullable},
};

use crate::context::GlobalCtx;
use crate::schema::chatmessage;

use super::*;

/// Available orders for chatmessage query
#[derive(InputObject, Clone)]
pub struct ChatMessageOrder {
    id: Option<Ordering>,
    created: Option<Ordering>,
    chatroom_id: Option<Ordering>,
    user_id: Option<Ordering>,
    modified: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct ChatMessageOrders(Vec<ChatMessageOrder>);

impl Default for ChatMessageOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl ChatMessageOrders {
    pub fn new(orders: Vec<ChatMessageOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: chatmessage::BoxedQuery<'a, DB>,
    ) -> chatmessage::BoxedQuery<'a, DB> {
        use crate::schema::chatmessage::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
            gen_order!(obj, created, query);
            gen_order!(obj, chatroom_id, query);
            gen_order!(obj, user_id, query);
            gen_order!(obj, modified, query);
        }

        query
    }
}

/// Available filters for chatmessage query
#[derive(InputObject, Clone)]
pub struct ChatMessageFilter {
    id: Option<I32Filtering>,
    content: Option<StringFiltering>,
    created: Option<NullableTimestamptzFiltering>,
    modified: Option<TimestamptzFiltering>,
    chatroom_id: Option<I32Filtering>,
    user_id: Option<I32Filtering>,
}

impl CindyFilter<chatmessage::table, DB> for ChatMessageFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<chatmessage::table, DB, SqlType = Nullable<Bool>> + Send>>
    {
        use crate::schema::chatmessage::dsl::*;

        let mut filter: Option<
            Box<dyn BoxableExpression<chatmessage, DB, SqlType = Nullable<Bool>> + Send>,
        > = None;
        let ChatMessageFilter {
            id: obj_id,
            content: obj_content,
            created: obj_created,
            chatroom_id: obj_chatroom_id,
            user_id: obj_user_id,
            modified: obj_modified,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_string_filter!(obj_content, content, filter);
        gen_nullable_number_filter!(obj_created: NullableTimestamptzFiltering, created, filter);
        gen_number_filter!(obj_modified: TimestamptzFiltering, created, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);
        gen_number_filter!(obj_chatroom_id: I32Filtering, chatroom_id, filter);
        filter
    }
}

/// Object for chatmessage table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "chatmessage"]
pub struct ChatMessage {
    pub id: ID,
    pub content: String,
    pub created: Option<Timestamptz>,
    #[column_name = "editTimes"]
    pub edit_times: i32,
    pub chatroom_id: ID,
    pub user_id: ID,
    pub modified: Timestamptz,
}

#[Object]
impl ChatMessage {
    async fn id(&self) -> ID {
        self.id
    }
    async fn content(&self) -> &str {
        &self.content
    }
    async fn edit_times(&self) -> i32 {
        self.edit_times
    }
    async fn created(&self) -> Option<Timestamptz> {
        self.created
    }
    async fn modified(&self) -> Timestamptz {
        self.modified
    }
    async fn user_id(&self) -> ID {
        self.user_id
    }
    async fn chatroom_id(&self) -> ID {
        self.chatroom_id
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
