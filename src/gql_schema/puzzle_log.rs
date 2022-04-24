use async_graphql::{Context, InputObject, Object, Subscription};
use diesel::prelude::*;
use futures::{Stream, StreamExt};

use crate::broker::CindyBroker;
use crate::context::GlobalCtx;
use crate::models::{dialogue::*, hint::*, puzzle_log::*, *};

#[derive(Default)]
pub struct PuzzleLogQuery;
#[derive(Default)]
pub struct PuzzleLogSubscription;

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

            let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

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

            query.load::<Dialogue>(&mut conn)?
        };

        let hints: Vec<Hint> = {
            use crate::schema::hint::dsl::*;

            let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

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

            query.load::<Hint>(&mut conn)?
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
        }

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

/// Available filters for puzzle log query
#[derive(InputObject, Clone)]
pub struct PuzzleLogSubFilter {
    /// ID of the Puzzle to check log with
    puzzle_id: i32,
    /// Whether to check log only related to given user_id
    user_id: Option<i32>,
}

impl RawFilter<Dialogue> for PuzzleLogSubFilter {
    fn check(&self, item: &Dialogue) -> bool {
        self.user_id.map(|uid| uid == item.user_id).unwrap_or(true)
            && self.puzzle_id == item.puzzle_id
    }
}

impl RawFilter<Hint> for PuzzleLogSubFilter {
    fn check(&self, item: &Hint) -> bool {
        (self.user_id == item.receiver_id || item.receiver_id.is_none())
            && self.puzzle_id == item.puzzle_id
    }
}

#[Subscription]
impl PuzzleLogSubscription {
    pub async fn puzzle_log_sub(
        &self,
        filter: Option<PuzzleLogSubFilter>,
    ) -> impl Stream<Item = Option<PuzzleLogSub>> {
        let key = if let Some(filter) = filter.as_ref() {
            if let Some(user_id) = filter.user_id {
                format!("puzzleLog<{}-{}>", filter.puzzle_id, user_id)
            } else {
                format!("puzzleLog<{}>", filter.puzzle_id)
            }
        } else {
            "puzzleLog".to_string()
        };
        CindyBroker::<PuzzleLogSub>::subscribe_to(key).filter(move |puzzle_log_sub| {
            let check = if let Some(filter) = filter.as_ref() {
                match puzzle_log_sub {
                    Some(PuzzleLogSub::DialogueCreated(obj)) => filter.check(obj),
                    Some(PuzzleLogSub::HintCreated(obj)) => filter.check(obj),
                    Some(PuzzleLogSub::DialogueUpdated(orig, _)) => filter.check(orig),
                    Some(PuzzleLogSub::HintUpdated(orig, _)) => filter.check(orig),
                    None => false,
                }
            } else {
                puzzle_log_sub.is_some()
            };

            async move { check }
        })
    }

    pub async fn unsolved_puzzle_stats_sub(
        &self,
    ) -> impl Stream<Item = Option<UnsolvedPuzzleStatsSub>> {
        CindyBroker::<UnsolvedPuzzleStatsSub>::subscribe()
    }
}
