use async_graphql::{Context, FieldResult};
use diesel::prelude::*;

use crate::context::GlobalCtx;
use crate::models::{Timestamptz, ID};
use crate::schema::user;

use super::*;

impl QueryRoot {
    pub async fn user_(&self, ctx: &Context<'_>, id: i32) -> FieldResult<User> {
        let conn = ctx.data::<GlobalCtx>().get_conn()?;

        let user = user::table.filter(user::id.eq(id)).limit(1).first(&conn)?;

        Ok(user)
    }

    pub async fn users_(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<UserFilter>>,
        order: Option<Vec<UserOrder>>,
    ) -> FieldResult<Vec<User>> {
        use crate::schema::user::dsl::*;

        let conn = ctx.data::<GlobalCtx>().get_conn()?;

        let mut query = user.into_boxed();
        if let Some(order) = order {
            query = UserOrders::new(order).apply_order(query);
        }
        if let Some(filter) = filter {
            query = UserFilters::new(filter).apply_filter(query);
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

#[async_graphql::InputObject]
#[derive(AsChangeset, Debug)]
#[table_name = "user"]
pub struct UpdateUserSet {
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
    pub last_read_dm_id: Option<Option<i32>>,
    pub icon: Option<Option<String>>,
}

impl MutationRoot {
    pub async fn update_user_(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateUserSet,
    ) -> FieldResult<User> {
        let conn = ctx.data::<GlobalCtx>().get_conn()?;
        diesel::update(user::table)
            .filter(user::id.eq(id))
            .set(&set)
            .get_result(&conn)
            .map_err(|err| err.into())
    }
}