use async_graphql::{MergedObject, MergedSubscription, Schema, SimpleObject, Subscription};
//use futures::lock::Mutex;
use futures::{Stream, StreamExt};
//use std::sync::Arc;
use std::time::Duration;

mod award;
mod bookmark;
mod chatmessage;
mod chatroom;
mod dialogue;
mod hint;
mod puzzle;
mod puzzle_log;
mod user;

use award::{AwardMutation, AwardQuery};
use bookmark::{BookmarkMutation, BookmarkQuery};
use chatmessage::{ChatmessageMutation, ChatmessageQuery};
use chatroom::{ChatroomMutation, ChatroomQuery};
use dialogue::{DialogueMutation, DialogueQuery};
use hint::{HintMutation, HintQuery};
use puzzle::{PuzzleMutation, PuzzleQuery, PuzzleSubscription};
use puzzle_log::{PuzzleLogQuery, PuzzleLogSubscription};
use user::{UserMutation, UserQuery};

pub type CindySchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    AwardQuery,
    BookmarkQuery,
    ChatmessageQuery,
    ChatroomQuery,
    DialogueQuery,
    HintQuery,
    PuzzleLogQuery,
    PuzzleQuery,
    UserQuery,
);

#[derive(MergedObject, Default)]
pub struct MutationRoot(
    AwardMutation,
    BookmarkMutation,
    ChatmessageMutation,
    ChatroomMutation,
    DialogueMutation,
    HintMutation,
    PuzzleMutation,
    UserMutation,
);

#[derive(MergedSubscription, Default)]
pub struct SubscriptionRoot(BaseSubscription, PuzzleSubscription, PuzzleLogSubscription);

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
