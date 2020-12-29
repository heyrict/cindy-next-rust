use async_graphql::{self, InputObject, Object};
use diesel::sql_types::Bool;
use diesel::{prelude::*, query_dsl::QueryDsl};

use crate::schema::tag;

use super::generics::*;

/// Available orders for tag query
#[derive(InputObject, Clone)]
pub struct TagOrder {
    id: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct TagOrders(Vec<TagOrder>);

impl Default for TagOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl TagOrders {
    pub fn new(orders: Vec<TagOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: crate::schema::tag::BoxedQuery<'a, DB>,
    ) -> crate::schema::tag::BoxedQuery<'a, DB> {
        use crate::schema::tag::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
        }

        query
    }
}

/// Available filters for tag query
#[derive(InputObject, Clone)]
pub struct TagFilter {
    id: Option<I32Filtering>,
    name: Option<StringFiltering>,
    created: Option<TimestamptzFiltering>,
}

impl CindyFilter<tag::table, DB> for TagFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<tag::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::tag::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<tag, DB, SqlType = Bool> + Send>> = None;
        let TagFilter {
            id: obj_id,
            name: obj_name,
            created: obj_created,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_string_filter!(obj_name, name, filter);
        gen_number_filter!(obj_created: TimestamptzFiltering, created, filter);

        filter
    }
}

/// Object for tag table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "tag"]
pub struct Tag {
    pub id: ID,
    pub name: String,
    pub created: Timestamptz,
}

#[Object]
impl Tag {
    async fn id(&self) -> ID {
        self.id
    }
    async fn name(&self) -> &str {
        &self.name
    }
    async fn created(&self) -> Timestamptz {
        self.created
    }
}
