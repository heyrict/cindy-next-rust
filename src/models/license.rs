use async_graphql::{self, Context, InputObject, Object};
use diesel::{prelude::*, query_dsl::QueryDsl, sql_types::Bool};

use crate::context::GlobalCtx;
use crate::schema::license;

use super::*;

/// Available orders for license query
#[derive(InputObject, Clone)]
pub struct LicenseOrder {
    id: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct LicenseOrders(Vec<LicenseOrder>);

impl Default for LicenseOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl LicenseOrders {
    pub fn new(orders: Vec<LicenseOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: license::BoxedQuery<'a, DB>,
    ) -> license::BoxedQuery<'a, DB> {
        use crate::schema::license::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
        }

        query
    }
}

/// Available filters for license query
#[derive(InputObject, Clone)]
pub struct LicenseFilter {
    id: Option<I32Filtering>,
    user_id: Option<NullableI32Filtering>,
    name: Option<StringFiltering>,
    description: Option<StringFiltering>,
}

impl CindyFilter<license::table> for LicenseFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<license::table, DB, SqlType = Bool>>> {
        use crate::schema::license::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<license, DB, SqlType = Bool>>> =
            None;
        let LicenseFilter {
            id: obj_id,
            user_id: obj_user_id,
            name: obj_name,
            description: obj_description,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_nullable_number_filter!(obj_user_id: NullableI32Filtering, user_id, filter);
        gen_string_filter!(obj_name, name, filter);
        gen_string_filter!(obj_description, description, filter);
        filter
    }
}

/// Object for license table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[diesel(table_name = license)]
pub struct License {
    pub id: ID,
    pub user_id: Option<ID>,
    pub name: String,
    pub description: String,
    pub url: Option<String>,
    pub contract: Option<String>,
}

#[Object]
impl License {
    async fn id(&self) -> ID {
        self.id
    }
    async fn user_id(&self) -> Option<ID> {
        self.user_id
    }
    async fn name(&self) -> &str {
        &self.name
    }
    async fn description(&self) -> &str {
        &self.description
    }
    async fn url(&self) -> Option<&String> {
        self.url.as_ref()
    }
    async fn contract(&self) -> Option<&String> {
        self.contract.as_ref()
    }

    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<User>> {
        use crate::schema::user;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        self.user_id
            .map(|user_id| {
                user::table
                    .filter(user::id.eq(user_id))
                    .limit(1)
                    .first(&mut conn)
            })
            .transpose()
            .map_err(|err| async_graphql::ServerError::new(err.to_string(), None).into())
    }
}
