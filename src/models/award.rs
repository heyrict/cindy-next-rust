use async_graphql::{self, Context, InputObject, Object};
use diesel::{prelude::*, query_dsl::QueryDsl, sql_types::Bool};

use super::*;
use crate::schema::award;

use super::user_award::{UserAward, UserAwardFilter, UserAwardOrder};

/// Available orders for award query
#[derive(InputObject, Clone)]
pub struct AwardOrder {
    id: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct AwardOrders(Vec<AwardOrder>);

impl Default for AwardOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl AwardOrders {
    pub fn new(orders: Vec<AwardOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: award::BoxedQuery<'a, DB>,
    ) -> award::BoxedQuery<'a, DB> {
        use crate::schema::award::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
        }

        query
    }
}

/// Available filters for award query
#[derive(InputObject, Clone)]
pub struct AwardFilter {
    id: Option<I32Filtering>,
    name: Option<StringFiltering>,
    description: Option<StringFiltering>,
    group_name: Option<StringFiltering>,
    requisition: Option<StringFiltering>,
}

impl CindyFilter<award::table, DB> for AwardFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<award::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::award::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<award, DB, SqlType = Bool> + Send>> = None;
        let AwardFilter {
            id: obj_id,
            name: obj_name,
            description: obj_description,
            group_name: obj_group_name,
            requisition: obj_requisition,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_string_filter!(obj_name, name, filter);
        gen_string_filter!(obj_description, description, filter);
        gen_string_filter!(obj_group_name, description, filter);
        gen_string_filter!(obj_requisition, description, filter);
        filter
    }
}

/// Object for award table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "award"]
pub struct Award {
    pub id: ID,
    pub name: String,
    pub description: String,
    pub group_name: String,
    pub requisition: String,
}

#[Object]
impl Award {
    async fn id(&self) -> ID {
        self.id
    }
    async fn name(&self) -> &str {
        &self.name
    }
    async fn description(&self) -> &str {
        &self.description
    }
    async fn group_name(&self) -> &str {
        &self.group_name
    }
    async fn requisition(&self) -> &str {
        &self.requisition
    }

    async fn user_awards(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<UserAwardFilter>,
        order: Option<Vec<UserAwardOrder>>,
    ) -> async_graphql::Result<Vec<UserAward>> {
        use crate::gql_schema::UserAwardQuery;

        let filter = filter
            .map(|mut filter| {
                filter.award_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| UserAwardFilter {
                award_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = UserAwardQuery::default();
        query
            .user_awards(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }
}
