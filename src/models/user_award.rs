use async_graphql::{self, Context, InputObject, Object};
use diesel::sql_types::Bool;
use diesel::{prelude::*, query_dsl::QueryDsl};

use crate::context::GlobalCtx;
use crate::schema::user_award;

use super::generics::*;
use super::{Award, User};

/// Available orders for user_award query
#[derive(InputObject, Clone)]
pub struct UserAwardOrder {
    id: Option<Ordering>,
    created: Option<Ordering>,
    award_id: Option<Ordering>,
    user_id: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct UserAwardOrders(Vec<UserAwardOrder>);

impl Default for UserAwardOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl UserAwardOrders {
    pub fn new(orders: Vec<UserAwardOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: crate::schema::user_award::BoxedQuery<'a, DB>,
    ) -> crate::schema::user_award::BoxedQuery<'a, DB> {
        use crate::schema::user_award::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
            gen_order!(obj, created, query);
            gen_order!(obj, award_id, query);
            gen_order!(obj, user_id, query);
        }

        query
    }
}

/// Available filters for user_award query
#[derive(InputObject, Clone, Default)]
pub struct UserAwardFilter {
    pub id: Option<I32Filtering>,
    pub created: Option<DateFiltering>,
    pub award_id: Option<I32Filtering>,
    pub user_id: Option<I32Filtering>,
}

impl CindyFilter<user_award::table> for UserAwardFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<user_award::table, DB, SqlType = Bool>>> {
        use crate::schema::user_award::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<user_award, DB, SqlType = Bool>>> = None;
        let UserAwardFilter {
            id: obj_id,
            created: obj_created,
            award_id: obj_award_id,
            user_id: obj_user_id,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_number_filter!(obj_created: DateFiltering, created, filter);
        gen_number_filter!(obj_award_id: I32Filtering, award_id, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);

        filter
    }
}

/// Object for user_award table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[diesel(table_name = user_award)]
pub struct UserAward {
    pub id: ID,
    pub created: Date,
    pub award_id: ID,
    pub user_id: ID,
}

#[Object]
impl UserAward {
    async fn id(&self) -> ID {
        self.id
    }
    async fn created(&self) -> Date {
        self.created
    }
    async fn award_id(&self) -> ID {
        self.award_id
    }
    async fn user_id(&self) -> ID {
        self.user_id
    }

    async fn award(&self, ctx: &Context<'_>) -> async_graphql::Result<Award> {
        use crate::schema::award;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let award = award::table
            .filter(award::id.eq(self.award_id))
            .limit(1)
            .first(&mut conn)?;

        Ok(award)
    }

    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        use crate::schema::user;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user = user::table
            .filter(user::id.eq(self.user_id))
            .limit(1)
            .first(&mut conn)?;

        Ok(user)
    }
}
