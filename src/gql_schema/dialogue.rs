use async_graphql::{self, guard::Guard, Context, InputObject, Object};
use chrono::Utc;
use diesel::prelude::*;

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::dialogue::*;
use crate::models::*;
use crate::schema::dialogue;

#[derive(Default)]
pub struct DialogueQuery;
#[derive(Default)]
pub struct DialogueMutation;

#[Object]
impl DialogueQuery {
    pub async fn dialogue(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Dialogue> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let dialogue = dialogue::table.filter(dialogue::id.eq(id)).limit(1).first(&conn)?;

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

#[derive(InputObject, AsChangeset, Debug)]
#[table_name = "dialogue"]
pub struct UpdateDialogueInput {
    pub id: Option<ID>,
    pub question: Option<String>,
    pub answer: Option<String>,
    #[column_name = "good"]
    pub is_good: Option<bool>,
    #[column_name = "true"]
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
    pub modified: Option<Timestamptz>,
}

#[derive(InputObject, Insertable)]
#[table_name = "dialogue"]
pub struct CreateDialogueInput {
    pub id: Option<ID>,
    pub question: Option<String>,
    #[graphql(default)]
    pub answer: Option<String>,
    #[column_name = "good"]
    #[graphql(default)]
    pub is_good: Option<bool>,
    #[column_name = "true"]
    #[graphql(default)]
    pub is_true: Option<bool>,
    #[graphql(default_with = "Utc::now()")]
    pub created: Option<Timestamptz>,
    #[column_name = "answeredtime"]
    pub answered_time: Option<Option<Timestamptz>>,
    pub puzzle_id: Option<ID>,
    pub user_id: Option<ID>,
    #[column_name = "answerEditTimes"]
    pub answer_edit_times: Option<i32>,
    #[column_name = "questionEditTimes"]
    #[graphql(default)]
    pub question_edit_times: i32,
    pub qno: Option<i32>,
    #[graphql(default_with = "Utc::now()")]
    pub modified: Timestamptz,
}

impl CreateDialogueInput {
    pub fn set_default(mut self) -> Self {
        let now = Utc::now();
        // Set field `created`
        if self.created.is_none() {
            self.created = Some(now.clone());
        };

        self
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
        use crate::schema::puzzle;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        // User should be the owner on update mutation
        let dialogue_inst: Dialogue = dialogue::table.filter(dialogue::id.eq(id)).limit(1).first(&conn)?;
        let puzzle_inst: Puzzle = puzzle::table
            .filter(puzzle::id.eq(dialogue_inst.puzzle_id))
            .limit(1)
            .first(&conn)?;
        user_id_guard(ctx, puzzle_inst.user_id)?;

        // Set `modified` to the current time when edited
        set.modified = Some(Utc::now());
        set.edit_times = Some(dialogue_inst.edit_times + 1);

        let dialogue: Dialogue = diesel::update(dialogue::table)
            .filter(dialogue::id.eq(id))
            .set(set)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(dialogue)
    }

    pub async fn create_dialogue(
        &self,
        ctx: &Context<'_>,
        data: CreateDialogueInput,
    ) -> async_graphql::Result<Dialogue> {
        use crate::schema::puzzle;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        let insert_data = match role {
            Role::User => {
                // Assert that time-related are unset
                assert_eq_guard(data.created, None)?;
                assert_eq_guard(data.modified, None)?;
                // Assert that upstream puzzle exists
                let puzzle_inst: Puzzle = puzzle::table
                    .filter(puzzle::id.eq(data.puzzle_id))
                    .limit(1)
                    .first(&conn)?;
                // Assert the user is the owner of the puzzle.
                user_id_guard(ctx, puzzle_inst.user_id)?;
                data.set_default()
            }
            Role::Admin => data.set_default(),
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        let dialogue: Dialogue = diesel::insert_into(dialogue::table)
            .values(&insert_data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(dialogue)
    }

    // Delete dialogue (admin only)
    #[graphql(guard(
        DenyRoleGuard(role = "Role::User"),
        DenyRoleGuard(role = "Role::Guest")
    ))]
    pub async fn delete_dialogue(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Dialogue> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let dialogue = diesel::delete(dialogue::table.filter(dialogue::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(dialogue)
    }
}
