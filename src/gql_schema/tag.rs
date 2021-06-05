use async_graphql::{self, guard::Guard, Context, InputObject, Object};
use chrono::Utc;
use diesel::prelude::*;

use crate::auth::Role;
use crate::context::GlobalCtx;
use crate::models::tag::*;
use crate::models::*;
use crate::schema::tag;

#[derive(Default)]
pub struct TagQuery;
#[derive(Default)]
pub struct TagMutation;

#[Object]
impl TagQuery {
    pub async fn tag(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Tag> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let tag = tag::table.filter(tag::id.eq(id)).limit(1).first(&conn)?;

        Ok(tag)
    }

    pub async fn tags(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<TagAggrFilter>>,
        order: Option<Vec<TagAggrOrder>>,
    ) -> async_graphql::Result<Vec<TagAggr>> {
        use crate::schema_view::tag_aggr::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = tag_aggr.into_boxed();
        if let Some(order) = order {
            query = TagAggrOrders::new(order).apply_order(query);
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

        let tags = query.load::<TagAggr>(&conn)?;

        Ok(tags)
    }
}

#[derive(InputObject, AsChangeset, Debug)]
#[table_name = "tag"]
pub struct UpdateTagInput {
    pub id: Option<ID>,
    pub name: Option<String>,
    pub created: Option<Timestamptz>,
}

#[derive(InputObject, Insertable)]
#[table_name = "tag"]
pub struct CreateTagInput {
    pub id: Option<ID>,
    pub name: String,
    #[graphql(default_with = "Utc::now()")]
    pub created: Timestamptz,
}

#[Object]
impl TagMutation {
    #[graphql(guard(and(
        DenyRoleGuard(role = "Role::User"),
        DenyRoleGuard(role = "Role::Guest")
    )))]
    pub async fn update_tag(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateTagInput,
    ) -> async_graphql::Result<Tag> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let tag: Tag = diesel::update(tag::table)
            .filter(tag::id.eq(id))
            .set(set)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(tag)
    }

    #[graphql(guard(DenyRoleGuard(role = "Role::Guest")))]
    pub async fn create_tag(
        &self,
        ctx: &Context<'_>,
        data: CreateTagInput,
    ) -> async_graphql::Result<Tag> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let tag: Tag = diesel::insert_into(tag::table)
            .values(&data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(tag)
    }

    // Delete tag
    #[graphql(guard(and(
        DenyRoleGuard(role = "Role::User"),
        DenyRoleGuard(role = "Role::Guest")
    )))]
    pub async fn delete_tag(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Tag> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let tag = diesel::delete(tag::table.filter(tag::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(tag)
    }
}
