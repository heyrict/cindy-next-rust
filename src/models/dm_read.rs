use async_graphql::{self, Context, InputObject, Object};
use diesel::{prelude::*, query_dsl::QueryDsl, sql_types::Bool};

use super::*;
use crate::context::GlobalCtx;
use crate::schema::dm_read;

/// Available orders for dm_read query
#[derive(InputObject, Clone)]
pub struct DmReadOrder {
    id: Option<Ordering>,
    user_id: Option<Ordering>,
    with_user_id: Option<Ordering>,
    dm_id: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct DmReadOrders(Vec<DmReadOrder>);

impl Default for DmReadOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl DmReadOrders {
    pub fn new(orders: Vec<DmReadOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: dm_read::BoxedQuery<'a, DB>,
    ) -> dm_read::BoxedQuery<'a, DB> {
        use crate::schema::dm_read::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
            gen_order!(obj, user_id, query);
            gen_order!(obj, with_user_id, query);
            gen_order!(obj, dm_id, query);
        }

        query
    }
}

/// Available filters for dm_read query
#[derive(InputObject, Clone)]
pub struct DmReadFilter {
    id: Option<I32Filtering>,
    user_id: Option<I32Filtering>,
}

impl CindyFilter<dm_read::table, DB> for DmReadFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<dm_read::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::dm_read::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<dm_read, DB, SqlType = Bool> + Send>> = None;
        let DmReadFilter {
            id: obj_id,
            user_id: obj_user_id,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);
        filter
    }
}

/// Object for dm_read table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "dm_read"]
pub struct DmRead {
    pub id: ID,
    pub user_id: ID,
    pub with_user_id: ID,
    pub dm_id: ID,
}

#[Object]
impl DmRead {
    async fn id(&self) -> ID {
        self.id
    }
    async fn user_id(&self) -> ID {
        self.user_id
    }
    async fn with_user_id(&self) -> ID {
        self.with_user_id
    }
    async fn dm_id(&self) -> ID {
        self.dm_id
    }

    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        use crate::schema::user;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user = user::table
            .filter(user::id.eq(self.user_id))
            .limit(1)
            .first(&conn)?;

        Ok(user)
    }

    async fn with_user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        use crate::schema::user;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user = user::table
            .filter(user::id.eq(self.with_user_id))
            .limit(1)
            .first(&conn)?;

        Ok(user)
    }

    async fn dm(&self, ctx: &Context<'_>) -> async_graphql::Result<DirectMessage> {
        use crate::schema::direct_message;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let dm = direct_message::table
            .filter(direct_message::id.eq(self.dm_id))
            .limit(1)
            .first(&conn)?;

        Ok(dm)
    }
}
