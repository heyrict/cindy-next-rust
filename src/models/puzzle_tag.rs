use async_graphql::{self, Context, InputObject, Object};
use diesel::sql_types::Bool;
use diesel::{prelude::*, query_dsl::QueryDsl};

use crate::context::GlobalCtx;
use crate::schema::puzzle_tag;

use super::generics::*;
use super::{Puzzle, Tag, User};

/// Available orders for puzzle_tag query
#[derive(InputObject, Clone)]
pub struct PuzzleTagOrder {
    id: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct PuzzleTagOrders(Vec<PuzzleTagOrder>);

impl Default for PuzzleTagOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl PuzzleTagOrders {
    pub fn new(orders: Vec<PuzzleTagOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: crate::schema::puzzle_tag::BoxedQuery<'a, DB>,
    ) -> crate::schema::puzzle_tag::BoxedQuery<'a, DB> {
        use crate::schema::puzzle_tag::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
        }

        query
    }
}

/// Available filters for puzzle_tag query
#[derive(InputObject, Clone)]
pub struct PuzzleTagFilter {
    id: Option<I32Filtering>,
    puzzle_id: Option<I32Filtering>,
    tag_id: Option<I32Filtering>,
    user_id: Option<I32Filtering>,
}

impl CindyFilter<puzzle_tag::table, DB> for PuzzleTagFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<puzzle_tag::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::puzzle_tag::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<puzzle_tag, DB, SqlType = Bool> + Send>> =
            None;
        let PuzzleTagFilter {
            id: obj_id,
            puzzle_id: obj_puzzle_id,
            tag_id: obj_tag_id,
            user_id: obj_user_id,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_number_filter!(obj_puzzle_id: I32Filtering, puzzle_id, filter);
        gen_number_filter!(obj_tag_id: I32Filtering, tag_id, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);

        filter
    }
}

/// Object for puzzle_tag table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "puzzle_tag"]
pub struct PuzzleTag {
    pub id: ID,
    pub puzzle_id: ID,
    pub tag_id: ID,
    pub user_id: ID,
}

#[Object]
impl PuzzleTag {
    async fn id(&self) -> ID {
        self.id
    }
    async fn puzzle_id(&self) -> ID {
        self.puzzle_id
    }
    async fn tag_id(&self) -> ID {
        self.tag_id
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

    async fn tag(&self, ctx: &Context<'_>) -> async_graphql::Result<Tag> {
        use crate::schema::tag;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let tag = tag::table
            .filter(tag::id.eq(self.tag_id))
            .limit(1)
            .first(&conn)?;

        Ok(tag)
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
