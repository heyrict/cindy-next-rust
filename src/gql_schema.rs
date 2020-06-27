use async_graphql::{Context, FieldResult, Schema, SimpleBroker, ID};
use diesel::prelude::*;
use futures::lock::Mutex;
use futures::{Stream, StreamExt};
use std::sync::Arc;
use std::time::Duration;

use crate::context::CindyContext;
use crate::models::*;

pub type CindySchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub struct QueryRoot;

/*
macro_rules! gen_list_query {
    ($filter: ident, $order: ident, $limit:ident, $offset:ident) => {

    };
}
*/

#[async_graphql::Object]
impl QueryRoot {
    async fn users(
        &self,
        ctx: &Context<'_>,
        limit: i64,
        offset: i64,
        order: Vec<UserOrder>,
    ) -> FieldResult<Vec<User>> {
        use crate::schema::user::dsl::*;

        let conn = ctx.data::<CindyContext>().get_conn()?;

        let mut query = UserOrders::new(order).apply_order(user.into_boxed());
        query = query.limit(limit).offset(offset);

        let users = query.load::<User>(&conn)?;

        Ok(users)
    }
}

pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    /*
    async fn create_book(&self, ctx: &Context<'_>, name: String, author: String) -> ID {
        let mut books = ctx.data::<Storage>().lock().await;
        let entry = books.vacant_entry();
        let id: ID = entry.key().into();
        let book = Book {
            id: id.clone(),
            name,
            author,
        };
        entry.insert(book);
        SimpleBroker::publish(BookChanged {
            mutation_type: MutationType::Created,
            id: id.clone(),
        });
        id
    }

    async fn delete_book(&self, ctx: &Context<'_>, id: ID) -> FieldResult<bool> {
        let mut books = ctx.data::<Storage>().lock().await;
        let id = id.parse::<usize>()?;
        if books.contains(id) {
            books.remove(id);
            SimpleBroker::publish(BookChanged {
                mutation_type: MutationType::Deleted,
                id: id.into(),
            });
            Ok(true)
        } else {
            Ok(false)
        }
    }
    */
}

/*
#[async_graphql::Enum]
enum MutationType {
    Created,
    Deleted,
}

#[async_graphql::SimpleObject]
#[derive(Clone)]
struct BookChanged {
    mutation_type: MutationType,
    id: ID,
}
*/

pub struct SubscriptionRoot;

#[async_graphql::Subscription]
impl SubscriptionRoot {
    /*
    async fn interval(&self, #[arg(default = 1)] n: i32) -> impl Stream<Item = i32> {
        let mut value = 0;
        tokio::time::interval(Duration::from_secs(1)).map(move |_| {
            value += n;
            value
        })
    }

    async fn books(&self, mutation_type: Option<MutationType>) -> impl Stream<Item = BookChanged> {
        SimpleBroker::<BookChanged>::subscribe().filter(move |event| {
            let res = if let Some(mutation_type) = mutation_type {
                event.mutation_type == mutation_type
            } else {
                true
            };
            async move { res }
        })
    }
    */
}
