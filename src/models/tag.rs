use async_graphql::{self, Context, InputObject, Object};
use diesel::sql_types::Bool;
use diesel::{prelude::*, query_dsl::QueryDsl};

use crate::context::GlobalCtx;
use crate::schema::tag;

use super::puzzle_tag::{PuzzleTagFilter, PuzzleTagOrder};
use super::*;

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

    async fn puzzle_tags(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<PuzzleTagFilter>,
        order: Option<Vec<PuzzleTagOrder>>,
    ) -> async_graphql::Result<Vec<PuzzleTag>> {
        use crate::gql_schema::PuzzleTagQuery;

        let filter = filter
            .map(|mut filter| {
                filter.tag_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| PuzzleTagFilter {
                tag_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = PuzzleTagQuery::default();
        query
            .puzzle_tags(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }

    async fn puzzle_tag_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::puzzle_tag::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = puzzle_tag
            .filter(tag_id.eq(self.id))
            .count()
            .get_result::<i64>(&conn)?;

        Ok(result)
    }
}
