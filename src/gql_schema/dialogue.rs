use async_graphql::{self, guard::Guard, Context, InputObject, Object, Subscription};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use futures::{Stream, StreamExt};

use crate::auth::Role;
use crate::broker::CindyBroker;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::puzzle::*;
use crate::models::*;
use crate::schema::puzzle;

#[derive(Default)]
pub struct DialogueQuery;
#[derive(Default)]
pub struct DialogueMutation;
#[derive(Default)]
pub struct DialogueSubscription;

