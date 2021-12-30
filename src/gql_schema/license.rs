use async_graphql::{self, Context, InputObject, Object};
use diesel::prelude::*;

use crate::auth::Role;
use crate::context::GlobalCtx;
use crate::models::license::*;
use crate::models::*;
use crate::schema::license;

#[derive(Default)]
pub struct LicenseQuery;
#[derive(Default)]
pub struct LicenseMutation;

#[Object]
impl LicenseQuery {
    pub async fn license(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<License> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let license = license::table
            .filter(license::id.eq(id))
            .limit(1)
            .first(&conn)?;

        Ok(license)
    }

    pub async fn licenses(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<LicenseFilter>>,
        order: Option<Vec<LicenseOrder>>,
    ) -> async_graphql::Result<Vec<License>> {
        use crate::schema::license::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = license.into_boxed();
        if let Some(order) = order {
            query = LicenseOrders::new(order).apply_order(query);
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

        let licenses = query.load::<License>(&conn)?;

        Ok(licenses)
    }
}

#[derive(InputObject, AsChangeset, Debug)]
#[table_name = "license"]
pub struct UpdateLicenseInput {
    pub id: Option<ID>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
}

#[derive(InputObject, Insertable)]
#[table_name = "license"]
pub struct CreateLicenseInput {
    pub id: Option<ID>,
    pub user_id: Option<ID>,
    pub name: String,
    pub url: String,
    pub description: String,
}

#[Object]
impl LicenseMutation {
    // Update license (admin only)
    #[graphql(guard = "DenyRoleGuard::new(Role::User).and(DenyRoleGuard::new(Role::Guest))")]
    pub async fn update_license(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateLicenseInput,
    ) -> async_graphql::Result<License> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let license: License = diesel::update(license::table)
            .filter(license::id.eq(id))
            .set(set)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(license)
    }

    // Create license (admin only)
    #[graphql(guard = "DenyRoleGuard::new(Role::User).and(DenyRoleGuard::new(Role::Guest))")]
    pub async fn create_license(
        &self,
        ctx: &Context<'_>,
        data: CreateLicenseInput,
    ) -> async_graphql::Result<License> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let license: License = diesel::insert_into(license::table)
            .values(&data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(license)
    }

    // Delete license (admin only)
    #[graphql(guard = "DenyRoleGuard::new(Role::User).and(DenyRoleGuard::new(Role::Guest))")]
    pub async fn delete_license(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<License> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let license = diesel::delete(license::table.filter(license::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(license)
    }
}
