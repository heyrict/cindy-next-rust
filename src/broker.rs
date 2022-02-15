use chrono::{Date, Duration, Local};
use futures::Stream;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::convert::TryInto;
use std::marker::{PhantomData, Unpin};
use std::sync::Mutex;
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;

pub struct SubscribePair {
    pub tx: Box<dyn Any + Send>,
    pub rx: Box<dyn Any + Send>,
    pub updated: Date<Local>,
}

impl SubscribePair {
    pub fn new(tx: Box<dyn Any + Send>, rx: Box<dyn Any + Send>) -> Self {
        SubscribePair {
            tx,
            rx,
            updated: Local::today(),
        }
    }
}

type Key = String;

lazy_static! {
    static ref SUBSCRIPTIONS: Mutex<HashMap<TypeId, HashMap<Key, SubscribePair>>> =
        Default::default();
}

fn with_senders_to<T, SP, F>(key: Key, f: F) -> SP
where
    T: Sync + Send + Unpin + Clone + 'static,
    F: FnOnce(&watch::Sender<Option<T>>, &watch::Receiver<Option<T>>) -> SP,
{
    let mut map = SUBSCRIPTIONS.lock().unwrap();
    let submap = map
        .entry(TypeId::of::<T>())
        .or_insert_with(|| Default::default());
    let sp = submap.entry(key).or_insert_with(|| {
        let (tx, rx) = watch::channel::<Option<T>>(None);
        SubscribePair::new(Box::new(tx), Box::new(rx))
    });
    let today = Local::today();
    if sp.updated != today {
        sp.updated = today;
    };
    let tx = sp.tx.downcast_ref::<watch::Sender<Option<T>>>().unwrap();
    let rx = sp.rx.downcast_ref::<watch::Receiver<Option<T>>>().unwrap();
    f(tx, rx)
}

fn with_senders_to_if_exists<T, SP, F>(key: Key, f: F) -> Option<SP>
where
    T: Sync + Send + Unpin + Clone + 'static,
    F: FnOnce(&watch::Sender<Option<T>>, &watch::Receiver<Option<T>>) -> SP,
{
    let mut map = SUBSCRIPTIONS.lock().unwrap();
    let type_id = TypeId::of::<T>();
    if map.contains_key(&type_id) {
        let submap = map.get_mut(&type_id).unwrap();
        if submap.contains_key(&key) {
            let sp = submap.get_mut(&key).unwrap();
            let today = Local::today();
            if sp.updated != today {
                sp.updated = today;
            };
            let tx = sp.tx.downcast_ref::<watch::Sender<Option<T>>>().unwrap();
            let rx = sp.rx.downcast_ref::<watch::Receiver<Option<T>>>().unwrap();
            Some(f(tx, rx))
        } else {
            None
        }
    } else {
        None
    }
}

/// A simple broker based on memory
pub struct CindyBroker<T>(PhantomData<T>);

impl<T: Sync + Unpin + Send + Clone + 'static> CindyBroker<T> {
    /// Publish a message that all subscription streams can receive.
    pub fn publish(msg: T) {
        with_senders_to_if_exists::<T, _, _>(Key::default(), |tx, _| {
            tx.send(Some(msg.clone())).ok();
        });
    }

    /// Subscribe to the message of the specified type and returns a `Stream`.
    pub fn subscribe() -> impl Stream<Item = Option<T>> {
        with_senders_to::<T, _, _>(Key::default(), |_, rx| WatchStream::new(rx.clone()))
    }

    /// Publish a message that all subscription streams can receive with a given key.
    pub fn publish_to(key: Key, msg: T) {
        with_senders_to_if_exists::<T, _, _>(key, |tx, _| {
            tx.send(Some(msg.clone())).ok();
        });
    }

    /// Publish a message that all subscription streams can receive with a given key filter.
    pub fn publish_to_all(filter: impl Fn(&Key) -> bool, msg: T) {
        let mut map = SUBSCRIPTIONS.lock().unwrap();
        let submap = map
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Default::default());
        submap
            .iter_mut()
            .filter(|(key, _)| filter(key))
            .for_each(|(_, sp)| {
                let today = Local::today();
                if sp.updated != today {
                    sp.updated = today;
                };
                let tx = sp.tx.downcast_ref::<watch::Sender<Option<T>>>().unwrap();
                tx.send(Some(msg.clone())).ok();
            });
    }

    /// Subscribe to the message of the specified type with a given key and returns a `Stream`.
    pub fn subscribe_to(key: Key) -> impl Stream<Item = Option<T>> {
        with_senders_to::<T, _, _>(key, |_, rx| WatchStream::new(rx.clone()))
    }
}

/// Number of online users
pub fn online_users_count() -> u64 {
    type DmType = crate::models::chatmessage::ChatmessageSub;

    let mut count = 0;
    let mut map = SUBSCRIPTIONS.lock().unwrap();
    let submap = map
        .entry(TypeId::of::<DmType>())
        .or_insert_with(|| Default::default());

    for (_, sp) in submap.iter() {
        let tx = sp
            .tx
            .downcast_ref::<watch::Sender<Option<DmType>>>()
            .unwrap();
        if !tx.is_closed() {
            count += tx.receiver_count();
        }
    }

    count.try_into().unwrap()
}

pub fn cleanup() {
    let mut map = SUBSCRIPTIONS.lock().unwrap();
    let today = Local::today();
    let env_max_cache_days = dotenv::var("SUBSCRIPTION_MAX_CACHE_TIME")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    let max_cache_time = Duration::days(env_max_cache_days);

    for (_, submap) in map.iter_mut() {
        let keys: Vec<Key> = submap
            .keys()
            .into_iter()
            .map(|key| key.to_owned())
            .collect();
        for key in keys {
            if submap[&key].updated - today > max_cache_time {
                submap.remove(&key);
            }
        }
    }
}
