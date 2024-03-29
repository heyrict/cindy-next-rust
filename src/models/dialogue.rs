use async_graphql::{self, Context, InputObject, Object};
use diesel::sql_types::Bool;
use diesel::{
    prelude::*,
    query_dsl::QueryDsl,
    sql_types::{BigInt, Int4},
};

use crate::context::GlobalCtx;
use crate::schema::dialogue;

use super::*;

/// Available orders for dialogue query
#[derive(InputObject, Clone)]
pub struct DialogueOrder {
    id: Option<Ordering>,
    created: Option<Ordering>,
    answered_time: Option<Ordering>,
    modified: Option<Ordering>,
    puzzle_id: Option<Ordering>,
    user_id: Option<Ordering>,
    qno: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct DialogueOrders(Vec<DialogueOrder>);

impl Default for DialogueOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl DialogueOrders {
    pub fn new(orders: Vec<DialogueOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: crate::schema::dialogue::BoxedQuery<'a, DB>,
    ) -> crate::schema::dialogue::BoxedQuery<'a, DB> {
        use crate::schema::dialogue::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
            gen_order!(obj, created, query);
            gen_order!(obj, modified, query);
        }

        query
    }
}

/// Available filters for dialogue query
#[derive(InputObject, Clone, Default)]
pub struct DialogueFilter {
    pub id: Option<I32Filtering>,
    pub question: Option<StringFiltering>,
    pub answer: Option<StringFiltering>,
    #[graphql(name = "good")]
    pub is_good: Option<bool>,
    #[graphql(name = "true")]
    pub is_true: Option<bool>,
    pub created: Option<TimestamptzFiltering>,
    pub answered_time: Option<NullableTimestamptzFiltering>,
    pub modified: Option<TimestamptzFiltering>,
    pub puzzle_id: Option<I32Filtering>,
    pub user_id: Option<I32Filtering>,
}

impl CindyFilter<dialogue::table> for DialogueFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<dialogue::table, DB, SqlType = Bool>>> {
        use crate::schema::dialogue::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<dialogue, DB, SqlType = Bool>>> = None;
        let DialogueFilter {
            id: obj_id,
            question: obj_question,
            answer: obj_answer,
            is_good: obj_is_good,
            is_true: obj_is_true,
            created: obj_created,
            answered_time: obj_answered_time,
            modified: obj_modified,
            puzzle_id: obj_puzzle_id,
            user_id: obj_user_id,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_number_filter!(obj_puzzle_id: I32Filtering, puzzle_id, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);
        gen_string_filter!(obj_question, question, filter);
        gen_string_filter!(obj_answer, answer, filter);
        gen_bool_filter!(obj_is_good, good, filter);
        gen_bool_filter!(obj_is_true, true_, filter);
        gen_number_filter!(obj_created: TimestamptzFiltering, created, filter);
        gen_number_filter!(obj_modified: TimestamptzFiltering, modified, filter);
        gen_nullable_number_filter!(
            obj_answered_time: NullableTimestamptzFiltering,
            answeredtime,
            filter
        );
        filter
    }
}

#[derive(QueryableByName, Clone, Debug)]
pub struct UserMaxYamiDialogueCountResult {
    /// Puzzle ID
    #[diesel(sql_type = Int4)]
    pub id: ID,
    #[diesel(sql_type = BigInt)]
    pub dialogue_count: i64,
}

#[Object]
impl UserMaxYamiDialogueCountResult {
    async fn id(&self) -> ID {
        self.id
    }
    async fn dialogue_count(&self) -> i64 {
        self.dialogue_count
    }

    async fn puzzle(&self, ctx: &Context<'_>) -> async_graphql::Result<Puzzle> {
        use crate::schema::puzzle;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let puzzle = puzzle::table
            .filter(puzzle::id.eq(self.id))
            .limit(1)
            .first(&mut conn)?;

        Ok(puzzle)
    }
}

/// Object for dialogue table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[diesel(table_name = dialogue)]
pub struct Dialogue {
    pub id: ID,
    pub question: String,
    pub answer: String,
    #[diesel(column_name = good)]
    pub is_good: bool,
    #[diesel(column_name = true_)]
    pub is_true: bool,
    pub created: Timestamptz,
    #[diesel(column_name = answeredtime)]
    pub answered_time: Option<Timestamptz>,
    pub puzzle_id: ID,
    pub user_id: ID,
    #[diesel(column_name = answerEditTimes)]
    pub answer_edit_times: i32,
    #[diesel(column_name = questionEditTimes)]
    pub question_edit_times: i32,
    pub qno: i32,
    pub modified: Timestamptz,
}

#[Object]
impl Dialogue {
    pub async fn id(&self) -> ID {
        self.id
    }
    async fn question(&self) -> &str {
        &self.question
    }
    async fn answer(&self) -> &str {
        &self.answer
    }
    #[graphql(name = "good")]
    async fn is_good(&self) -> bool {
        self.is_good
    }
    #[graphql(name = "true")]
    async fn is_true(&self) -> bool {
        self.is_true
    }
    pub async fn created(&self) -> Timestamptz {
        self.created
    }
    async fn answered_time(&self) -> Option<Timestamptz> {
        self.answered_time
    }
    pub async fn puzzle_id(&self) -> ID {
        self.puzzle_id
    }
    async fn user_id(&self) -> ID {
        self.user_id
    }
    async fn answer_edit_times(&self) -> i32 {
        self.answer_edit_times
    }
    async fn question_edit_times(&self) -> i32 {
        self.question_edit_times
    }
    async fn qno(&self) -> i32 {
        self.qno
    }
    pub async fn modified(&self) -> Timestamptz {
        self.modified
    }

    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        use crate::schema::user;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user_inst = user::table
            .filter(user::id.eq(self.user_id))
            .limit(1)
            .first(&mut conn)?;

        Ok(user_inst)
    }

    pub async fn puzzle(&self, ctx: &Context<'_>) -> async_graphql::Result<Puzzle> {
        use crate::schema::puzzle;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let puzzle_inst = puzzle::table
            .filter(puzzle::id.eq(self.puzzle_id))
            .limit(1)
            .first(&mut conn)?;

        Ok(puzzle_inst)
    }
}
