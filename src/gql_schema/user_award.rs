use async_graphql::{self, Context, InputObject, Object};
use chrono::Utc;
use diesel::prelude::*;

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::user_award::*;
use crate::models::*;
use crate::schema::user_award;

#[derive(Default)]
pub struct UserAwardQuery;
#[derive(Default)]
pub struct UserAwardMutation;

#[Object]
impl UserAwardQuery {
    pub async fn user_award(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<UserAward> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user_award = user_award::table
            .filter(user_award::id.eq(id))
            .limit(1)
            .first(&mut conn)?;

        Ok(user_award)
    }

    pub async fn user_awards(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<UserAwardFilter>>,
        order: Option<Vec<UserAwardOrder>>,
    ) -> async_graphql::Result<Vec<UserAward>> {
        use crate::schema::user_award::dsl::*;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = user_award.into_boxed();
        if let Some(order) = order {
            query = UserAwardOrders::new(order).apply_order(query);
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

        let user_awards = query.load::<UserAward>(&mut conn)?;

        Ok(user_awards)
    }

    pub async fn user_award_count(
        &self,
        ctx: &Context<'_>,
        filter: Option<Vec<UserAwardFilter>>,
    ) -> async_graphql::Result<i64> {
        use crate::schema::user_award::dsl::*;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = user_award.into_boxed();
        if let Some(filter) = filter {
            if let Some(filter_exp) = filter.as_expression() {
                query = query.filter(filter_exp)
            }
        }

        let result = query.count().get_result(&mut conn)?;

        Ok(result)
    }
}

#[derive(InputObject, AsChangeset, Debug)]
#[diesel(table_name = user_award)]
pub struct UpdateUserAwardInput {
    pub id: Option<ID>,
    pub created: Option<Date>,
    pub award_id: Option<ID>,
    pub user_id: Option<ID>,
}

#[derive(InputObject, Insertable)]
#[diesel(table_name = user_award)]
pub struct CreateUserAwardInput {
    pub id: Option<ID>,
    #[graphql(default_with = "Utc::today().naive_utc()")]
    pub created: Date,
    pub award_id: ID,
    pub user_id: Option<ID>,
}

#[Object]
impl UserAwardMutation {
    // Update user_award
    #[graphql(guard = "DenyRoleGuard::new(Role::User).and(DenyRoleGuard::new(Role::Guest))")]
    pub async fn update_user_award(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateUserAwardInput,
    ) -> async_graphql::Result<UserAward> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user_award: UserAward = diesel::update(user_award::table)
            .filter(user_award::id.eq(id))
            .set(set)
            .get_result(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(user_award)
    }

    // Create user_award
    #[graphql(guard = "DenyRoleGuard::new(Role::Guest)")]
    pub async fn create_user_award(
        &self,
        ctx: &Context<'_>,
        mut data: CreateUserAwardInput,
    ) -> async_graphql::Result<UserAward> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let user_id = reqctx.get_user_id();
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // Assert user_id is set to the user
                if let Some(user_id) = data.user_id {
                    user_id_guard(ctx, user_id)?;
                } else {
                    data.user_id = user_id
                };
            }
            Role::Staff | Role::Admin => {}
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        let user_award: UserAward = diesel::insert_into(user_award::table)
            .values(&data)
            .get_result(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(user_award)
    }

    // Delete user_award (admin only)
    #[graphql(guard = "DenyRoleGuard::new(Role::User).and(DenyRoleGuard::new(Role::Guest))")]
    pub async fn delete_user_award(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<UserAward> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user_award = diesel::delete(user_award::table.filter(user_award::id.eq(id)))
            .get_result(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(user_award)
    }
}
