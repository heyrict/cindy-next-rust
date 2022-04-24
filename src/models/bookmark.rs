use async_graphql::{self, Context, InputObject, Object};
use diesel::{prelude::*, query_dsl::QueryDsl, sql_types::Bool};

use crate::context::GlobalCtx;
use crate::schema::bookmark;

use super::*;

/// Available orders for bookmark query
#[derive(InputObject, Clone)]
pub struct BookmarkOrder {
    id: Option<Ordering>,
    value: Option<Ordering>,
    puzzle_id: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct BookmarkOrders(Vec<BookmarkOrder>);

impl Default for BookmarkOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl BookmarkOrders {
    pub fn new(orders: Vec<BookmarkOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: bookmark::BoxedQuery<'a, DB>,
    ) -> bookmark::BoxedQuery<'a, DB> {
        use crate::schema::bookmark::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
            gen_order!(obj, value, query);
            gen_order!(obj, puzzle_id, query);
        }

        query
    }
}

/// Available filters for bookmark query
#[derive(InputObject, Clone, Default)]
pub struct BookmarkFilter {
    pub id: Option<I32Filtering>,
    pub value: Option<I16Filtering>,
    pub puzzle_id: Option<I32Filtering>,
    pub user_id: Option<I32Filtering>,
}

impl CindyFilter<bookmark::table> for BookmarkFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<bookmark::table, DB, SqlType = Bool>>> {
        use crate::schema::bookmark::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<bookmark, DB, SqlType = Bool>>> =
            None;
        let BookmarkFilter {
            id: obj_id,
            value: obj_value,
            puzzle_id: obj_puzzle_id,
            user_id: obj_user_id,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_number_filter!(obj_value: I16Filtering, value, filter);
        gen_number_filter!(obj_puzzle_id: I32Filtering, puzzle_id, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);
        filter
    }
}

/// Available filters for bookmark_count query
#[derive(InputObject, Clone, Default)]
pub struct BookmarkCountFilter {
    pub puzzle_id: Option<I32Filtering>,
    pub user_id: Option<I32Filtering>,
}

impl CindyFilter<bookmark::table> for BookmarkCountFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<bookmark::table, DB, SqlType = Bool>>> {
        use crate::schema::bookmark::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<bookmark, DB, SqlType = Bool>>> =
            None;
        let BookmarkCountFilter {
            puzzle_id: obj_puzzle_id,
            user_id: obj_user_id,
        } = self;
        gen_number_filter!(obj_puzzle_id: I32Filtering, puzzle_id, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);
        filter
    }
}

/// Object for bookmark table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[diesel(table_name = bookmark)]
pub struct Bookmark {
    pub id: ID,
    pub value: i16,
    pub puzzle_id: ID,
    pub user_id: ID,
}

#[Object]
impl Bookmark {
    async fn id(&self) -> ID {
        self.id
    }
    async fn value(&self) -> i16 {
        self.value
    }
    async fn puzzle_id(&self) -> ID {
        self.puzzle_id
    }
    async fn user_id(&self) -> ID {
        self.user_id
    }

    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        use crate::schema::user;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user = user::table
            .filter(user::id.eq(self.user_id))
            .limit(1)
            .first(&mut conn)?;

        Ok(user)
    }

    async fn puzzle(&self, ctx: &Context<'_>) -> async_graphql::Result<Puzzle> {
        use crate::schema::puzzle;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let puzzle = puzzle::table
            .filter(puzzle::id.eq(self.puzzle_id))
            .limit(1)
            .first(&mut conn)?;

        Ok(puzzle)
    }
}
