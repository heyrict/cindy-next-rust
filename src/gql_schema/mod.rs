use async_graphql::{MergedObject, MergedSubscription, Schema, SimpleObject, Subscription};
//use futures::lock::Mutex;
use futures::{Stream, StreamExt};
//use std::sync::Arc;
use std::time::Duration;

mod hint;
mod puzzle;
mod user;

use hint::{HintMutation, HintQuery};
use puzzle::{PuzzleMutation, PuzzleQuery, PuzzleSubscription};
use user::{UserMutation, UserQuery};

pub type CindySchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(UserQuery, PuzzleQuery, HintQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(UserMutation, PuzzleMutation, HintMutation);

#[derive(MergedSubscription, Default)]
pub struct SubscriptionRoot(BaseSubscription, PuzzleSubscription);

#[derive(Clone, Default, SimpleObject)]
struct IntervalMsg {
    msg: String,
}

impl IntervalMsg {
    pub fn new(msg: String) -> Self {
        IntervalMsg { msg }
    }
}

#[derive(Default)]
struct BaseSubscription;

#[Subscription]
impl BaseSubscription {
    async fn interval(
        &self,
        #[graphql(default = 1)] n: i32,
    ) -> impl Stream<Item = Option<IntervalMsg>> {
        use crate::broker::CindyBroker;

        tokio::spawn(async move {
            let mut stream = tokio::time::interval(Duration::from_secs(n as u64)).map(move |_| {
                CindyBroker::publish(IntervalMsg::new("hello, world!".to_string()));
            });
            loop {
                stream.next().await;
            }
        });

        CindyBroker::<IntervalMsg>::subscribe()
    }
}
