use async_graphql::{self, Context, InputObject, Object};
use diesel::sql_types::Bool;
use diesel::{prelude::*, query_dsl::QueryDsl};

use crate::context::GlobalCtx;
use crate::schema::star;

use super::generics::*;
use super::{Puzzle, User};

/// Available orders for star query
#[derive(InputObject, Clone)]
pub struct StarOrder {
    id: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct StarOrders(Vec<StarOrder>);

impl Default for StarOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl StarOrders {
    pub fn new(orders: Vec<StarOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: crate::schema::star::BoxedQuery<'a, DB>,
    ) -> crate::schema::star::BoxedQuery<'a, DB> {
        use crate::schema::star::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
        }

        query
    }
}

/// Available filters for star query
#[derive(InputObject, Clone, Default)]
pub struct StarFilter {
    pub id: Option<I32Filtering>,
    pub value: Option<I16Filtering>,
    pub puzzle_id: Option<I32Filtering>,
    pub user_id: Option<I32Filtering>,
}

impl CindyFilter<star::table, DB> for StarFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<star::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::star::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<star, DB, SqlType = Bool> + Send>> = None;
        let StarFilter {
            id: obj_id,
            value: obj_value,
            puzzle_id: obj_puzzle_id,
            user_id: obj_user_id,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_number_filter!(obj_value: I16Filtering, value, filter);
        gen_number_filter!(obj_puzzle_id: I32Filtering, puzzle_id, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);

        filter
    }
}

/// Object for star table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "star"]
pub struct Star {
    pub id: ID,
    pub value: i16,
    pub puzzle_id: ID,
    pub user_id: ID,
}

#[Object]
impl Star {
    async fn id(&self) -> ID {
        self.id
    }
    async fn value(&self) -> i16 {
        self.value
    }
    async fn puzzle_id(&self) -> ID {
        self.puzzle_id
    }
    async fn user_id(&self) -> ID {
        self.user_id
    }

    async fn puzzle(&self, ctx: &Context<'_>) -> async_graphql::Result<Puzzle> {
        use crate::schema::puzzle;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let puzzle = puzzle::table
            .filter(puzzle::id.eq(self.puzzle_id))
            .limit(1)
            .first(&conn)?;

        Ok(puzzle)
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
}
