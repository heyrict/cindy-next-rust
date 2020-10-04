use async_graphql::{self, Context, InputObject, Object};
use diesel::prelude::*;

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::user::*;
use crate::models::*;
use crate::schema::user;

#[derive(Default)]
pub struct UserQuery;
#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserQuery {
    pub async fn user(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<User> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user = user::table.filter(user::id.eq(id)).limit(1).first(&conn)?;

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

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

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

        let users = query.load::<User>(&conn)?;

        Ok(users)
    }
}

#[derive(InputObject, AsChangeset, Debug)]
#[table_name = "user"]
pub struct UpdateUserSet {
    pub password: Option<String>,
    // TODO use MaybeUndefined
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
    // TODO use MaybeUndefined
    pub current_award_id: Option<Option<i32>>,
    pub hide_bookmark: Option<bool>,
    // TODO use MaybeUndefined
    pub last_read_dm_id: Option<Option<i32>>,
    pub icon: Option<Option<String>>,
}

#[Object]
impl UserMutation {
    pub async fn update_user(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateUserSet,
    ) -> async_graphql::Result<User> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
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
                    &None,
                    "Setting last_login explicitly is prohibited",
                )?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        diesel::update(user::table)
            .filter(user::id.eq(id))
            .set(&set)
            .get_result(&conn)
            .map_err(|err| err.into())
    }
}
