use async_graphql::{self, Context, InputObject, Object};
use diesel::prelude::*;

use crate::auth::Role;
use crate::context::GlobalCtx;
use crate::models::award::*;
use crate::models::*;
use crate::schema::award;

#[derive(Default)]
pub struct AwardQuery;
#[derive(Default)]
pub struct AwardMutation;

#[Object]
impl AwardQuery {
    pub async fn award(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Award> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let award = award::table
            .filter(award::id.eq(id))
            .limit(1)
            .first(&conn)?;

        Ok(award)
    }

    pub async fn awards(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<AwardFilter>>,
        order: Option<Vec<AwardOrder>>,
    ) -> async_graphql::Result<Vec<Award>> {
        use crate::schema::award::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = award.into_boxed();
        if let Some(order) = order {
            query = AwardOrders::new(order).apply_order(query);
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

        let awards = query.load::<Award>(&conn)?;

        Ok(awards)
    }
}

#[derive(InputObject, AsChangeset, Debug)]
#[table_name = "award"]
pub struct UpdateAwardInput {
    pub id: Option<ID>,
    pub name: Option<String>,
    pub description: Option<String>,
    #[column_name = "groupName"]
    pub group_name: Option<String>,
    pub requisition: Option<String>,
}

#[derive(InputObject, Insertable)]
#[table_name = "award"]
pub struct CreateAwardInput {
    pub id: Option<ID>,
    pub name: String,
    pub description: String,
    #[graphql(default_with = "String::new()")]
    #[column_name = "groupName"]
    pub group_name: String,
    #[graphql(default_with = "String::new()")]
    pub requisition: String,
}

#[Object]
impl AwardMutation {
    // Update award (admin only)
    #[graphql(guard = "DenyRoleGuard::new(Role::User).and(DenyRoleGuard::new(Role::Guest))")]
    pub async fn update_award(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateAwardInput,
    ) -> async_graphql::Result<Award> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let award: Award = diesel::update(award::table)
            .filter(award::id.eq(id))
            .set(set)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(award)
    }

    // Create award (admin only)
    #[graphql(guard = "DenyRoleGuard::new(Role::User).and(DenyRoleGuard::new(Role::Guest))")]
    pub async fn create_award(
        &self,
        ctx: &Context<'_>,
        data: CreateAwardInput,
    ) -> async_graphql::Result<Award> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let award: Award = diesel::insert_into(award::table)
            .values(&data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(award)
    }

    // Delete award (admin only)
    #[graphql(guard = "DenyRoleGuard::new(Role::User).and(DenyRoleGuard::new(Role::Guest))")]
    pub async fn delete_award(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Award> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let award = diesel::delete(award::table.filter(award::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(award)
    }
}
