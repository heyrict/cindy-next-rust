use async_graphql::{self, Context, InputObject, Object, Subscription};
use diesel::prelude::*;
use futures::{Stream, StreamExt};

use crate::broker::CindyBroker;
use crate::context::GlobalCtx;
use crate::models::*;
use crate::schema::puzzle;

#[derive(Default)]
pub struct PuzzleQuery;
#[derive(Default)]
pub struct PuzzleMutation;
#[derive(Default)]
pub struct PuzzleSubscription;

#[Object]
impl PuzzleQuery {
    pub async fn puzzle(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Puzzle> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let puzzle = puzzle::table
            .filter(puzzle::id.eq(id))
            .limit(1)
            .first(&conn)?;

        Ok(puzzle)
    }

    pub async fn puzzles(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<PuzzleFilter>>,
        order: Option<Vec<PuzzleOrder>>,
    ) -> async_graphql::Result<Vec<Puzzle>> {
        use crate::schema::puzzle::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = puzzle.into_boxed();
        if let Some(order) = order {
            query = PuzzleOrders::new(order).apply_order(query);
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

        let puzzles = query.load::<Puzzle>(&conn)?;

        Ok(puzzles)
    }
}

#[derive(InputObject)]
pub struct UpdatePuzzleSet {
    pub title: Option<String>,
    pub yami: Option<Yami>,
    pub genre: Option<Genre>,
    pub content: Option<String>,
    pub solution: Option<String>,
    pub created: Option<Timestamptz>,
    pub modified: Option<Timestamptz>,
    pub status: Option<Status>,
    pub memo: Option<String>,
    pub user_id: Option<i32>,
    pub anonymous: Option<bool>,
    pub dazed_on: Option<Date>,
    pub grotesque: Option<bool>,
}

#[derive(AsChangeset, Debug)]
#[table_name = "puzzle"]
pub struct UpdatePuzzleData {
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

impl From<UpdatePuzzleSet> for UpdatePuzzleData {
    fn from(data: UpdatePuzzleSet) -> Self {
        Self {
            title: data.title,
            yami: data.yami.map(|yami| yami as i32),
            genre: data.yami.map(|genre| genre as i32),
            content: data.content,
            solution: data.solution,
            created: data.created,
            modified: data.modified,
            status: data.status.map(|status| status as i32),
            memo: data.memo,
            user_id: data.user_id,
            anonymous: data.anonymous,
            dazed_on: data.dazed_on,
            grotesque: data.grotesque,
        }
    }
}

#[Object]
impl PuzzleMutation {
    pub async fn update_puzzle(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdatePuzzleSet,
    ) -> async_graphql::Result<Puzzle> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        // User should be the owner on update mutation
        let puzzle_inst: Puzzle = puzzle::table
            .filter(puzzle::id.eq(id))
            .limit(1)
            .first(&conn)?;
        user_id_guard(ctx, puzzle_inst.user_id)?;

        let puzzle: Puzzle = diesel::update(puzzle::table)
            .filter(puzzle::id.eq(id))
            .set(UpdatePuzzleData::from(set))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        CindyBroker::publish(PuzzleSub::Updated(puzzle_inst, puzzle.clone()));

        Ok(puzzle)
    }
}

#[derive(InputObject, Eq, PartialEq, Clone)]
pub struct PuzzleSubFilter {
    status: Option<StatusFiltering>,
    yami: Option<YamiFiltering>,
    genre: Option<GenreFiltering>,
}

impl RawFilter<Puzzle> for PuzzleSubFilter {
    fn check(&self, item: &Puzzle) -> bool {
        if let Some(filter) = self.status.as_ref() {
            filter.check(&item.status)
        } else if let Some(filter) = self.yami.as_ref() {
            filter.check(&item.yami)
        } else if let Some(filter) = self.genre.as_ref() {
            filter.check(&item.genre)
        } else {
            true
        }
    }
}

#[Subscription]
impl PuzzleSubscription {
    pub async fn puzzle_sub(
        &self,
        filter: Option<PuzzleSubFilter>,
    ) -> impl Stream<Item = Option<PuzzleSub>> {
        CindyBroker::<PuzzleSub>::subscribe().filter(move |puzzle_sub| {
            let check = if let Some(filter) = filter.as_ref() {
                match puzzle_sub {
                    Some(PuzzleSub::Created(puzzle)) => filter.check(&puzzle),
                    Some(PuzzleSub::Updated(orig, _)) => filter.check(&orig),
                    None => false,
                }
            } else {
                puzzle_sub.is_some()
            };

            async move { check }
        })
    }
}
