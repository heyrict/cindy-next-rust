use async_graphql::{self, Context, InputObject, Object};
use diesel::{prelude::*, query_dsl::QueryDsl};

use crate::context::GlobalCtx;
use crate::schema::hint;

use super::generics::*;
use super::puzzle::*;
use super::user::*;

/// Available orders for hint query
#[derive(InputObject, Clone)]
pub struct HintOrder {
    id: Option<Ordering>,
    created: Option<Ordering>,
    modified: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct HintOrders(Vec<HintOrder>);

impl Default for HintOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl HintOrders {
    pub fn new(orders: Vec<HintOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: crate::schema::puzzle::BoxedQuery<'a, DB>,
    ) -> crate::schema::puzzle::BoxedQuery<'a, DB> {
        use crate::schema::puzzle::dsl::*;

        let mut query = query_dsl;
        let mut flag = false;

        for obj in self.0 {
            gen_order!(obj, id, query, flag);
            gen_order!(obj, created, query, flag);
            gen_order!(obj, modified, query, flag);
        }

        query
    }
}

/// Object for dialogue table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "hint"]
pub struct Hint {
    pub id: ID,
    pub content: String,
    pub created: Timestamptz,
    pub puzzle_id: ID,
    pub edittimes: i32,
    pub receiver_id: Option<ID>,
    pub modified: Timestamptz,
}

#[Object]
impl Hint {
    async fn id(&self) -> ID {
        self.id
    }
    async fn content(&self) -> &str {
        &self.content
    }
    async fn created(&self) -> Timestamptz {
        self.created
    }
    async fn puzzle_id(&self) -> ID {
        self.puzzle_id
    }
    async fn edittimes(&self) -> i32 {
        self.edittimes
    }
    async fn receiver_id(&self) -> Option<ID> {
        self.receiver_id
    }
    async fn modified(&self) -> Timestamptz {
        self.modified
    }

    async fn receiver(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<User>> {
        if let Some(receiver_id) = self.receiver_id {
            use crate::schema::user;

            let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

            let user_inst = user::table
                .filter(user::id.eq(receiver_id))
                .limit(1)
                .first(&conn)?;

            Ok(Some(user_inst))
        } else {
            Ok(None)
        }
    }

    async fn puzzle(&self, ctx: &Context<'_>) -> async_graphql::Result<Puzzle> {
        use crate::schema::puzzle;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let puzzle_inst = puzzle::table
            .filter(puzzle::id.eq(self.puzzle_id))
            .limit(1)
            .first(&conn)?;

        Ok(puzzle_inst)
    }
}
