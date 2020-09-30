use async_graphql::{Context, FieldResult, Object, Schema, Subscription};
//use futures::lock::Mutex;
use futures::{Stream, StreamExt};
//use std::sync::Arc;
use std::time::Duration;

use crate::models::*;

mod puzzle;
mod user;

use user::UpdateUserSet;

pub type CindySchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn user(&self, ctx: &Context<'_>, id: i32) -> FieldResult<User> {
        self.user_(ctx, id).await
    }
    async fn users(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<UserFilter>>,
        order: Option<Vec<UserOrder>>,
    ) -> FieldResult<Vec<User>> {
        self.users_(ctx, limit, offset, filter, order).await
    }

    async fn puzzle(&self, ctx: &Context<'_>, id: i32) -> FieldResult<Puzzle> {
        self.puzzle_(ctx, id).await
    }
    async fn puzzles(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<PuzzleFilter>>,
        order: Option<Vec<PuzzleOrder>>,
    ) -> FieldResult<Vec<Puzzle>> {
        self.puzzles_(ctx, limit, offset, filter, order).await
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn update_user(
        &self,
        ctx: &Context<'_>,
        id: ID,
        set: UpdateUserSet,
    ) -> FieldResult<User> {
        self.update_user_(ctx, id, set).await
    }
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

#[Subscription]
impl SubscriptionRoot {
    async fn interval(&self, #[graphql(default = 1)] n: i32) -> impl Stream<Item = i32> {
        let mut value = 0;
        tokio::time::interval(Duration::from_secs(1)).map(move |_| {
            value += n;
            value
        })
    }
    /*
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
