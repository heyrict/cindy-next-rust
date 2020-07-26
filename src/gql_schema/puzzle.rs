use async_graphql::{Context, FieldResult};
use diesel::prelude::*;

use crate::context::GlobalCtx;
use crate::models::{Date, Timestamptz, ID};
use crate::schema::puzzle;

use super::*;

impl QueryRoot {
    pub async fn puzzle_(&self, ctx: &Context<'_>, id: i32) -> FieldResult<Puzzle> {
        let conn = ctx.data::<GlobalCtx>().get_conn()?;

        let puzzle = puzzle::table
            .filter(puzzle::id.eq(id))
            .limit(1)
            .first(&conn)?;

        Ok(puzzle)
    }

    pub async fn puzzles_(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<PuzzleFilter>>,
        order: Option<Vec<PuzzleOrder>>,
    ) -> FieldResult<Vec<Puzzle>> {
        use crate::schema::puzzle::dsl::*;

        let conn = ctx.data::<GlobalCtx>().get_conn()?;

        let mut query = puzzle.into_boxed();
        if let Some(order) = order {
            query = PuzzleOrders::new(order).apply_order(query);
        }
        if let Some(filter) = filter {
            query = PuzzleFilters::new(filter).apply_filter(query);
        }
        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let puzzles = query.load::<Puzzle>(&conn)?;

        Ok(puzzles)
    }
}

#[async_graphql::InputObject]
#[derive(AsChangeset, Debug)]
#[table_name = "puzzle"]
pub struct UpdatePuzzleSet {
    pub title: Option<String>,
    pub yami: Option<i32>,
    pub genre: Option<i32>,
    pub content: Option<String>,
    pub solution: Option<String>,
    pub created: Option<Timestamptz>,
    pub modified: Option<Timestamptz>,
    pub status: Option<i32>,
    pub memo: Option<String>,
    pub user_id: Option<i32>,
    pub anonymous: Option<bool>,
    pub dazed_on: Option<Date>,
    pub grotesque: Option<bool>,
}

impl MutationRoot {
    pub async fn update_puzzle_(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdatePuzzleSet,
    ) -> FieldResult<Puzzle> {
        let conn = ctx.data::<GlobalCtx>().get_conn()?;
        diesel::update(puzzle::table)
            .filter(puzzle::id.eq(id))
            .set(&set)
            .get_result(&conn)
            .map_err(|err| err.into())
    }
}
