use async_graphql::{self, Context, InputObject, MaybeUndefined, Object, Subscription};
use chrono::{Duration, TimeZone, Utc};
use diesel::{
    prelude::*,
    sql_types::{self, BigInt, Integer},
};
use futures::{Stream, StreamExt};
use regex::Regex;
use std::str::FromStr;

use crate::context::{GlobalCtx, RequestCtx};
use crate::models::puzzle::*;
use crate::models::*;
use crate::schema::puzzle;
use crate::SERVER_TZ;
use crate::{auth::Role, models::image::Image};
use crate::{broker::CindyBroker, models::puzzle_log::PuzzleLogSub};

#[derive(Default)]
pub struct PuzzleQuery;
#[derive(Default)]
pub struct PuzzleMutation;
#[derive(Default)]
pub struct PuzzleSubscription;

lazy_static! {
    static ref UPLOAD_IMAGE_PAT: Regex =
        Regex::new(r#"/images/([a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12})\."#)
            .unwrap();
}
const INVALID_DATETIME: &'static str = "Invalid Datetime";

#[Object]
impl PuzzleQuery {
    pub async fn puzzle(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Puzzle> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let puzzle = puzzle::table
            .filter(puzzle::id.eq(id))
            .limit(1)
            .first(&mut conn)?;

        Ok(puzzle)
    }

    pub async fn puzzles(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<PuzzleFilter>>,
        order: Option<Vec<PuzzleOrder>>,
    ) -> async_graphql::Result<Vec<Puzzle>> {
        use crate::schema::puzzle::dsl::*;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = puzzle.into_boxed();
        if let Some(order) = order {
            query = PuzzleOrders::new(order).apply_order(query);
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

        let puzzles = query.load::<Puzzle>(&mut conn)?;

        Ok(puzzles)
    }

    pub async fn puzzle_count(
        &self,
        ctx: &Context<'_>,
        filter: Option<Vec<PuzzleFilter>>,
    ) -> async_graphql::Result<i64> {
        use crate::schema::puzzle::dsl::*;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = puzzle.into_boxed();
        if let Some(filter) = filter {
            if let Some(filter_exp) = filter.as_expression() {
                query = query.filter(filter_exp)
            }
        }

        let result = query.count().get_result(&mut conn)?;

        Ok(result)
    }

    pub async fn puzzle_count_by_genre(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
    ) -> async_graphql::Result<Vec<PuzzleCountByGenre>> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let results: Vec<PuzzleCountByGenre> =
            diesel::sql_query(include_str!("../sql/puzzle_count_by_genre.sql"))
                .bind::<Integer, _>(user_id)
                .get_results(&mut conn)?;

        Ok(results)
    }

    pub async fn puzzle_star_count_groups(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
    ) -> async_graphql::Result<Vec<PuzzleStarAggrGroup>> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let results: Vec<PuzzleStarAggrGroup> =
            diesel::sql_query(include_str!("../sql/puzzle_star_count_groups.sql"))
                .bind::<Integer, _>(user_id)
                .get_results(&mut conn)?;

        Ok(results)
    }

    pub async fn puzzle_star_sum_groups(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
    ) -> async_graphql::Result<Vec<PuzzleStarAggrGroup>> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let results: Vec<PuzzleStarAggrGroup> =
            diesel::sql_query(include_str!("../sql/puzzle_star_sum_groups.sql"))
                .bind::<Integer, _>(user_id)
                .get_results(&mut conn)?;

        Ok(results)
    }

    pub async fn puzzle_participants(
        &self,
        ctx: &Context<'_>,
        puzzle_id: ID,
    ) -> async_graphql::Result<Vec<PuzzleParticipant>> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let results: Vec<PuzzleParticipant> =
            diesel::sql_query(include_str!("../sql/puzzle_participants.sql"))
                .bind::<Integer, _>(puzzle_id)
                .get_results(&mut conn)?;

        Ok(results)
    }

    pub async fn puzzle_footprints(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        limit: i64,
        offset: i64,
    ) -> async_graphql::Result<Vec<Puzzle>> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let results: Vec<Puzzle> = diesel::sql_query(include_str!("../sql/puzzle_footprints.sql"))
            .bind::<Integer, _>(user_id)
            .bind::<BigInt, _>(limit)
            .bind::<BigInt, _>(offset)
            .get_results(&mut conn)?;

        Ok(results)
    }

    pub async fn puzzle_footprint_count(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
    ) -> async_graphql::Result<i64> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result: PuzzleFootprintCount =
            diesel::sql_query(include_str!("../sql/puzzle_footprint_count.sql"))
                .bind::<Integer, _>(user_id)
                .get_result(&mut conn)?;

        Ok(result.count)
    }

    pub async fn puzzle_star_ranking(
        &self,
        ctx: &Context<'_>,
        /*#[graphql(validator(IntGreaterThan(value = "1990")))]*/ year: i32,
        /*#[graphql(validator(IntLessThan(value = "13")))]*/ month: u32,
        limit: i32,
        offset: i32,
    ) -> async_graphql::Result<Vec<Puzzle>> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        // The range of the time puzzles are created
        let start_time = SERVER_TZ
            .with_ymd_and_hms(year, month, 1, 0, 0, 0)
            .latest()
            .ok_or(INVALID_DATETIME)?;
        let end_time = if month == 12 {
            SERVER_TZ
                .with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0)
                .latest()
                .ok_or(INVALID_DATETIME)
        } else {
            SERVER_TZ
                .with_ymd_and_hms(year, month + 1, 1, 0, 0, 0)
                .latest()
                .ok_or(INVALID_DATETIME)
        }?;

        let results: Vec<Puzzle> =
            diesel::sql_query(include_str!("../sql/puzzle_star_ranking.sql"))
                .bind::<sql_types::Timestamptz, _>(start_time)
                .bind::<sql_types::Timestamptz, _>(end_time)
                .bind::<Integer, _>(limit)
                .bind::<Integer, _>(offset)
                .get_results(&mut conn)?;

        Ok(results)
    }
}

#[derive(InputObject)]
pub struct UpdatePuzzleInput {
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
    pub license_id: MaybeUndefined<ID>,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = puzzle)]
pub struct UpdatePuzzleData {
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
    pub license_id: Option<Option<ID>>,
}

impl From<UpdatePuzzleInput> for UpdatePuzzleData {
    fn from(data: UpdatePuzzleInput) -> Self {
        Self {
            title: data.title,
            yami: data.yami.map(|yami| yami as i32),
            genre: data.genre.map(|genre| genre as i32),
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
            license_id: data.license_id.as_options(),
        }
    }
}

/// Calculate dazing duration of a puzzle
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
pub struct CreatePuzzleInput {
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
    pub license_id: MaybeUndefined<ID>,
}

impl CreatePuzzleInput {
    pub fn set_default(mut self) -> Self {
        let now = Utc::now();
        // Set field `created`
        if self.created.is_none() {
            self.created = Some(now.clone());
        };

        // Set field `dazed_on`
        if self.dazed_on.is_none() {
            self.dazed_on = Some(
                now.date_naive()
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
#[diesel(table_name = puzzle)]
pub struct CreatePuzzleData {
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
    pub license_id: Option<Option<ID>>,
}

impl From<CreatePuzzleInput> for CreatePuzzleData {
    fn from(data: CreatePuzzleInput) -> Self {
        Self {
            title: data.title,
            yami: data.yami.map(|yami| yami as i32),
            genre: data.genre.map(|genre| genre as i32),
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
            license_id: data.license_id.as_options(),
        }
    }
}

#[Object]
impl PuzzleMutation {
    pub async fn update_puzzle(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdatePuzzleInput,
    ) -> async_graphql::Result<Puzzle> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        // User should be the owner on update mutation
        let puzzle_inst: Puzzle = puzzle::table
            .filter(puzzle::id.eq(id))
            .limit(1)
            .first(&mut conn)?;

        match role {
            Role::User => {
                // Assert that time-related are unset
                user_id_guard(ctx, puzzle_inst.user_id)?;

                // Prevent further edit from user if its status is forced hidden
                if let Status::ForceHidden = puzzle_inst.status {
                    return Err(async_graphql::Error::new(
                        "Further edits are blocked from a forced hidden puzzle",
                    ));
                };
            }
            Role::Staff | Role::Admin => {}
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        // Set `modified` to the current time when puzzle is solved
        // TODO separate `modified` from `time_solved`
        //
        // This is redundant as it is already set by postgresql function
        //if puzzle_inst.status == Status::Undergoing && set.status != Some(Status::Undergoing) {
        //    set.modified = Some(Utc::now());
        //};

        // When a puzzle is solved, assign all referred images to that puzzle
        if puzzle_inst.status == Status::Undergoing
            && set.status.is_some()
            && set.status != Some(Status::Undergoing)
        {
            use crate::schema::image;
            for image_id in UPLOAD_IMAGE_PAT.captures_iter(&puzzle_inst.solution) {
                let uuid_str = match uuid::Uuid::from_str(&image_id[1]) {
                    Ok(uuid_str) => uuid_str,
                    Err(_) => {
                        continue;
                    }
                };
                let result = diesel::update(image::table.filter(image::id.eq(uuid_str)))
                    .set(image::puzzle_id.eq(puzzle_inst.id))
                    .execute(&mut conn);
                if let Err(e) = result {
                    info!("{:?}", e);
                    continue;
                }
            }
        }

        // When a puzzle is solved, close all realtime update channels
        let key_starts_with = format!("puzzleLog<{}", puzzle_inst.id);
        CindyBroker::<PuzzleLogSub>::cleaup_all(|key| key.starts_with(&key_starts_with));

        let puzzle: Puzzle = diesel::update(puzzle::table)
            .filter(puzzle::id.eq(id))
            .set(UpdatePuzzleData::from(set))
            .get_result(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        CindyBroker::publish(PuzzleSub::Updated(puzzle_inst, puzzle.clone()));

        Ok(puzzle)
    }

    // Update many puzzle (admin only)
    #[graphql(guard = "DenyRoleGuard::new(Role::User).and(DenyRoleGuard::new(Role::Guest))")]
    pub async fn update_many_puzzle(
        &self,
        ctx: &Context<'_>,
        filter: Option<Vec<PuzzleFilter>>,
        set: UpdatePuzzleInput,
    ) -> async_graphql::Result<Vec<Puzzle>> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let puzzles: Vec<Puzzle> =
            if let Some(filter_exp) = filter.and_then(|filter| filter.as_expression()) {
                diesel::update(puzzle::table)
                    .filter(filter_exp)
                    .set(UpdatePuzzleData::from(set))
                    .get_results(&mut conn)
                    .map_err(|err| async_graphql::Error::from(err))?
            } else {
                diesel::update(puzzle::table)
                    .set(UpdatePuzzleData::from(set))
                    .get_results(&mut conn)
                    .map_err(|err| async_graphql::Error::from(err))?
            };

        // TODO Publish to subscriptions
        Ok(puzzles)
    }

    pub async fn create_puzzle(
        &self,
        ctx: &Context<'_>,
        data: CreatePuzzleInput,
    ) -> async_graphql::Result<Puzzle> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;
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
                    CreatePuzzleData::from(data.set_default())
                } else {
                    CreatePuzzleData::from(data.set_default().set_user_id(user_id))
                };

                insert_data
            }
            Role::Staff | Role::Admin => CreatePuzzleData::from(data.set_default()),
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        let puzzle: Puzzle = diesel::insert_into(puzzle::table)
            .values(&insert_data)
            .get_result(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        // When a puzzle is created, assign all referred images in puzzle content
        use crate::schema::image;
        let concated_string;
        let referring_text = if puzzle.yami == Yami::Longterm {
            concated_string = puzzle.content.clone() + &puzzle.solution;
            &concated_string
        } else {
            &puzzle.content
        };
        for image_id in UPLOAD_IMAGE_PAT.captures_iter(&referring_text) {
            let uuid_str = match uuid::Uuid::from_str(&image_id[1]) {
                Ok(uuid_str) => uuid_str,
                Err(_) => {
                    continue;
                }
            };
            let result = diesel::update(image::table.filter(image::id.eq(uuid_str)))
                .set(image::puzzle_id.eq(puzzle.id))
                .execute(&mut conn);
            if let Err(e) = result {
                info!("{:?}", e);
                continue;
            }
        }

        CindyBroker::publish(PuzzleSub::Created(puzzle.clone()));

        Ok(puzzle)
    }

    // Delete puzzle (admin only)
    #[graphql(guard = "DenyRoleGuard::new(Role::User).and(DenyRoleGuard::new(Role::Guest))")]
    pub async fn delete_puzzle(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Puzzle> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        // When a puzzle is deleted, delete all referred images
        use crate::schema::image;
        let images: Vec<Image> = image::table
            .filter(image::puzzle_id.eq(id))
            .select(image::all_columns)
            .get_results(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;
        for im in images {
            im.delete_file().await?;
        }
        diesel::delete(image::table.filter(image::puzzle_id.eq(id)))
            .execute(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        // Deletes the puzzle instance
        let puzzle = diesel::delete(puzzle::table.filter(puzzle::id.eq(id)))
            .get_result(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(puzzle)
    }
}

#[derive(InputObject, Eq, PartialEq, Clone)]
pub struct PuzzleSubFilter {
    id: Option<I32Filtering>,
    status: Option<StatusFiltering>,
    yami: Option<YamiFiltering>,
    genre: Option<GenreFiltering>,
}

impl RawFilter<Puzzle> for PuzzleSubFilter {
    fn check(&self, item: &Puzzle) -> bool {
        if let Some(filter) = self.id.as_ref() {
            filter.check(&item.id)
        } else if let Some(filter) = self.status.as_ref() {
            filter.check(&item.status)
        } else if let Some(filter) = self.yami.as_ref() {
            filter.check(&item.yami)
        } else if let Some(filter) = self.genre.as_ref() {
            filter.check(&item.genre)
        } else {
            true
        }
    }
}

#[Subscription]
impl PuzzleSubscription {
    pub async fn puzzle_sub(
        &self,
        filter: Option<PuzzleSubFilter>,
    ) -> impl Stream<Item = Option<PuzzleSub>> {
        CindyBroker::<PuzzleSub>::subscribe().filter(move |puzzle_sub| {
            let check = if let Some(filter) = filter.as_ref() {
                match puzzle_sub {
                    Some(PuzzleSub::Created(puzzle)) => filter.check(&puzzle),
                    Some(PuzzleSub::Updated(orig, _)) => filter.check(&orig),
                    None => false,
                }
            } else {
                puzzle_sub.is_some()
            };

            async move { check }
        })
    }
}
