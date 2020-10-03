use async_graphql::{self, Context, InputObject, Object, Subscription, Union};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::sql_types::Bool;
use futures::{Stream, StreamExt};

use crate::auth::Role;
use crate::broker::CindyBroker;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::{dialogue::*, hint::*, *};
use crate::schema::{dialogue, hint};

#[derive(Default)]
pub struct PuzzleLogQuery;
#[derive(Default)]
pub struct PuzzleLogSubscription;

#[derive(Union)]
pub enum PuzzleLog {
    Dialogue(Dialogue),
    Hint(Hint),
}

/// Available orders for puzzle log query
#[derive(InputObject, Clone)]
pub struct PuzzleLogOrder {
    id: Option<Ordering>,
    created: Option<Ordering>,
    modified: Option<Ordering>,
}

/// Available filters for puzzle log query
#[derive(InputObject, Clone)]
pub struct PuzzleLogFilter {
    /// ID of the Puzzle to check log with
    puzzle_id: i32,
    /// Whether to check log only related to given user_id
    user_id: Option<i32>,
}

impl CindyFilter<hint::table, DB> for PuzzleLogFilter {
    fn as_expression(self) -> Option<Box<dyn BoxableExpression<hint::table, DB, SqlType = Bool>>> {
        use crate::schema::hint::dsl::*;
        Some(if let Some(user_id_val) = self.user_id {
            Box::new(
                puzzle_id
                    .eq(self.puzzle_id)
                    .and(receiver_id.is_null().or(receiver_id.eq(user_id_val))),
            )
        } else {
            Box::new(puzzle_id.eq(self.puzzle_id).and(receiver_id.is_null()))
        })
    }
}

impl CindyFilter<dialogue::table, DB> for PuzzleLogFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<dialogue::table, DB, SqlType = Bool>>> {
        use crate::schema::dialogue::dsl::*;
        Some(if let Some(user_id_val) = self.user_id {
            Box::new(puzzle_id.eq(self.puzzle_id).and(user_id.eq(user_id_val)))
        } else {
            Box::new(puzzle_id.eq(self.puzzle_id))
        })
    }
}

#[Object]
impl PuzzleLogQuery {
    pub async fn puzzle_logs(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: PuzzleLogFilter,
        order: PuzzleLogOrder,
    ) -> async_graphql::Result<Vec<PuzzleLog>> {
        let dialogues: Vec<Dialogue> = {
            use crate::schema::dialogue::dsl::*;

            let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

            let mut query = dialogue.into_boxed();
            gen_order!(order, id, query);
            gen_order!(order, created, query);
            gen_order!(order, modified, query);

            if let Some(filter_exp) = filter.clone().as_expression() {
                query = query.filter(filter_exp)
            }
            if let Some(limit) = limit {
                query = query.limit(limit);
            }
            if let Some(offset) = offset {
                query = query.offset(offset);
            }

            query.load::<Dialogue>(&conn)?
        };

        let hints: Vec<Hint> = {
            use crate::schema::hint::dsl::*;

            let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

            let mut query = hint.into_boxed();
            gen_order!(order, id, query);
            gen_order!(order, created, query);
            gen_order!(order, modified, query);

            if let Some(filter_exp) = filter.as_expression() {
                query = query.filter(filter_exp)
            }
            if let Some(limit) = limit {
                query = query.limit(limit);
            }
            if let Some(offset) = offset {
                query = query.offset(offset);
            }

            query.load::<Hint>(&conn)?
        };

        let mut puzzle_logs: Vec<PuzzleLog> = dialogues
            .into_iter()
            .map(|dlg| PuzzleLog::Dialogue(dlg))
            .chain(hints.into_iter().map(|hint| PuzzleLog::Hint(hint)))
            .collect();

        macro_rules! sort_fn {
            ($key:ident) => {
                |a, b| {
                    let a_value = match a {
                        PuzzleLog::Hint(hint) => hint.$key,
                        PuzzleLog::Dialogue(dialogue) => dialogue.$key,
                    };
                    let b_value = match b {
                        PuzzleLog::Hint(hint) => hint.$key,
                        PuzzleLog::Dialogue(dialogue) => dialogue.$key,
                    };
                    a_value.partial_cmp(&b_value).unwrap()
                }
            };
            (-$key:ident) => {
                |a, b| {
                    let a_value = match a {
                        PuzzleLog::Hint(hint) => hint.$key,
                        PuzzleLog::Dialogue(dialogue) => dialogue.$key,
                    };
                    let b_value = match b {
                        PuzzleLog::Hint(hint) => hint.$key,
                        PuzzleLog::Dialogue(dialogue) => dialogue.$key,
                    };
                    b_value.partial_cmp(&a_value).unwrap()
                }
            };
        };

        if let Some(ordering) = order.id {
            match ordering {
                Ordering::Asc | Ordering::AscNullsLast | Ordering::AscNullsFirst => {
                    puzzle_logs.sort_unstable_by(sort_fn!(id))
                }
                _ => puzzle_logs.sort_unstable_by(sort_fn!(-id)),
            }
        } else if let Some(ordering) = order.created {
            match ordering {
                Ordering::Asc | Ordering::AscNullsLast | Ordering::AscNullsFirst => {
                    puzzle_logs.sort_unstable_by(sort_fn!(created))
                }
                _ => puzzle_logs.sort_unstable_by(sort_fn!(-created)),
            }
        } else if let Some(ordering) = order.modified {
            match ordering {
                Ordering::Asc | Ordering::AscNullsLast | Ordering::AscNullsFirst => {
                    puzzle_logs.sort_unstable_by(sort_fn!(modified))
                }
                _ => puzzle_logs.sort_unstable_by(sort_fn!(-modified)),
            }
        };

        Ok(puzzle_logs)
    }
}
