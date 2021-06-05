use async_graphql::{MergedObject, MergedSubscription, Object, Schema, SimpleObject, Subscription};
//use futures::lock::Mutex;
use futures::{Stream, StreamExt};
//use std::sync::Arc;
use std::time::Duration;
use tokio_stream::wrappers::IntervalStream;

mod award;
mod bookmark;
mod chatmessage;
mod chatroom;
mod comment;
mod dialogue;
mod direct_message;
mod dm_read;
mod favchat;
mod hint;
mod puzzle;
mod puzzle_log;
mod puzzle_tag;
mod star;
mod tag;
mod user;
mod user_award;

pub use award::{AwardMutation, AwardQuery};
pub use bookmark::{BookmarkMutation, BookmarkQuery};
pub use chatmessage::{ChatmessageMutation, ChatmessageQuery, ChatmessageSubscription};
pub use chatroom::{ChatroomMutation, ChatroomQuery};
pub use comment::{CommentMutation, CommentQuery};
pub use dialogue::{DialogueMutation, DialogueQuery};
pub use direct_message::{DirectMessageMutation, DirectMessageQuery, DirectMessageSubscription};
pub use dm_read::{DmReadMutation, DmReadQuery};
pub use favchat::{FavchatMutation, FavchatQuery};
pub use hint::{HintMutation, HintQuery};
pub use puzzle::{PuzzleMutation, PuzzleQuery, PuzzleSubscription};
pub use puzzle_log::{PuzzleLogQuery, PuzzleLogSubscription};
pub use puzzle_tag::{PuzzleTagMutation, PuzzleTagQuery};
pub use star::{StarMutation, StarQuery};
pub use tag::{TagMutation, TagQuery};
pub use user::{UserMutation, UserQuery};
pub use user_award::{UserAwardMutation, UserAwardQuery};

pub type CindySchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    AwardQuery,
    BaseQuery,
    BookmarkQuery,
    ChatmessageQuery,
    ChatroomQuery,
    CommentQuery,
    DialogueQuery,
    DirectMessageQuery,
    DmReadQuery,
    FavchatQuery,
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
    DirectMessageMutation,
    DmReadMutation,
    FavchatMutation,
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
    DirectMessageSubscription,
    PuzzleLogSubscription,
    PuzzleSubscription,
);

#[derive(Default)]
struct BaseQuery;

#[Object]
impl BaseQuery {
    async fn online_users_count(&self) -> i32 {
        use crate::broker::online_users_count;
        online_users_count()
    }
}

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
            let mut stream =
                IntervalStream::new(tokio::time::interval(Duration::from_secs(n as u64)));
            loop {
                stream.next().await;
                CindyBroker::publish(IntervalMsg::new("hello, world!".to_string()));
            }
        });

        CindyBroker::<IntervalMsg>::subscribe()
    }
}
