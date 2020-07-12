use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use diesel::prelude::*;
use diesel::query_dsl::methods::ThenOrderDsl;
use diesel::query_dsl::QueryDsl;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use djangohashers::check_password_tolerant;

use super::generics::*;

use crate::schema::user;

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

/// Helper object to apply the filtering to the query
pub struct UserFilters(Vec<UserFilter>);

impl Default for UserFilters {
    fn default() -> Self {
        Self(vec![])
    }
}

impl UserFilters {
    pub fn new(orders: Vec<UserFilter>) -> Self {
        Self(orders)
    }

    pub fn apply_filter<'a>(
        self,
        query_dsl: crate::schema::user::BoxedQuery<'a, DB>,
    ) -> crate::schema::user::BoxedQuery<'a, DB> {
        use crate::schema::user::dsl::*;

        let mut query = query_dsl;

        for (index, obj) in self.0.into_iter().enumerate() {
            let UserFilter {
                username: obj_username,
                nickname: obj_nickname,
            } = obj;
            gen_string_filter!(obj_username, username, query, index);
            gen_string_filter!(obj_nickname, nickname, query, index);
        }

        query
    }
}

/// Object for user table
#[derive(Queryable, Identifiable, Insertable)]
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

impl Default for User {
    fn default() -> Self {
        Self {
            date_joined: Utc::now(),
            ..Default::default()
        }
    }
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
}

impl User {
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

        let password_valid = check_password_tolerant(password, &usr.password);

        if password_valid {
            diesel::update(&usr)
                .set(last_login.eq(Some(Utc::now())))
                .execute(&conn)?;
            Ok(usr)
        } else {
            Err(anyhow!(
                "Error authenticating user {}, please try again.",
                username
            ))
        }
    }
}
