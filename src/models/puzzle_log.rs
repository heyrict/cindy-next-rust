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

#[derive(InputObject, Clone, Debug)]
pub struct PuzzleLogUserIdFiltering {
    eq: Option<i32>,
    is_null: Option<bool>,
}

/// Available filters for puzzle log query
#[derive(InputObject, Clone)]
pub struct PuzzleLogFilter {
    /// ID of the Puzzle to check log with
    puzzle_id: i32,
    /// Whether to check log only related to given user_id
    user_id: Option<PuzzleLogUserIdFiltering>,
    created: Option<TimestamptzFiltering>,
    modified: Option<TimestamptzFiltering>,
}

impl CindyFilter<hint::table> for PuzzleLogFilter {
    fn as_expression(self) -> Option<Box<dyn BoxableExpression<hint::table, DB, SqlType = Bool>>> {
        use crate::schema::hint::dsl::*;

        let PuzzleLogFilter {
            puzzle_id: obj_puzzle_id,
            user_id: obj_user_id,
            created: obj_created,
            modified: obj_modified,
        } = self;

        let mut filter: Option<Box<dyn BoxableExpression<hint, DB, SqlType = Bool>>> =
            Some(if let Some(user_id_filtering) = obj_user_id {
                if let Some(user_id_val) = user_id_filtering.eq {
                    Box::new(
                        puzzle_id.eq(obj_puzzle_id).and(
                            receiver_id
                                .is_null()
                                .or(receiver_id.eq(user_id_val).assume_not_null()),
                        ),
                    )
                } else if let Some(true) = user_id_filtering.is_null {
                    Box::new(puzzle_id.eq(obj_puzzle_id).and(receiver_id.is_null()))
                } else {
                    Box::new(puzzle_id.eq(obj_puzzle_id))
                }
            } else {
                Box::new(puzzle_id.eq(obj_puzzle_id))
            });

        gen_number_filter!(obj_created: TimestamptzFiltering, created, filter);
        gen_number_filter!(obj_modified: TimestamptzFiltering, modified, filter);
        filter
    }
}

impl CindyFilter<dialogue::table> for PuzzleLogFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<dialogue::table, DB, SqlType = Bool>>> {
        use crate::schema::dialogue::dsl::*;

        let PuzzleLogFilter {
            puzzle_id: obj_puzzle_id,
            user_id: obj_user_id,
            created: obj_created,
            modified: obj_modified,
        } = self;

        let mut filter: Option<Box<dyn BoxableExpression<dialogue, DB, SqlType = Bool>>> =
            Some(if let Some(user_id_filtering) = obj_user_id {
                if let Some(user_id_val) = user_id_filtering.eq {
                    Box::new(puzzle_id.eq(obj_puzzle_id).and(user_id.eq(user_id_val)))
                } else {
                    Box::new(puzzle_id.eq(obj_puzzle_id))
                }
            } else {
                Box::new(puzzle_id.eq(obj_puzzle_id))
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

#[derive(Clone)]
pub struct UnsolvedPuzzleStatsSub {
    pub puzzle_id: i32,
    pub dialogue_count: i64,
    pub dialogue_count_answered: i64,
    pub dialogue_max_answered_time: Timestamptz,
}

#[Object]
impl UnsolvedPuzzleStatsSub {
    async fn puzzle_id(&self) -> i32 {
        self.puzzle_id
    }
    async fn dialogue_count(&self) -> i64 {
        self.dialogue_count
    }
    async fn dialogue_count_answered(&self) -> i64 {
        self.dialogue_count_answered
    }
    async fn dialogue_max_answered_time(&self) -> Timestamptz {
        self.dialogue_max_answered_time
    }
}
