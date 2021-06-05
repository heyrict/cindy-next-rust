use async_graphql::{self, Context, InputObject, Object};
use diesel::sql_types::Bool;
use diesel::{prelude::*, query_dsl::QueryDsl};

use crate::context::GlobalCtx;
use crate::schema::tag;
use crate::schema_view::tag_aggr;

use super::puzzle_tag::{PuzzleTagFilter, PuzzleTagOrder};
use super::*;

/// Available orders for tag query
#[derive(InputObject, Clone)]
pub struct TagAggrOrder {
    id: Option<Ordering>,
    puzzle_tag_count: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct TagAggrOrders(Vec<TagAggrOrder>);

impl Default for TagAggrOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl TagAggrOrders {
    pub fn new(orders: Vec<TagAggrOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: crate::schema_view::tag_aggr::BoxedQuery<'a, DB>,
    ) -> crate::schema_view::tag_aggr::BoxedQuery<'a, DB> {
        use crate::schema_view::tag_aggr::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
            gen_order!(obj, puzzle_tag_count, query);
        }

        query
    }
}

/// Available filters for tag query
#[derive(InputObject, Clone)]
pub struct TagAggrFilter {
    id: Option<I32Filtering>,
    name: Option<StringFiltering>,
    created: Option<TimestamptzFiltering>,
}

impl CindyFilter<tag_aggr::table, DB> for TagAggrFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<tag_aggr::table, DB, SqlType = Bool> + Send>> {
        use crate::schema_view::tag_aggr::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<tag_aggr, DB, SqlType = Bool> + Send>> =
            None;
        let TagAggrFilter {
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

/// Object for tag table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "tag_aggr"]
pub struct TagAggr {
    pub id: ID,
    pub name: String,
    pub created: Timestamptz,
    pub puzzle_tag_count: i64,
}

impl From<Tag> for TagAggr {
    fn from(item: Tag) -> Self {
        Self {
            id: item.id,
            name: item.name,
            created: item.created,
            puzzle_tag_count: -1,
        }
    }
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

#[Object]
impl TagAggr {
    async fn id(&self) -> ID {
        self.id
    }
    async fn name(&self) -> &str {
        &self.name
    }
    async fn created(&self) -> Timestamptz {
        self.created
    }
    async fn puzzle_tag_count(&self) -> i64 {
        self.puzzle_tag_count
    }
}
