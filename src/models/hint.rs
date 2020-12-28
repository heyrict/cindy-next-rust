use async_graphql::{self, Context, InputObject, Object};
use diesel::{
    prelude::*,
    query_dsl::QueryDsl,
    sql_types::{Bool},
};

use crate::context::GlobalCtx;
use crate::schema::hint;

use super::*;

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
        query_dsl: crate::schema::hint::BoxedQuery<'a, DB>,
    ) -> crate::schema::hint::BoxedQuery<'a, DB> {
        use crate::schema::hint::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
            gen_order!(obj, created, query);
            gen_order!(obj, modified, query);
        }

        query
    }
}

/// Available filters for hint query
#[derive(InputObject, Clone)]
pub struct HintFilter {
    id: Option<I32Filtering>,
    content: Option<StringFiltering>,
    created: Option<TimestamptzFiltering>,
    receiver_id: Option<NullableI32Filtering>,
    modified: Option<TimestamptzFiltering>,
}

impl CindyFilter<hint::table, DB> for HintFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<hint::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::hint::dsl::*;

        let mut filter: Option<
            Box<dyn BoxableExpression<hint, DB, SqlType = Bool> + Send>,
        > = None;
        let HintFilter {
            id: obj_id,
            content: obj_content,
            created: obj_created,
            modified: obj_modified,
            receiver_id: obj_receiver_id,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_string_filter!(obj_content, content, filter);
        gen_number_filter!(obj_created: TimestamptzFiltering, created, filter);
        gen_number_filter!(obj_modified: TimestamptzFiltering, modified, filter);
        gen_nullable_number_filter!(obj_receiver_id: NullableI32Filtering, receiver_id, filter);
        filter
    }
}

/// Object for hint table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "hint"]
pub struct Hint {
    pub id: ID,
    pub content: String,
    pub created: Timestamptz,
    pub puzzle_id: ID,
    #[column_name = "edittimes"]
    pub edit_times: i32,
    pub receiver_id: Option<ID>,
    pub modified: Timestamptz,
}

#[Object]
impl Hint {
    pub async fn id(&self) -> ID {
        self.id
    }
    async fn content(&self) -> &str {
        &self.content
    }
    pub async fn created(&self) -> Timestamptz {
        self.created
    }
    pub async fn puzzle_id(&self) -> ID {
        self.puzzle_id
    }
    async fn edit_times(&self) -> i32 {
        self.edit_times
    }
    async fn receiver_id(&self) -> Option<ID> {
        self.receiver_id
    }
    pub async fn modified(&self) -> Timestamptz {
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
