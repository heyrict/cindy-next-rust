use async_graphql::{InputObject, Interface, Object};
use diesel::prelude::*;
use diesel::sql_types::Bool;

use crate::schema::{dialogue, hint};

use super::*;

/// Available orders for puzzle log query
#[derive(InputObject, Clone)]
pub struct PuzzleLogOrder {
    pub id: Option<Ordering>,
    pub created: Option<Ordering>,
    pub modified: Option<Ordering>,
}

/// Available filters for puzzle log query
#[derive(InputObject, Clone)]
pub struct PuzzleLogFilter {
    /// ID of the Puzzle to check log with
    puzzle_id: i32,
    /// Whether to check log only related to given user_id
    user_id: Option<i32>,
    created: Option<TimestamptzFiltering>,
    modified: Option<TimestamptzFiltering>,
}

impl CindyFilter<hint::table, DB> for PuzzleLogFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<hint::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::hint::dsl::*;

        let PuzzleLogFilter {
            puzzle_id: obj_puzzle_id,
            user_id: obj_user_id,
            created: obj_created,
            modified: obj_modified,
        } = self;

        let mut filter: Option<Box<dyn BoxableExpression<hint, DB, SqlType = Bool> + Send>> =
            Some(if let Some(user_id_val) = obj_user_id {
                Box::new(
                    puzzle_id
                        .eq(obj_puzzle_id)
                        .and(receiver_id.is_null().or(receiver_id.eq(user_id_val))),
                )
            } else {
                Box::new(puzzle_id.eq(obj_puzzle_id).and(receiver_id.is_null()))
            });

        gen_number_filter!(obj_created: TimestamptzFiltering, created, filter);
        gen_number_filter!(obj_modified: TimestamptzFiltering, modified, filter);
        filter
    }
}

impl CindyFilter<dialogue::table, DB> for PuzzleLogFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<dialogue::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::dialogue::dsl::*;

        let PuzzleLogFilter {
            puzzle_id: obj_puzzle_id,
            user_id: obj_user_id,
            created: obj_created,
            modified: obj_modified,
        } = self;

        let mut filter: Option<Box<dyn BoxableExpression<dialogue, DB, SqlType = Bool> + Send>> =
            Some(if let Some(user_id_val) = obj_user_id {
                Box::new(
                    puzzle_id
                        .eq(obj_puzzle_id)
                        .and(user_id.is_null().or(user_id.eq(user_id_val))),
                )
            } else {
                Box::new(puzzle_id.eq(obj_puzzle_id).and(user_id.is_null()))
            });

        gen_number_filter!(obj_created: TimestamptzFiltering, created, filter);
        gen_number_filter!(obj_modified: TimestamptzFiltering, modified, filter);
        filter
    }
}

#[derive(Interface, Clone)]
#[graphql(
    field(name = "id", type = "ID"),
    field(name = "puzzle_id", type = "ID"),
    field(name = "created", type = "Timestamptz"),
    field(name = "modified", type = "Timestamptz")
)]
pub enum PuzzleLog {
    Dialogue(Dialogue),
    Hint(Hint),
}

#[derive(Clone)]
pub enum PuzzleLogSub {
    DialogueCreated(Dialogue),
    DialogueUpdated(Dialogue, Dialogue),
    HintCreated(Hint),
    HintUpdated(Hint, Hint),
}

#[Object]
impl PuzzleLogSub {
    async fn op(&self) -> DbOp {
        match &self {
            PuzzleLogSub::DialogueCreated(_) | PuzzleLogSub::HintCreated(_) => DbOp::Created,
            PuzzleLogSub::DialogueUpdated(_, _) | PuzzleLogSub::HintUpdated(_, _) => DbOp::Updated,
        }
    }

    async fn data(&self) -> PuzzleLog {
        match &self {
            PuzzleLogSub::DialogueCreated(obj) => PuzzleLog::Dialogue(obj.clone()),
            PuzzleLogSub::HintCreated(obj) => PuzzleLog::Hint(obj.clone()),
            PuzzleLogSub::DialogueUpdated(_, obj) => PuzzleLog::Dialogue(obj.clone()),
            PuzzleLogSub::HintUpdated(_, obj) => PuzzleLog::Hint(obj.clone()),
        }
    }
}
