use async_graphql::{self, guard::Guard, Context, InputObject, MaybeUndefined, Object};
use chrono::Utc;
use diesel::prelude::*;

use crate::auth::Role;
use crate::broker::CindyBroker;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::{hint::*, puzzle_log::PuzzleLogSub, *};
use crate::schema::hint;

#[derive(Default)]
pub struct HintQuery;
#[derive(Default)]
pub struct HintMutation;

#[Object]
impl HintQuery {
    pub async fn hint(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Hint> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let hint = hint::table.filter(hint::id.eq(id)).limit(1).first(&conn)?;

        Ok(hint)
    }

    pub async fn hints(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<HintFilter>>,
        order: Option<Vec<HintOrder>>,
    ) -> async_graphql::Result<Vec<Hint>> {
        use crate::schema::hint::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = hint.into_boxed();
        if let Some(order) = order {
            query = HintOrders::new(order).apply_order(query);
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

        let hints = query.load::<Hint>(&conn)?;

        Ok(hints)
    }
}

#[derive(InputObject, Debug)]
pub struct UpdateHintInput {
    pub id: Option<ID>,
    pub content: Option<String>,
    pub created: Option<Timestamptz>,
    pub puzzle_id: Option<ID>,
    pub edit_times: Option<i32>,
    pub receiver_id: MaybeUndefined<ID>,
    #[graphql(default_with = "Utc::now()")]
    pub modified: Timestamptz,
}

#[derive(AsChangeset, Debug)]
#[table_name = "hint"]
pub struct UpdateHintData {
    pub id: Option<ID>,
    pub content: Option<String>,
    pub created: Option<Timestamptz>,
    pub puzzle_id: Option<ID>,
    #[column_name = "edittimes"]
    pub edit_times: Option<i32>,
    pub receiver_id: Option<Option<ID>>,
    pub modified: Timestamptz,
}

impl From<UpdateHintInput> for UpdateHintData {
    fn from(x: UpdateHintInput) -> Self {
        Self {
            id: x.id,
            content: x.content,
            created: x.created,
            puzzle_id: x.puzzle_id,
            edit_times: x.edit_times,
            receiver_id: x.receiver_id.as_options(),
            modified: x.modified,
        }
    }
}

#[derive(InputObject)]
pub struct CreateHintInput {
    pub id: Option<ID>,
    #[graphql(default)]
    pub content: String,
    #[graphql(default_with = "Utc::now()")]
    pub created: Timestamptz,
    pub puzzle_id: ID,
    #[graphql(default)]
    pub edit_times: i32,
    #[graphql(default)]
    pub receiver_id: MaybeUndefined<ID>,
    #[graphql(default_with = "Utc::now()")]
    pub modified: Timestamptz,
}

#[derive(Insertable)]
#[table_name = "hint"]
pub struct CreateHintData {
    pub id: Option<ID>,
    pub content: String,
    pub created: Timestamptz,
    pub puzzle_id: ID,
    #[column_name = "edittimes"]
    pub edit_times: i32,
    pub receiver_id: Option<Option<ID>>,
    pub modified: Timestamptz,
}

impl From<CreateHintInput> for CreateHintData {
    fn from(x: CreateHintInput) -> Self {
        Self {
            id: x.id,
            content: x.content,
            created: x.created,
            puzzle_id: x.puzzle_id,
            edit_times: x.edit_times,
            receiver_id: x.receiver_id.as_options(),
            modified: x.modified,
        }
    }
}

#[Object]
impl HintMutation {
    pub async fn update_hint(
        &self,
        ctx: &Context<'_>,
        id: ID,
        mut set: UpdateHintInput,
    ) -> async_graphql::Result<Hint> {
        use crate::schema::puzzle;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        let hint_inst: Hint = hint::table.filter(hint::id.eq(id)).limit(1).first(&conn)?;

        match role {
            Role::User => {
                // User should be the owner on update mutation
                let puzzle_inst: Puzzle = puzzle::table
                    .filter(puzzle::id.eq(hint_inst.puzzle_id))
                    .limit(1)
                    .first(&conn)?;
                user_id_guard(ctx, puzzle_inst.user_id)?;

                // Set `modified` to the current time when edited
                set.edit_times = Some(hint_inst.edit_times + 1);
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        debug!("update_hint: {:?}", &set);
        let data = UpdateHintData::from(set);
        debug!("update_hint: {:?}", &data);

        let hint: Hint = diesel::update(hint::table)
            .filter(hint::id.eq(id))
            .set(data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        let key_starts_with = format!("puzzleLog<{}", hint.puzzle_id);
        CindyBroker::publish(PuzzleLogSub::HintUpdated(hint_inst.clone(), hint.clone()));
        CindyBroker::publish_to_all(
            |key| key.starts_with(&key_starts_with),
            PuzzleLogSub::HintUpdated(hint_inst, hint.clone()),
        );

        Ok(hint)
    }

    pub async fn create_hint(
        &self,
        ctx: &Context<'_>,
        data: CreateHintInput,
    ) -> async_graphql::Result<Hint> {
        use crate::schema::puzzle;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // Assert that upstream puzzle exists
                let puzzle_inst: Puzzle = puzzle::table
                    .filter(puzzle::id.eq(data.puzzle_id))
                    .limit(1)
                    .first(&conn)?;
                // Assert the user is the owner of the puzzle.
                user_id_guard(ctx, puzzle_inst.user_id)?;
            }
            Role::Admin => {}
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        let hint: Hint = diesel::insert_into(hint::table)
            .values(&CreateHintData::from(data))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        let key_starts_with = format!("puzzleLog<{}", hint.puzzle_id);
        CindyBroker::publish(PuzzleLogSub::HintCreated(hint.clone()));
        CindyBroker::publish_to_all(
            |key| key.starts_with(&key_starts_with),
            PuzzleLogSub::HintCreated(hint.clone()),
        );

        Ok(hint)
    }

    // Delete hint (admin only)
    #[graphql(guard(
        DenyRoleGuard(role = "Role::User"),
        DenyRoleGuard(role = "Role::Guest")
    ))]
    pub async fn delete_hint(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Hint> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let hint = diesel::delete(hint::table.filter(hint::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(hint)
    }
}
