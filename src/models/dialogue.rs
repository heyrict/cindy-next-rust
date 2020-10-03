use async_graphql::{self, Context, Object};
use diesel::{prelude::*, query_dsl::QueryDsl};

use crate::context::GlobalCtx;
use crate::schema::dialogue;

use super::generics::*;
use super::puzzle::*;
use super::user::*;

/// Object for dialogue table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "dialogue"]
pub struct Dialogue {
    pub id: ID,
    pub question: String,
    pub answer: String,
    #[column_name = "true"]
    pub is_good: bool,
    #[column_name = "true"]
    pub is_true: bool,
    pub created: Timestamptz,
    #[column_name = "answeredtime"]
    pub answered_time: Option<Timestamptz>,
    pub puzzle_id: ID,
    pub user_id: ID,
    #[column_name = "answerEditTimes"]
    pub answer_edit_times: i32,
    #[column_name = "questionEditTimes"]
    pub question_edit_times: i32,
    pub qno: i32,
    pub modified: Timestamptz,
}

#[Object]
impl Dialogue {
    async fn id(&self) -> ID {
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
    async fn created(&self) -> Timestamptz {
        self.created
    }
    async fn answered_time(&self) -> Option<Timestamptz> {
        self.answered_time
    }
    async fn puzzle_id(&self) -> ID {
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
    async fn modified(&self) -> Timestamptz {
        self.modified
    }

    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        use crate::schema::user;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user_inst = user::table
            .filter(user::id.eq(self.user_id))
            .limit(1)
            .first(&conn)?;

        Ok(user_inst)
    }

    async fn puzzle(&self, ctx: &Context<'_>) -> async_graphql::Result<Puzzle> {
        use crate::schema::puzzle;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let puzzle_inst = puzzle::table
            .filter(puzzle::id.eq(self.puzzle_id))
            .limit(1)
            .first(&conn)?;

        Ok(puzzle_inst)
    }
}
