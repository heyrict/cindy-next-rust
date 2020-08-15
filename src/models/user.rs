use anyhow::{anyhow, Context as _, Result};
use async_graphql::{Context, FieldResult};
use chrono::Utc;
use diesel::expression::BoxableExpression;
use diesel::prelude::*;
use diesel::query_dsl::{methods::ThenOrderDsl, QueryDsl};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sql_types::Bool;
use ring::pbkdf2;
use std::num::NonZeroU32;

use super::generics::*;
use super::puzzle::*;

use crate::context::GlobalCtx;
use crate::schema::user;
use rand::{distributions::Alphanumeric, Rng};

const SALT_LEN: usize = 16;
const CRED_LEN: usize = 32;
const ITER_TIMES: u32 = 100000;

/// Available orders for users query
#[async_graphql::InputObject]
pub struct UserOrder {
    id: Option<Ordering>,
    nickname: Option<Ordering>,
    date_joined: Option<Ordering>,
    last_login: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct UserOrders(Vec<UserOrder>);

impl Default for UserOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl UserOrders {
    pub fn new(orders: Vec<UserOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: crate::schema::user::BoxedQuery<'a, DB>,
    ) -> crate::schema::user::BoxedQuery<'a, DB> {
        use crate::schema::user::dsl::*;

        let mut query = query_dsl;
        let mut flag = false;

        for obj in self.0 {
            gen_order!(obj, id, query, flag);
            gen_order!(obj, nickname, query, flag);
            gen_order!(obj, date_joined, query, flag);
            gen_order!(obj, last_login, query, flag);
        }

        query
    }
}

/// Available filters for users query
#[async_graphql::InputObject]
pub struct UserFilter {
    username: Option<StringFiltering>,
    nickname: Option<StringFiltering>,
}

impl CindyFilter<user::table, DB> for UserFilter {
    fn as_expression(self) -> Option<Box<dyn BoxableExpression<user::table, DB, SqlType = Bool>>> {
        use crate::schema::user::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<user, DB, SqlType = Bool>>> = None;
        let UserFilter {
            username: obj_username,
            nickname: obj_nickname,
        } = self;
        gen_string_filter!(obj_username, username, filter);
        gen_string_filter!(obj_nickname, nickname, filter);
        filter
    }
}

impl CindyFilter<user::table, DB> for Vec<UserFilter> {
    fn as_expression(self) -> Option<Box<dyn BoxableExpression<user::table, DB, SqlType = Bool>>> {
        let mut filter: Option<Box<dyn BoxableExpression<user::table, DB, SqlType = Bool>>> = None;
        for item in self.into_iter() {
            if let Some(item) = item.as_expression() {
                filter = Some(if let Some(filter_) = filter {
                    Box::new(filter_.or(item))
                } else {
                    Box::new(item)
                });
            }
        }
        filter
    }
}

/// Object for user table
#[derive(Queryable, Identifiable, Debug)]
#[table_name = "user"]
pub struct User {
    pub id: ID,
    pub password: String,
    pub last_login: Option<Timestamptz>,
    pub is_superuser: bool,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub is_staff: bool,
    pub is_active: bool,
    pub date_joined: Timestamptz,
    pub nickname: String,
    pub profile: String,
    pub current_award_id: Option<i32>,
    pub hide_bookmark: bool,
    pub last_read_dm_id: Option<i32>,
    pub icon: Option<String>,
}

#[async_graphql::Object]
impl User {
    async fn id(&self) -> ID {
        self.id
    }
    async fn username(&self) -> &str {
        &self.username
    }
    /*
    async fn password(&self) -> &str {
        &self.password
    }
    */
    async fn nickname(&self) -> &str {
        &self.nickname
    }
    async fn profile(&self) -> &str {
        &self.profile
    }
    async fn hide_bookmark(&self) -> bool {
        self.hide_bookmark
    }
    async fn icon(&self) -> Option<&String> {
        self.icon.as_ref()
    }
    async fn first_name(&self) -> &str {
        &self.first_name
    }
    async fn last_name(&self) -> &str {
        &self.last_name
    }
    async fn email(&self) -> &str {
        &self.email
    }
    async fn is_superuser(&self) -> bool {
        self.is_superuser
    }
    async fn is_staff(&self) -> bool {
        self.is_staff
    }
    async fn is_active(&self) -> bool {
        self.is_active
    }
    async fn last_login(&self) -> Option<String> {
        self.last_login.map(|ts| ts.to_string())
    }
    async fn date_joined(&self) -> String {
        self.date_joined.to_string()
    }

    async fn puzzles<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<PuzzleFilter>>,
        order: Option<Vec<PuzzleOrder>>,
    ) -> FieldResult<Vec<Puzzle>> {
        use crate::schema::puzzle::dsl::*;

        let conn = ctx.data::<GlobalCtx>().get_conn()?;

        let mut query = puzzle.filter(user_id.eq(self.id)).into_boxed();
        if let Some(order) = order {
            query = PuzzleOrders::new(order).apply_order(query);
        }
        if let Some(filter) = filter {
            if let Some(filter_exp) = filter.as_expression() {
                query = query.filter(filter_exp)
            }
        }
        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let puzzles = query.load::<Puzzle>(&conn)?;

        Ok(puzzles)
    }
}

struct Password {
    pub alg: pbkdf2::Algorithm,
    pub salt: Vec<u8>,
    pub iter: NonZeroU32,
    pub credential: Vec<u8>,
}

impl User {
    fn salt() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(SALT_LEN)
            .collect()
    }

    pub fn derive_credential(password: &str) -> String {
        let mut credential = [0u8; CRED_LEN];
        let salt = Self::salt();
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(ITER_TIMES).unwrap(),
            salt.as_bytes(),
            password.as_bytes(),
            &mut credential,
        );
        let credential = base64::encode(credential.as_ref());
        format!("pbkdf2_sha256${}${}${}", ITER_TIMES, salt, credential)
    }

    /// Authenticate the user.
    ///
    /// Returns `Ok(user)` if authentication passed, otherwise `Err(error)`.
    pub async fn local_auth(
        username: &str,
        password: &str,
        conn: PooledConnection<ConnectionManager<PgConnection>>,
    ) -> Result<Self> {
        use crate::schema::user::last_login;

        let usr: Self = user::table
            .filter(user::username.eq(username))
            .limit(1)
            .first(&conn)
            .context("User does not exist. Please re-check your username and password.")?;

        if !usr.is_active {
            return Err(anyhow!("User is not activated by administrator. Contact the administrator for more details."));
        }

        let Password {
            alg,
            iter,
            salt,
            credential,
        } = usr.decompose_password()?;
        let mut reproduce = [0u8; CRED_LEN];
        pbkdf2::derive(alg, iter, &salt, password.as_bytes(), &mut reproduce);
        pbkdf2::verify(alg, iter, &salt, password.as_bytes(), &credential)
            .map_err(|_| anyhow!("Invalid password"))?;

        diesel::update(&usr)
            .set(last_login.eq(Some(Utc::now())))
            .execute(&conn)?;
        Ok(usr)
    }

    fn decompose_password(&self) -> Result<Password> {
        let mut parts = self.password.split('$');
        let _alg = parts
            .next()
            .ok_or(anyhow!("Unable to parse password: algorithm not found"))?;
        let iter = parts.next().ok_or(anyhow!(
            "Unable to parse password: iteration number not found"
        ))?;
        let salt = parts
            .next()
            .ok_or(anyhow!("Unable to parse password: salt not found"))?;
        let credential = parts
            .next()
            .ok_or(anyhow!("Unable to parse password: credential not found"))?;

        Ok(Password {
            alg: pbkdf2::PBKDF2_HMAC_SHA256,
            iter: iter.parse()?,
            salt: salt.as_bytes().to_owned(),
            credential: base64::decode(&credential)?,
        })
    }
}
