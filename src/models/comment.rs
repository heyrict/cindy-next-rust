use async_graphql::{self, Context, InputObject, Object};
use diesel::sql_types::Bool;
use diesel::{prelude::*, query_dsl::QueryDsl};

use crate::context::GlobalCtx;
use crate::schema::comment;

use super::generics::*;
use super::{Puzzle, User};

/// Available orders for comment query
#[derive(InputObject, Clone)]
pub struct CommentOrder {
    id: Option<Ordering>,
    spoiler: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct CommentOrders(Vec<CommentOrder>);

impl Default for CommentOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl CommentOrders {
    pub fn new(orders: Vec<CommentOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: crate::schema::comment::BoxedQuery<'a, DB>,
    ) -> crate::schema::comment::BoxedQuery<'a, DB> {
        use crate::schema::comment::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
            gen_order!(obj, spoiler, query);
        }

        query
    }
}

/// Available filters for comment query
#[derive(InputObject, Clone, Default)]
pub struct CommentFilter {
    pub id: Option<I32Filtering>,
    pub content: Option<StringFiltering>,
    pub spoiler: Option<bool>,
    pub puzzle_id: Option<I32Filtering>,
    pub user_id: Option<I32Filtering>,
}

impl CindyFilter<comment::table, DB> for CommentFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<comment::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::comment::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<comment, DB, SqlType = Bool> + Send>> =
            None;
        let CommentFilter {
            id: obj_id,
            content: obj_content,
            spoiler: obj_spoiler,
            puzzle_id: obj_puzzle_id,
            user_id: obj_user_id,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_string_filter!(obj_content, content, filter);
        gen_number_filter!(obj_puzzle_id: I32Filtering, puzzle_id, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);

        let eq = obj_spoiler;
        apply_filter!(eq, spoiler, filter);

        filter
    }
}

/// Available filters for comment_count query
#[derive(InputObject, Clone, Default)]
pub struct CommentCountFilter {
    pub puzzle_id: Option<I32Filtering>,
    pub user_id: Option<I32Filtering>,
}

impl CindyFilter<comment::table, DB> for CommentCountFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<comment::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::comment::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<comment, DB, SqlType = Bool> + Send>> =
            None;
        let CommentCountFilter {
            puzzle_id: obj_puzzle_id,
            user_id: obj_user_id,
        } = self;
        gen_number_filter!(obj_puzzle_id: I32Filtering, puzzle_id, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);

        filter
    }
}

/// Object for comment table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "comment"]
pub struct Comment {
    pub id: ID,
    pub content: String,
    pub spoiler: bool,
    pub puzzle_id: ID,
    pub user_id: ID,
}

#[Object]
impl Comment {
    async fn id(&self) -> ID {
        self.id
    }
    async fn content(&self) -> &str {
        &self.content
    }
    async fn spoiler(&self) -> bool {
        self.spoiler
    }
    async fn puzzle_id(&self) -> ID {
        self.puzzle_id
    }
    async fn user_id(&self) -> ID {
        self.user_id
    }

    async fn puzzle(&self, ctx: &Context<'_>) -> async_graphql::Result<Puzzle> {
        use crate::schema::puzzle;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let puzzle = puzzle::table
            .filter(puzzle::id.eq(self.puzzle_id))
            .limit(1)
            .first(&conn)?;

        Ok(puzzle)
    }

    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        use crate::schema::user;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user = user::table
            .filter(user::id.eq(self.user_id))
            .limit(1)
            .first(&conn)?;

        Ok(user)
    }
}
