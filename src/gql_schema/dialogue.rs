use async_graphql::{self, guard::Guard, Context, InputObject, MaybeUndefined, Object};
use chrono::Utc;
use diesel::prelude::*;

use crate::auth::Role;
use crate::broker::CindyBroker;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::{dialogue::*, puzzle_log::PuzzleLogSub, *};
use crate::schema::dialogue;

#[derive(Default)]
pub struct DialogueQuery;
#[derive(Default)]
pub struct DialogueMutation;

#[Object]
impl DialogueQuery {
    pub async fn dialogue(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Dialogue> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let dialogue = dialogue::table
            .filter(dialogue::id.eq(id))
            .limit(1)
            .first(&conn)?;

        Ok(dialogue)
    }

    pub async fn dialogues(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<DialogueFilter>>,
        order: Option<Vec<DialogueOrder>>,
    ) -> async_graphql::Result<Vec<Dialogue>> {
        use crate::schema::dialogue::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = dialogue.into_boxed();
        if let Some(order) = order {
            query = DialogueOrders::new(order).apply_order(query);
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

        let dialogues = query.load::<Dialogue>(&conn)?;

        Ok(dialogues)
    }
}

#[derive(InputObject, Debug)]
pub struct UpdateDialogueInput {
    pub id: Option<ID>,
    pub question: Option<String>,
    pub answer: Option<String>,
    #[graphql(name = "good")]
    pub is_good: Option<bool>,
    #[graphql(name = "true")]
    pub is_true: Option<bool>,
    pub created: Option<Timestamptz>,
    pub answered_time: MaybeUndefined<Timestamptz>,
    pub puzzle_id: Option<ID>,
    pub user_id: Option<ID>,
    pub answer_edit_times: Option<i32>,
    pub question_edit_times: Option<i32>,
    pub qno: Option<i32>,
    #[graphql(default_with = "Utc::now()")]
    pub modified: Timestamptz,
}

#[derive(AsChangeset, Debug)]
#[table_name = "dialogue"]
pub struct UpdateDialogueData {
    pub id: Option<ID>,
    pub question: Option<String>,
    pub answer: Option<String>,
    #[column_name = "good"]
    pub is_good: Option<bool>,
    #[column_name = "true_"]
    pub is_true: Option<bool>,
    pub created: Option<Timestamptz>,
    #[column_name = "answeredtime"]
    pub answered_time: Option<Option<Timestamptz>>,
    pub puzzle_id: Option<ID>,
    pub user_id: Option<ID>,
    #[column_name = "answerEditTimes"]
    pub answer_edit_times: Option<i32>,
    #[column_name = "questionEditTimes"]
    pub question_edit_times: Option<i32>,
    pub qno: Option<i32>,
    pub modified: Timestamptz,
}

impl From<UpdateDialogueInput> for UpdateDialogueData {
    fn from(x: UpdateDialogueInput) -> Self {
        Self {
            id: x.id,
            question: x.question,
            answer: x.answer,
            is_good: x.is_good,
            is_true: x.is_true,
            created: x.created,
            answered_time: x.answered_time.as_options(),
            puzzle_id: x.puzzle_id,
            user_id: x.user_id,
            answer_edit_times: x.answer_edit_times,
            question_edit_times: x.question_edit_times,
            qno: x.qno,
            modified: x.modified,
        }
    }
}

#[derive(InputObject)]
pub struct CreateDialogueInput {
    pub id: Option<ID>,
    pub question: Option<String>,
    #[graphql(default)]
    pub answer: String,
    #[graphql(default, name = "good")]
    pub is_good: bool,
    #[graphql(default, name = "true")]
    pub is_true: bool,
    #[graphql(default_with = "Utc::now()")]
    pub created: Timestamptz,
    pub answered_time: MaybeUndefined<Timestamptz>,
    pub puzzle_id: ID,
    pub user_id: Option<ID>,
    #[graphql(default)]
    pub answer_edit_times: i32,
    #[graphql(default)]
    pub question_edit_times: i32,
    pub qno: Option<i32>,
    #[graphql(default_with = "Utc::now()")]
    pub modified: Timestamptz,
}

#[derive(Insertable)]
#[table_name = "dialogue"]
pub struct CreateDialogueData {
    pub id: Option<ID>,
    pub question: Option<String>,
    pub answer: String,
    #[column_name = "good"]
    pub is_good: bool,
    #[column_name = "true_"]
    pub is_true: bool,
    pub created: Timestamptz,
    #[column_name = "answeredtime"]
    pub answered_time: Option<Option<Timestamptz>>,
    pub puzzle_id: ID,
    pub user_id: Option<ID>,
    #[column_name = "answerEditTimes"]
    pub answer_edit_times: i32,
    #[column_name = "questionEditTimes"]
    pub question_edit_times: i32,
    pub qno: Option<i32>,
    pub modified: Timestamptz,
}

impl From<CreateDialogueInput> for CreateDialogueData {
    fn from(x: CreateDialogueInput) -> Self {
        Self {
            id: x.id,
            question: x.question,
            answer: x.answer,
            is_good: x.is_good,
            is_true: x.is_true,
            created: x.created,
            answered_time: x.answered_time.as_options(),
            puzzle_id: x.puzzle_id,
            user_id: x.user_id,
            answer_edit_times: x.answer_edit_times,
            question_edit_times: x.question_edit_times,
            qno: x.qno,
            modified: x.modified,
        }
    }
}

#[Object]
impl DialogueMutation {
    pub async fn update_dialogue(
        &self,
        ctx: &Context<'_>,
        id: ID,
        mut set: UpdateDialogueInput,
    ) -> async_graphql::Result<Dialogue> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        let dialogue_inst: Dialogue = dialogue::table
            .filter(dialogue::id.eq(id))
            .limit(1)
            .first(&conn)?;

        match role {
            Role::User => {
                assert_eq_guard_msg(set.qno, None, "Setting qno explicitly is prohibited")?;

                // Update edit times
                if set.question.is_some() {
                    set.question_edit_times = Some(dialogue_inst.question_edit_times + 1);
                }
                if set.answer.is_some() {
                    // Update answered time
                    if dialogue_inst.answer.is_empty() && dialogue_inst.answered_time.is_none() {
                        set.answered_time = MaybeUndefined::Value(Utc::now());
                    } else {
                        set.answer_edit_times = Some(dialogue_inst.answer_edit_times + 1);
                    }
                }
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            Role::Admin => {}
        };

        let dialogue: Dialogue = diesel::update(dialogue::table)
            .filter(dialogue::id.eq(id))
            .set(UpdateDialogueData::from(set))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        let key_starts_with = format!("puzzleLog<{}", dialogue.puzzle_id);
        CindyBroker::publish(PuzzleLogSub::DialogueUpdated(
            dialogue_inst.clone(),
            dialogue.clone(),
        ));
        CindyBroker::publish_to_all(
            |key| key.starts_with(&key_starts_with),
            PuzzleLogSub::DialogueUpdated(dialogue_inst, dialogue.clone()),
        );

        Ok(dialogue)
    }

    #[graphql(guard(DenyRoleGuard(role = "Role::Guest")))]
    pub async fn create_dialogue(
        &self,
        ctx: &Context<'_>,
        mut data: CreateDialogueInput,
    ) -> async_graphql::Result<Dialogue> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;

        // Assert user_id is set to the user
        if let Some(user_id) = data.user_id {
            user_id_guard(ctx, user_id)?;
        } else {
            data.user_id = reqctx.get_user_id();
        };

        // Set qno
        let qno: i64 = dialogue::table
            .filter(dialogue::puzzle_id.eq(data.puzzle_id))
            .count()
            .get_result(&conn)?;
        data.qno = Some((qno + 1) as i32);

        let dialogue: Dialogue = diesel::insert_into(dialogue::table)
            .values(&CreateDialogueData::from(data))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        let key_starts_with = format!("puzzleLog<{}", dialogue.puzzle_id);
        CindyBroker::publish(PuzzleLogSub::DialogueCreated(dialogue.clone()));
        CindyBroker::publish_to_all(
            |key| key.starts_with(&key_starts_with),
            PuzzleLogSub::DialogueCreated(dialogue.clone()),
        );

        Ok(dialogue)
    }

    // Delete dialogue (admin only)
    #[graphql(guard(and(
        DenyRoleGuard(role = "Role::User"),
        DenyRoleGuard(role = "Role::Guest")
    )))]
    pub async fn delete_dialogue(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Dialogue> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let dialogue = diesel::delete(dialogue::table.filter(dialogue::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(dialogue)
    }
}
