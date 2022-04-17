use async_graphql::{self, Context, InputObject, Object};
use diesel::prelude::*;

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::favchat::*;
use crate::models::*;
use crate::schema::favorite_chatroom as favchat;

#[derive(Default)]
pub struct FavchatQuery;
#[derive(Default)]
pub struct FavchatMutation;

#[Object]
impl FavchatQuery {
    pub async fn favchat(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Favchat> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let favchat = favchat::table
            .filter(favchat::id.eq(id))
            .limit(1)
            .first(&conn)?;

        Ok(favchat)
    }

    pub async fn favchats(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<FavchatFilter>>,
        order: Option<Vec<FavchatOrder>>,
    ) -> async_graphql::Result<Vec<Favchat>> {
        use crate::schema::favorite_chatroom::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = favorite_chatroom.into_boxed();
        if let Some(order) = order {
            query = FavchatOrders::new(order).apply_order(query);
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

        let favchats = query.load::<Favchat>(&conn)?;

        Ok(favchats)
    }
}

#[derive(InputObject, AsChangeset, Debug)]
#[table_name = "favchat"]
pub struct UpdateFavchatInput {
    pub id: Option<ID>,
    pub chatroom_id: Option<ID>,
    pub user_id: Option<ID>,
}

#[derive(InputObject, Insertable)]
#[table_name = "favchat"]
pub struct CreateFavchatInput {
    pub id: Option<ID>,
    pub chatroom_id: ID,
    pub user_id: Option<ID>,
}

#[Object]
impl FavchatMutation {
    // Update favchat
    pub async fn update_favchat(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateFavchatInput,
    ) -> async_graphql::Result<Favchat> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // User should be the owner on update mutation
                let favchat_inst: Favchat = favchat::table
                    .filter(favchat::id.eq(id))
                    .limit(1)
                    .first(&conn)?;
                user_id_guard(ctx, favchat_inst.user_id)?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let favchat: Favchat = diesel::update(favchat::table)
            .filter(favchat::id.eq(id))
            .set(set)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(favchat)
    }

    // Create favchat
    pub async fn create_favchat(
        &self,
        ctx: &Context<'_>,
        mut data: CreateFavchatInput,
    ) -> async_graphql::Result<Favchat> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
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

        let favchat: Favchat = diesel::insert_into(favchat::table)
            .values(&data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(favchat)
    }

    // Delete favchat
    #[graphql(guard = "DenyRoleGuard::new(Role::Guest)")]
    pub async fn delete_favchat(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Favchat> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // User should be the owner
                let favchat_inst: Favchat = favchat::table
                    .filter(favchat::id.eq(id))
                    .limit(1)
                    .first(&conn)?;
                user_id_guard(ctx, favchat_inst.user_id)?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let favchat = diesel::delete(favchat::table.filter(favchat::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(favchat)
    }
}
