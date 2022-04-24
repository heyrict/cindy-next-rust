use async_graphql::{self, Context, InputObject, MaybeUndefined, Object};
use chrono::TimeZone;
use diesel::{
    prelude::*,
    sql_types::{self, Integer},
};

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::user::*;
use crate::models::*;
use crate::schema::user;
use crate::SERVER_TZ;

#[derive(Default)]
pub struct UserQuery;
#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserQuery {
    pub async fn user(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<User> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user = user::table.filter(user::id.eq(id)).limit(1).first(&mut conn)?;

        Ok(user)
    }

    pub async fn users(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<UserFilter>>,
        order: Option<Vec<UserOrder>>,
    ) -> async_graphql::Result<Vec<User>> {
        use crate::schema::user::dsl::*;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = user.into_boxed();
        if let Some(order) = order {
            query = UserOrders::new(order).apply_order(query);
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

        let users = query.load::<User>(&mut conn)?;

        Ok(users)
    }

    pub async fn user_count(
        &self,
        ctx: &Context<'_>,
        filter: Option<Vec<UserFilter>>,
    ) -> async_graphql::Result<i64> {
        use crate::schema::user::dsl::*;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = user.into_boxed();
        if let Some(filter) = filter {
            if let Some(filter_exp) = filter.as_expression() {
                query = query.filter(filter_exp)
            }
        }

        let result = query.count().get_result(&mut conn)?;

        Ok(result)
    }

    pub async fn user_dialogue_ranking(
        &self,
        ctx: &Context<'_>,
        /*#[graphql(validator(IntGreaterThan(value = "1990")))]*/ year: i32,
        /*#[graphql(validator(IntLessThan(value = "13")))]*/ month: u32,
        limit: i32,
        offset: i32,
    ) -> async_graphql::Result<Vec<UserRankingRow>> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        // The range of the time puzzles are created
        let start_time = SERVER_TZ.ymd(year, month, 1).and_hms(0, 0, 0);
        let end_time = if month == 12 {
            SERVER_TZ.ymd(year + 1, 1, 1).and_hms(0, 0, 0)
        } else {
            SERVER_TZ.ymd(year, month + 1, 1).and_hms(0, 0, 0)
        };

        let results: Vec<UserRankingRow> =
            diesel::sql_query(include_str!("../sql/user_dialogue_ranking.sql"))
                .bind::<sql_types::Timestamptz, _>(start_time)
                .bind::<sql_types::Timestamptz, _>(end_time)
                .bind::<Integer, _>(limit)
                .bind::<Integer, _>(offset)
                .get_results(&mut conn)?;

        Ok(results)
    }

    pub async fn user_puzzle_ranking(
        &self,
        ctx: &Context<'_>,
        /*#[graphql(validator(IntGreaterThan(value = "1990")))]*/ year: i32,
        /*#[graphql(validator(IntLessThan(value = "13")))]*/ month: u32,
        limit: i32,
        offset: i32,
    ) -> async_graphql::Result<Vec<UserRankingRow>> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        // The range of the time puzzles are created
        let start_time = SERVER_TZ.ymd(year, month, 1).and_hms(0, 0, 0);
        let end_time = if month == 12 {
            SERVER_TZ.ymd(year + 1, 1, 1).and_hms(0, 0, 0)
        } else {
            SERVER_TZ.ymd(year, month + 1, 1).and_hms(0, 0, 0)
        };

        let results: Vec<UserRankingRow> =
            diesel::sql_query(include_str!("../sql/user_puzzle_ranking.sql"))
                .bind::<sql_types::Timestamptz, _>(start_time)
                .bind::<sql_types::Timestamptz, _>(end_time)
                .bind::<Integer, _>(limit)
                .bind::<Integer, _>(offset)
                .get_results(&mut conn)?;

        Ok(results)
    }
}

#[derive(InputObject, Debug)]
pub struct UpdateUserSet {
    pub password: Option<String>,
    pub last_login: MaybeUndefined<Timestamptz>,
    pub is_superuser: Option<bool>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub is_staff: Option<bool>,
    pub is_active: Option<bool>,
    pub date_joined: Option<Timestamptz>,
    pub nickname: Option<String>,
    pub profile: Option<String>,
    pub current_award_id: MaybeUndefined<i32>,
    pub hide_bookmark: Option<bool>,
    pub last_read_dm_id: MaybeUndefined<i32>,
    pub icon: MaybeUndefined<String>,
    pub default_license_id: MaybeUndefined<ID>,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = user)]
pub struct UpdateUserData {
    pub password: Option<String>,
    pub last_login: Option<Option<Timestamptz>>,
    pub is_superuser: Option<bool>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub is_staff: Option<bool>,
    pub is_active: Option<bool>,
    pub date_joined: Option<Timestamptz>,
    pub nickname: Option<String>,
    pub profile: Option<String>,
    pub current_award_id: Option<Option<i32>>,
    pub hide_bookmark: Option<bool>,
    pub icon: Option<Option<String>>,
    pub default_license_id: Option<Option<ID>>,
}

impl From<UpdateUserSet> for UpdateUserData {
    fn from(x: UpdateUserSet) -> Self {
        Self {
            password: x.password,
            last_login: x.last_login.as_options(),
            is_superuser: x.is_superuser,
            username: x.username,
            first_name: x.first_name,
            last_name: x.last_name,
            email: x.email,
            is_staff: x.is_staff,
            is_active: x.is_active,
            date_joined: x.date_joined,
            nickname: x.nickname,
            profile: x.profile,
            current_award_id: x.current_award_id.as_options(),
            hide_bookmark: x.hide_bookmark,
            icon: x.icon.as_options(),
            default_license_id: x.default_license_id.as_options(),
        }
    }
}

#[Object]
impl UserMutation {
    pub async fn update_user(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateUserSet,
    ) -> async_graphql::Result<User> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // Some fields shouldn't be modified by a user
                assert_eq_guard_msg(
                    &set.password,
                    &None,
                    "Setting password explicitly is prohibited",
                )?;
                assert_eq_guard_msg(
                    &set.date_joined,
                    &None,
                    "Setting date_joined explicitly is prohibited",
                )?;
                assert_eq_guard_msg(
                    &set.last_login,
                    &MaybeUndefined::Undefined,
                    "Setting last_login explicitly is prohibited",
                )?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        diesel::update(user::table)
            .filter(user::id.eq(id))
            .set(&UpdateUserData::from(set))
            .get_result(&mut conn)
            .map_err(|err| err.into())
    }
}
