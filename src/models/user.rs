use anyhow::{anyhow, Context as _, Result};
use async_graphql::{self, guard::Guard, Context, InputObject, Object};
use chrono::Utc;
use diesel::{
    dsl::{max, not, sum},
    expression::BoxableExpression,
    prelude::*,
    query_dsl::QueryDsl,
    r2d2::{ConnectionManager, PooledConnection},
    sql_types::{BigInt, Bool, Int4},
};
use rand::{distributions::Alphanumeric, Rng};
use ring::pbkdf2;
use std::num::NonZeroU32;

use super::bookmark::{BookmarkFilter, BookmarkOrder};
use super::comment::{CommentFilter, CommentOrder};
use super::favchat::{FavchatFilter, FavchatOrder};
use super::puzzle::{PuzzleFilter, PuzzleOrder};
use super::puzzle_tag::{PuzzleTagFilter, PuzzleTagOrder};
use super::star::{StarFilter, StarOrder};
use super::user_award::{UserAwardFilter, UserAwardOrder};
use super::*;

use crate::auth::Role;
use crate::context::GlobalCtx;
use crate::schema::user;

const SALT_LEN: usize = 16;
const CRED_LEN: usize = 32;
const ITER_TIMES: u32 = 100000;

/// Available orders for users query
#[derive(InputObject, Clone)]
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

        for obj in self.0 {
            gen_order!(obj, id, query);
            gen_order!(obj, nickname, query);
            gen_order!(obj, date_joined, query);
            gen_order!(obj, last_login, query);
        }

        query
    }
}

/// Available filters for users query
#[derive(InputObject, Clone)]
pub struct UserFilter {
    username: Option<StringFiltering>,
    nickname: Option<StringFiltering>,
}

impl CindyFilter<user::table, DB> for UserFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<user::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::user::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<user, DB, SqlType = Bool> + Send>> = None;
        let UserFilter {
            username: obj_username,
            nickname: obj_nickname,
        } = self;
        gen_string_filter!(obj_username, username, filter);
        gen_string_filter!(obj_nickname, nickname, filter);
        filter
    }
}

#[derive(QueryableByName, Debug)]
pub struct UserRankingRow {
    /// User ID
    #[sql_type = "Int4"]
    pub id: ID,
    /// The aggregated value of given user
    #[sql_type = "BigInt"]
    pub value_count: i64,
}

#[Object]
impl UserRankingRow {
    async fn id(&self) -> ID {
        self.id
    }
    async fn value_count(&self) -> i64 {
        self.value_count
    }

    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        use crate::schema::user;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user = user::table
            .filter(user::id.eq(self.id))
            .limit(1)
            .first(&conn)?;

        Ok(user)
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

#[Object]
impl User {
    async fn id(&self) -> ID {
        self.id
    }
    async fn username(&self) -> &str {
        &self.username
    }
    #[graphql(guard(and(
        DenyRoleGuard(role = "Role::User"),
        DenyRoleGuard(role = "Role::Guest")
    )))]
    async fn password(&self) -> &str {
        &self.password
    }
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
    async fn last_login(&self) -> Option<&Timestamptz> {
        self.last_login.as_ref()
    }
    async fn date_joined(&self) -> &Timestamptz {
        &self.date_joined
    }

    async fn current_award(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<UserAward>> {
        use crate::schema::user_award;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let current_award_inst = if let Some(id) = self.current_award_id {
            user_award::table
                .filter(user_award::id.eq(id))
                .limit(1)
                .first(&conn)
                .ok()
        } else {
            None
        };

        Ok(current_award_inst)
    }

    async fn bookmarks(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<BookmarkFilter>,
        order: Option<Vec<BookmarkOrder>>,
    ) -> async_graphql::Result<Vec<Bookmark>> {
        use crate::gql_schema::BookmarkQuery;

        let filter = filter
            .map(|mut filter| {
                filter.user_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| BookmarkFilter {
                user_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = BookmarkQuery::default();
        query
            .bookmarks(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }

    async fn comments(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<CommentFilter>,
        order: Option<Vec<CommentOrder>>,
    ) -> async_graphql::Result<Vec<Comment>> {
        use crate::gql_schema::CommentQuery;

        let filter = filter
            .map(|mut filter| {
                filter.user_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| CommentFilter {
                user_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = CommentQuery::default();
        query
            .comments(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }

    async fn received_comment_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::{comment, puzzle};

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = comment::table
            .inner_join(puzzle::table)
            .filter(puzzle::user_id.eq(self.id))
            .count()
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn comment_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::comment::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = comment
            .filter(user_id.eq(self.id))
            .count()
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn puzzles(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<PuzzleFilter>,
        order: Option<Vec<PuzzleOrder>>,
    ) -> async_graphql::Result<Vec<Puzzle>> {
        use crate::gql_schema::PuzzleQuery;

        let filter = filter
            .map(|mut filter| {
                filter.user_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| PuzzleFilter {
                user_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = PuzzleQuery::default();
        query
            .puzzles(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }

    async fn puzzle_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::puzzle::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = puzzle
            .filter(user_id.eq(self.id))
            .count()
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn yami_puzzle_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::puzzle::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = puzzle
            .filter(user_id.eq(self.id))
            .filter(not(yami.eq(0)))
            .count()
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn puzzle_max_created(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<Timestamptz>> {
        use crate::schema::puzzle::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = puzzle
            .filter(user_id.eq(self.id))
            .select(max(created))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn dialogue_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::dialogue::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = dialogue
            .filter(user_id.eq(self.id))
            .count()
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn stars(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<StarFilter>,
        order: Option<Vec<StarOrder>>,
    ) -> async_graphql::Result<Vec<Star>> {
        use crate::gql_schema::StarQuery;

        let filter = filter
            .map(|mut filter| {
                filter.user_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| StarFilter {
                user_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = StarQuery::default();
        query
            .stars(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }

    async fn received_star_sum(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<i64>> {
        use crate::schema::{puzzle, star};

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = star::table
            .inner_join(puzzle::table)
            .filter(puzzle::user_id.eq(self.id))
            .select(sum(star::value))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn star_sum(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<i64>> {
        use crate::schema::star::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = star
            .filter(user_id.eq(self.id))
            .select(sum(value))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn received_star_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::{puzzle, star};

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = star::table
            .inner_join(puzzle::table)
            .filter(puzzle::user_id.eq(self.id))
            .count()
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn star_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::star::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = star
            .filter(user_id.eq(self.id))
            .count()
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn favchats(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<FavchatFilter>,
        order: Option<Vec<FavchatOrder>>,
    ) -> async_graphql::Result<Vec<Favchat>> {
        use crate::gql_schema::FavchatQuery;

        let filter = filter
            .map(|mut filter| {
                filter.user_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| FavchatFilter {
                user_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = FavchatQuery::default();
        query
            .favchats(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }

    async fn puzzle_tags(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<PuzzleTagFilter>,
        order: Option<Vec<PuzzleTagOrder>>,
    ) -> async_graphql::Result<Vec<PuzzleTag>> {
        use crate::gql_schema::PuzzleTagQuery;

        let filter = filter
            .map(|mut filter| {
                filter.user_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| PuzzleTagFilter {
                user_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = PuzzleTagQuery::default();
        query
            .puzzle_tags(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }

    async fn user_awards(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<UserAwardFilter>,
        order: Option<Vec<UserAwardOrder>>,
    ) -> async_graphql::Result<Vec<UserAward>> {
        use crate::gql_schema::UserAwardQuery;

        let filter = filter
            .map(|mut filter| {
                filter.user_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| UserAwardFilter {
                user_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = UserAwardQuery::default();
        query
            .user_awards(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }

    async fn good_question_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::dialogue::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = dialogue
            .filter(user_id.eq(self.id))
            .filter(good.eq(true))
            .count()
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn true_answer_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::dialogue::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = dialogue
            .filter(user_id.eq(self.id))
            .filter(true_.eq(true))
            .count()
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
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
