use async_graphql::{self, guard::Guard, Context, InputObject, Object, Subscription};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use futures::{Stream, StreamExt};

use crate::auth::Role;
use crate::broker::CindyBroker;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::hint::*;
use crate::models::*;
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

#[derive(InputObject)]
pub struct UpdateHintInput {
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
#[table_name = "hint"]
pub struct UpdateHintData {
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

impl From<UpdateHintInput> for UpdateHintData {
    fn from(data: UpdateHintInput) -> Self {
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

/// Calculate dazing duration of a hint
#[derive(Default)]
struct DazedTimeCalc {
    yami: Option<Yami>,
    genre: Option<Genre>,
}

impl DazedTimeCalc {
    pub fn yami(mut self, yami: Option<Yami>) -> Self {
        self.yami = yami;
        self
    }
    pub fn genre(mut self, genre: Option<Genre>) -> Self {
        self.genre = genre;
        self
    }
    /// Get dazing duration
    pub fn duration(&self) -> Duration {
        dotenv::dotenv().ok();
        let mut duration = std::env::var("DAZE_DURATION_DEFAULT").unwrap_or("7".to_owned());

        if let Some(genre) = self.genre {
            match genre {
                Genre::Classic => {
                    if let Ok(value) = std::env::var("DAZE_DURATION_GENRE_CLASSIC") {
                        duration = value;
                    }
                }
                Genre::TwentyQuestions => {
                    if let Ok(value) = std::env::var("DAZE_DURATION_GENRE_TWENTY_QUESTIONS") {
                        duration = value;
                    }
                }
                Genre::LittleAlbat => {
                    if let Ok(value) = std::env::var("DAZE_DURATION_GENRE_LITTLE_ALBAT") {
                        duration = value;
                    }
                }
                Genre::Others => {
                    if let Ok(value) = std::env::var("DAZE_DURATION_GENRE_OTHERS") {
                        duration = value;
                    }
                }
            }
        }

        if let Some(yami) = self.yami {
            match yami {
                Yami::None => {
                    if let Ok(value) = std::env::var("DAZE_DURATION_YAMI_NONE") {
                        duration = value;
                    }
                }
                Yami::Normal => {
                    if let Ok(value) = std::env::var("DAZE_DURATION_YAMI_NORMAL") {
                        duration = value;
                    }
                }
                Yami::Longterm => {
                    if let Ok(value) = std::env::var("DAZE_DURATION_YAMI_LONGTERM") {
                        duration = value;
                    }
                }
            }
        }

        Duration::days(
            duration
                .parse::<i64>()
                .expect("Invalid DAZE_DURATION_* variable"),
        )
    }
}

#[derive(InputObject)]
pub struct CreateHintInput {
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

impl CreateHintInput {
    pub fn set_default(mut self) -> Self {
        let now = Utc::now();
        // Set field `created`
        if self.created.is_none() {
            self.created = Some(now.clone());
        };

        // Set field `dazed_on`
        if self.dazed_on.is_none() {
            self.dazed_on = Some(
                now.date().naive_utc()
                    + DazedTimeCalc::default()
                        .yami(self.yami.clone())
                        .genre(self.genre.clone())
                        .duration(),
            );
        };

        // Set field `status`
        if self.status.is_none() {
            self.status = Some(Status::Undergoing);
        };

        self
    }

    pub fn set_user_id(mut self, user_id: Option<i32>) -> Self {
        self.user_id = user_id;
        self
    }
}

#[derive(Insertable)]
#[table_name = "hint"]
pub struct CreateHintData {
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

impl From<CreateHintInput> for CreateHintData {
    fn from(data: CreateHintInput) -> Self {
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
impl HintMutation {
    pub async fn update_hint(
        &self,
        ctx: &Context<'_>,
        id: ID,
        mut set: UpdateHintInput,
    ) -> async_graphql::Result<Hint> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        // User should be the owner on update mutation
        let hint_inst: Hint = hint::table.filter(hint::id.eq(id)).limit(1).first(&conn)?;
        user_id_guard(ctx, hint_inst.user_id)?;

        // Prevent further edit from user if its status is forced hidden
        if let Status::ForceHidden = hint_inst.status {
            return Err(async_graphql::Error::new(
                "Further edits are blocked from a forced hidden hint",
            ));
        };

        // Set `modified` to the current time when hint is solved
        // TODO rename `modified` -> `time_solved`
        if hint_inst.status == Status::Undergoing && set.status != Some(Status::Undergoing) {
            set.modified = Some(Utc::now());
        };

        let hint: Hint = diesel::update(hint::table)
            .filter(hint::id.eq(id))
            .set(UpdateHintData::from(set))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        CindyBroker::publish(HintSub::Updated(hint_inst, hint.clone()));

        Ok(hint)
    }

    pub async fn create_hint(
        &self,
        ctx: &Context<'_>,
        data: CreateHintInput,
    ) -> async_graphql::Result<Hint> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let user_id = reqctx.get_user_id();
        let role = reqctx.get_role();

        let insert_data = match role {
            Role::User => {
                // Assert that time-related are unset
                assert_eq_guard(data.created, None)?;
                assert_eq_guard(data.modified, None)?;
                // Assert user_id is set to the user
                let insert_data = if let Some(user_id) = data.user_id {
                    user_id_guard(ctx, user_id)?;
                    CreateHintData::from(data.set_default())
                } else {
                    CreateHintData::from(data.set_default().set_user_id(user_id))
                };

                insert_data
            }
            Role::Admin => CreateHintData::from(data.set_default()),
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        let hint: Hint = diesel::insert_into(hint::table)
            .values(&insert_data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        CindyBroker::publish(HintSub::Created(hint.clone()));

        Ok(hint)
    }

    // Delete hint (admin only)
    #[graphql(guard(
        DenyRoleGuard(role = "Role::User"),
        DenyRoleGuard(role = "Role::Guest")
    ))]
    pub async fn delete_hint(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Hint> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let user_id = reqctx.get_user_id();

        let hint = diesel::delete(hint::table.filter(hint::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(hint)
    }
}
