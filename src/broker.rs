use futures::{
    task::{Context, Poll},
    Stream, StreamExt,
};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Mutex;
use tokio::sync::watch;

type SubscribePair = (Box<dyn Any + Send>, Box<dyn Any + Send>);

lazy_static! {
    static ref SUBSCRIPTIONS: Mutex<HashMap<TypeId, SubscribePair>> = Default::default();
}

struct BrokerStream<T: Sync + Send + Clone + 'static>(watch::Receiver<Option<T>>);

fn with_senders<T, SP, F>(f: F) -> SP
where
    T: Sync + Send + Clone + 'static,
    F: FnOnce(&watch::Sender<Option<T>>, &watch::Receiver<Option<T>>) -> SP,
{
    let mut map = SUBSCRIPTIONS.lock().unwrap();
    let sp = map.entry(TypeId::of::<T>()).or_insert_with(|| {
        let (tx, rx) = watch::channel::<Option<T>>(None);
        (Box::new(tx), Box::new(rx))
    });
    let tx = sp.0.downcast_ref::<watch::Sender<Option<T>>>().unwrap();
    let rx = sp.1.downcast_ref::<watch::Receiver<Option<T>>>().unwrap();
    f(tx, rx)
}

impl<T: Sync + Send + Clone + 'static> Stream for BrokerStream<T> {
    type Item = Option<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx)
    }
}

/// A simple broker based on memory
pub struct CindyBroker<T>(PhantomData<T>);

impl<T: Sync + Send + Clone + 'static> CindyBroker<T> {
    /// Publish a message that all subscription streams can receive.
    pub fn publish(msg: T) {
        with_senders::<T, _, _>(|tx, _| {
            tx.broadcast(Some(msg.clone())).ok();
        });
    }

    /// Subscribe to the message of the specified type and returns a `Stream`.
    pub fn subscribe() -> impl Stream<Item = Option<T>> {
        with_senders::<T, _, _>(|_, rx| BrokerStream(rx.clone()))
    }
}
