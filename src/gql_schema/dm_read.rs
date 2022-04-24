use async_graphql::{self, Context, InputObject, Object};
use diesel::{
    prelude::*,
    sql_types::{BigInt, Integer},
};

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::dm_read::*;
use crate::models::*;
use crate::schema::dm_read;

#[derive(Default)]
pub struct DmReadQuery;
#[derive(Default)]
pub struct DmReadMutation;

#[Object]
impl DmReadQuery {
    pub async fn dm_read(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<DmRead> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let dm_read = dm_read::table
            .filter(dm_read::id.eq(id))
            .limit(1)
            .first(&mut conn)?;

        Ok(dm_read)
    }

    pub async fn dm_reads(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<DmReadFilter>>,
        order: Option<Vec<DmReadOrder>>,
    ) -> async_graphql::Result<Vec<DmRead>> {
        use crate::schema::dm_read::dsl::*;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = dm_read.into_boxed();
        if let Some(order) = order {
            query = DmReadOrders::new(order).apply_order(query);
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

        let dm_reads = query.load::<DmRead>(&mut conn)?;

        Ok(dm_reads)
    }

    pub async fn dm_read_all(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        limit: i64,
        offset: i64,
    ) -> async_graphql::Result<Vec<DmReadAllEntry>> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let results: Vec<DmReadAllEntry> =
            diesel::sql_query(include_str!("../sql/dm_read_all.sql"))
                .bind::<Integer, _>(user_id)
                .bind::<BigInt, _>(limit)
                .bind::<BigInt, _>(offset)
                .get_results(&mut conn)?;

        Ok(results)
    }
}

#[derive(AsChangeset, InputObject, Debug)]
#[diesel(table_name = dm_read)]
pub struct UpdateDmReadInput {
    pub id: Option<ID>,
    pub user_id: Option<ID>,
    pub with_user_id: Option<ID>,
    pub dm_id: Option<ID>,
}

#[derive(Insertable, InputObject, Debug)]
#[diesel(table_name = dm_read)]
pub struct CreateDmReadInput {
    pub id: Option<ID>,
    pub user_id: Option<ID>,
    pub with_user_id: ID,
    pub dm_id: ID,
}

#[derive(InputObject, Debug)]
pub struct UpsertDmReadInput {
    pub user_id: ID,
    pub with_user_id: ID,
    pub dm_id: ID,
}

impl From<UpsertDmReadInput> for UpdateDmReadInput {
    fn from(x: UpsertDmReadInput) -> Self {
        Self {
            id: None,
            user_id: None,
            with_user_id: None,
            dm_id: Some(x.dm_id),
        }
    }
}

impl From<UpsertDmReadInput> for CreateDmReadInput {
    fn from(x: UpsertDmReadInput) -> Self {
        Self {
            id: None,
            user_id: Some(x.user_id),
            with_user_id: x.with_user_id,
            dm_id: x.dm_id,
        }
    }
}

#[Object]
impl DmReadMutation {
    // Update dm_read
    #[graphql(guard = "DenyRoleGuard::new(Role::Guest)")]
    pub async fn update_dm_read(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateDmReadInput,
    ) -> async_graphql::Result<DmRead> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // User should be the owner on update mutation
                let dm_read_inst: DmRead = dm_read::table
                    .filter(dm_read::id.eq(id))
                    .limit(1)
                    .first(&mut conn)?;
                user_id_guard(ctx, dm_read_inst.user_id)?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let dm_read: DmRead = diesel::update(dm_read::table)
            .filter(dm_read::id.eq(id))
            .set(set)
            .get_result(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(dm_read)
    }

    // Create dm_read
    #[graphql(guard = "DenyRoleGuard::new(Role::Guest)")]
    pub async fn create_dm_read(
        &self,
        ctx: &Context<'_>,
        mut data: CreateDmReadInput,
    ) -> async_graphql::Result<DmRead> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // Assert the user is the owner of the puzzle.
                if let Some(user_id) = data.user_id {
                    user_id_guard(ctx, user_id)?;
                } else {
                    data.user_id = reqctx.get_user_id();
                };
            }
            Role::Staff | Role::Admin => {}
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        let dm_read: DmRead = diesel::insert_into(dm_read::table)
            .values(&data)
            .get_result(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(dm_read)
    }

    // Upsert dm_read
    #[graphql(guard = "DenyRoleGuard::new(Role::Guest)")]
    pub async fn upsert_dm_read(
        &self,
        ctx: &Context<'_>,
        data: UpsertDmReadInput,
    ) -> async_graphql::Result<DmRead> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = dm_read::table
            .filter(
                dm_read::user_id
                    .eq(data.user_id)
                    .and(dm_read::with_user_id.eq(data.with_user_id)),
            )
            .limit(1)
            .first::<DmRead>(&mut conn);

        if let Ok(dm_read_inst) = result {
            // Update the object
            DmReadMutation::default()
                .update_dm_read(&ctx, dm_read_inst.id, data.into())
                .await
        } else {
            // Insert an object
            DmReadMutation::default()
                .create_dm_read(&ctx, data.into())
                .await
        }
    }

    // Delete dm_read (admin only)
    #[graphql(guard = "DenyRoleGuard::new(Role::User).and(DenyRoleGuard::new(Role::Guest))")]
    pub async fn delete_dm_read(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<DmRead> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let dm_read = diesel::delete(dm_read::table.filter(dm_read::id.eq(id)))
            .get_result(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(dm_read)
    }
}
