use async_graphql::{self, Context, InputObject, Object};
use diesel::{dsl::sum, prelude::*};

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::star::*;
use crate::models::*;
use crate::schema::star;

#[derive(Default)]
pub struct StarQuery;
#[derive(Default)]
pub struct StarMutation;

#[Object]
impl StarQuery {
    pub async fn star(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Star> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let star = star::table.filter(star::id.eq(id)).limit(1).first(&mut conn)?;

        Ok(star)
    }

    pub async fn stars(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<StarFilter>>,
        order: Option<Vec<StarOrder>>,
    ) -> async_graphql::Result<Vec<Star>> {
        use crate::schema::star::dsl::*;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = star.into_boxed();
        if let Some(order) = order {
            query = StarOrders::new(order).apply_order(query);
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

        let stars = query.load::<Star>(&mut conn)?;

        Ok(stars)
    }

    pub async fn star_count(
        &self,
        ctx: &Context<'_>,
        filter: Option<StarCountFilter>,
    ) -> async_graphql::Result<i64> {
        use crate::schema::star::dsl::*;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = star.into_boxed();
        if let Some(filter) = filter {
            if let Some(filter_exp) = filter.as_expression() {
                query = query.filter(filter_exp)
            }
        }

        let result = query.count().get_result::<i64>(&mut conn)?;

        Ok(result)
    }

    pub async fn star_sum_by_puzzle(
        &self,
        ctx: &Context<'_>,
        puzzle_id: ID,
    ) -> async_graphql::Result<Option<i64>> {
        use crate::schema::{puzzle, star};

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = star::table
            .inner_join(puzzle::table)
            .filter(puzzle::id.eq(puzzle_id))
            .select(sum(star::value))
            .get_result(&mut conn)?;

        Ok(result)
    }
}

#[derive(InputObject, AsChangeset, Debug)]
#[diesel(table_name = star)]
pub struct UpdateStarInput {
    pub id: Option<ID>,
    pub value: Option<i16>,
    pub puzzle_id: Option<ID>,
    pub user_id: Option<ID>,
}

#[derive(InputObject, Insertable)]
#[diesel(table_name = star)]
pub struct CreateStarInput {
    pub id: Option<ID>,
    pub value: i16,
    pub puzzle_id: ID,
    pub user_id: Option<ID>,
}

#[Object]
impl StarMutation {
    pub async fn update_star(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateStarInput,
    ) -> async_graphql::Result<Star> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // User should be the owner on update mutation
                let star_inst: Star = star::table.filter(star::id.eq(id)).limit(1).first(&mut conn)?;
                user_id_guard(ctx, star_inst.user_id)?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let star: Star = diesel::update(star::table)
            .filter(star::id.eq(id))
            .set(set)
            .get_result(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(star)
    }

    pub async fn create_star(
        &self,
        ctx: &Context<'_>,
        mut data: CreateStarInput,
    ) -> async_graphql::Result<Star> {
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

        let star: Star = diesel::insert_into(star::table)
            .values(&data)
            .get_result(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(star)
    }

    // Delete star
    #[graphql(guard = "DenyRoleGuard::new(Role::Guest)")]
    pub async fn delete_star(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Star> {
        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // User should be the owner
                let star_inst: Star = star::table.filter(star::id.eq(id)).limit(1).first(&mut conn)?;
                user_id_guard(ctx, star_inst.user_id)?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let star = diesel::delete(star::table.filter(star::id.eq(id)))
            .get_result(&mut conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(star)
    }
}
