use async_graphql::{MergedObject, MergedSubscription, Schema, SimpleObject, Subscription};
//use futures::lock::Mutex;
use futures::{Stream, StreamExt};
//use std::sync::Arc;
use std::time::Duration;

mod award;
mod bookmark;
mod chatmessage;
mod chatroom;
mod comment;
mod dialogue;
mod hint;
mod puzzle;
mod puzzle_log;
mod puzzle_tag;
mod star;
mod tag;
mod user;
mod user_award;

use award::{AwardMutation, AwardQuery};
use bookmark::{BookmarkMutation, BookmarkQuery};
use chatmessage::{ChatmessageMutation, ChatmessageQuery, ChatmessageSubscription};
use chatroom::{ChatroomMutation, ChatroomQuery};
use comment::{CommentMutation, CommentQuery};
use dialogue::{DialogueMutation, DialogueQuery};
use hint::{HintMutation, HintQuery};
use puzzle::{PuzzleMutation, PuzzleQuery, PuzzleSubscription};
use puzzle_log::{PuzzleLogQuery, PuzzleLogSubscription};
use puzzle_tag::{PuzzleTagMutation, PuzzleTagQuery};
use star::{StarMutation, StarQuery};
use tag::{TagMutation, TagQuery};
use user::{UserMutation, UserQuery};
use user_award::{UserAwardMutation, UserAwardQuery};

pub type CindySchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    AwardQuery,
    BookmarkQuery,
    ChatmessageQuery,
    ChatroomQuery,
    CommentQuery,
    DialogueQuery,
    HintQuery,
    PuzzleLogQuery,
    PuzzleQuery,
    PuzzleTagQuery,
    StarQuery,
    TagQuery,
    UserQuery,
    UserAwardQuery,
);

#[derive(MergedObject, Default)]
pub struct MutationRoot(
    AwardMutation,
    BookmarkMutation,
    ChatmessageMutation,
    ChatroomMutation,
    CommentMutation,
    DialogueMutation,
    HintMutation,
    PuzzleMutation,
    PuzzleTagMutation,
    StarMutation,
    TagMutation,
    UserMutation,
    UserAwardMutation,
);

#[derive(MergedSubscription, Default)]
pub struct SubscriptionRoot(
    BaseSubscription,
    ChatmessageSubscription,
    PuzzleLogSubscription,
    PuzzleSubscription,
);

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
