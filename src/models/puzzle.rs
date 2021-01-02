use async_graphql::{self, Context, Enum, InputObject, Object};
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    dsl::{max, sum},
    expression::{helper_types::AsExprOf, AsExpression},
    prelude::*,
    query_dsl::QueryDsl,
    serialize::{self, Output, ToSql},
    sql_types::{BigInt, Bool, Int4, Integer, Text},
};
use std::io;

use crate::context::GlobalCtx;
use crate::schema::puzzle;

use super::bookmark::{BookmarkFilter, BookmarkOrder};
use super::comment::{CommentFilter, CommentOrder};
use super::dialogue::{DialogueFilter, DialogueOrder};
use super::hint::{HintFilter, HintOrder};
use super::puzzle_tag::{PuzzleTagFilter, PuzzleTagOrder};
use super::star::{StarFilter, StarOrder};
use super::*;

/// Available orders for puzzle query
#[derive(InputObject, Clone)]
pub struct PuzzleOrder {
    id: Option<Ordering>,
    created: Option<Ordering>,
    modified: Option<Ordering>,
    yami: Option<Ordering>,
    genre: Option<Ordering>,
    status: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct PuzzleOrders(Vec<PuzzleOrder>);

impl Default for PuzzleOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl PuzzleOrders {
    pub fn new(orders: Vec<PuzzleOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: crate::schema::puzzle::BoxedQuery<'a, DB>,
    ) -> crate::schema::puzzle::BoxedQuery<'a, DB> {
        use crate::schema::puzzle::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, id, query);
            gen_order!(obj, yami, query);
            gen_order!(obj, genre, query);
            gen_order!(obj, created, query);
            gen_order!(obj, modified, query);
            gen_order!(obj, status, query);
        }

        query
    }
}

/// Available filters for puzzle query
#[derive(InputObject, Clone, Default)]
pub struct PuzzleFilter {
    pub id: Option<I32Filtering>,
    pub anonymous: Option<bool>,
    pub title: Option<StringFiltering>,
    pub genre: Option<GenreFiltering>,
    pub yami: Option<YamiFiltering>,
    pub status: Option<StatusFiltering>,
    pub content: Option<StringFiltering>,
    pub solution: Option<StringFiltering>,
    pub user_id: Option<I32Filtering>,
    pub created: Option<TimestamptzFiltering>,
    pub modified: Option<TimestamptzFiltering>,
}

impl CindyFilter<puzzle::table, DB> for PuzzleFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<puzzle::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::puzzle::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<puzzle, DB, SqlType = Bool> + Send>> =
            None;
        let PuzzleFilter {
            id: obj_id,
            anonymous: obj_anonymous,
            title: obj_title,
            genre: obj_genre,
            yami: obj_yami,
            status: obj_status,
            content: obj_content,
            solution: obj_solution,
            user_id: obj_user_id,
            created: obj_created,
            modified: obj_modified,
        } = self;
        gen_number_filter!(obj_id: I32Filtering, id, filter);
        gen_bool_filter!(obj_anonymous, anonymous, filter);
        gen_string_filter!(obj_title, title, filter);
        gen_enum_filter!(obj_genre: GenreFiltering, genre, filter);
        gen_enum_filter!(obj_yami: YamiFiltering, yami, filter);
        gen_enum_filter!(obj_status: StatusFiltering, status, filter);
        gen_string_filter!(obj_content, content, filter);
        gen_string_filter!(obj_solution, solution, filter);
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);
        gen_number_filter!(obj_created: TimestamptzFiltering, created, filter);
        gen_number_filter!(obj_modified: TimestamptzFiltering, modified, filter);
        filter
    }
}

#[derive(InputObject, Eq, PartialEq, Clone)]
pub struct YamiFiltering {
    pub eq: Option<Yami>,
    pub ne: Option<Yami>,
    pub eq_any: Option<Vec<Yami>>,
    pub ne_all: Option<Vec<Yami>>,
}

impl RawFilter<Yami> for YamiFiltering {
    fn check(&self, item: &Yami) -> bool {
        if let Some(eq) = self.eq.as_ref() {
            item == eq
        } else if let Some(ne) = self.ne.as_ref() {
            item != ne
        } else if let Some(eq_any) = self.eq_any.as_ref() {
            eq_any.iter().any(|u| u == item)
        } else if let Some(ne_all) = self.ne_all.as_ref() {
            ne_all.iter().all(|u| u != item)
        } else {
            true
        }
    }
}

#[derive(Enum, Eq, PartialEq, Clone, Copy, Debug, FromSqlRow)]
pub enum Yami {
    None = 0,
    Normal = 1,
    Longterm = 2,
}

impl<DB> ToSql<Integer, DB> for Yami
where
    DB: Backend,
    i32: ToSql<Integer, DB>,
{
    fn to_sql<W: io::Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        (*self as i32).to_sql(out)
    }
}

impl AsExpression<Integer> for Yami {
    type Expression = AsExprOf<i32, Integer>;

    fn as_expression(self) -> Self::Expression {
        <i32 as AsExpression<Integer>>::as_expression(self as i32)
    }
}

impl<DB> FromSql<Integer, DB> for Yami
where
    DB: Backend,
    i32: FromSql<Integer, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            0 => Ok(Yami::None),
            1 => Ok(Yami::Normal),
            2 => Ok(Yami::Longterm),
            v => Err(format!("Invalid value `{}` for yami", &v).into()),
        }
    }
}

#[derive(InputObject, Eq, PartialEq, Clone)]
pub struct GenreFiltering {
    pub eq: Option<Genre>,
    pub ne: Option<Genre>,
    pub eq_any: Option<Vec<Genre>>,
    pub ne_all: Option<Vec<Genre>>,
}

#[derive(Enum, Eq, PartialEq, Copy, Clone, Debug, FromSqlRow)]
pub enum Genre {
    Classic = 0,
    TwentyQuestions = 1,
    LittleAlbat = 2,
    Others = 3,
}

impl RawFilter<Genre> for GenreFiltering {
    fn check(&self, item: &Genre) -> bool {
        if let Some(eq) = self.eq.as_ref() {
            item == eq
        } else if let Some(ne) = self.ne.as_ref() {
            item != ne
        } else if let Some(eq_any) = self.eq_any.as_ref() {
            eq_any.iter().any(|u| u == item)
        } else if let Some(ne_all) = self.ne_all.as_ref() {
            ne_all.iter().all(|u| u != item)
        } else {
            true
        }
    }
}

impl<DB> ToSql<Integer, DB> for Genre
where
    DB: Backend,
    i32: ToSql<Integer, DB>,
{
    fn to_sql<W: io::Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        (*self as i32).to_sql(out)
    }
}

impl AsExpression<Integer> for Genre {
    type Expression = AsExprOf<i32, Integer>;

    fn as_expression(self) -> Self::Expression {
        <i32 as AsExpression<Integer>>::as_expression(self as i32)
    }
}

impl<DB> FromSql<Integer, DB> for Genre
where
    DB: Backend,
    i32: FromSql<Integer, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            0 => Ok(Genre::Classic),
            1 => Ok(Genre::TwentyQuestions),
            2 => Ok(Genre::LittleAlbat),
            3 => Ok(Genre::Others),
            v => Err(format!("Invalid value `{}` for genre", &v).into()),
        }
    }
}

#[derive(Enum, Eq, PartialEq, Clone, Copy, Debug, FromSqlRow)]
pub enum Status {
    Undergoing = 0,
    Solved = 1,
    Dazed = 2,
    Hidden = 3,
    ForceHidden = 4,
}

#[derive(InputObject, Eq, PartialEq, Clone)]
pub struct StatusFiltering {
    pub eq: Option<Status>,
    pub ne: Option<Status>,
    pub eq_any: Option<Vec<Status>>,
    pub ne_all: Option<Vec<Status>>,
}

impl RawFilter<Status> for StatusFiltering {
    fn check(&self, item: &Status) -> bool {
        if let Some(eq) = self.eq.as_ref() {
            item == eq
        } else if let Some(ne) = self.ne.as_ref() {
            item != ne
        } else if let Some(eq_any) = self.eq_any.as_ref() {
            eq_any.iter().any(|u| u == item)
        } else if let Some(ne_all) = self.ne_all.as_ref() {
            ne_all.iter().all(|u| u != item)
        } else {
            true
        }
    }
}

impl<DB> ToSql<Integer, DB> for Status
where
    DB: Backend,
    i32: ToSql<Integer, DB>,
{
    fn to_sql<W: io::Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        (*self as i32).to_sql(out)
    }
}

impl AsExpression<Integer> for Status {
    type Expression = AsExprOf<i32, Integer>;

    fn as_expression(self) -> Self::Expression {
        <i32 as AsExpression<Integer>>::as_expression(self as i32)
    }
}

impl<DB> FromSql<Integer, DB> for Status
where
    DB: Backend,
    i32: FromSql<Integer, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            0 => Ok(Status::Undergoing),
            1 => Ok(Status::Solved),
            2 => Ok(Status::Dazed),
            3 => Ok(Status::Hidden),
            4 => Ok(Status::ForceHidden),
            v => Err(format!("Invalid value `{}` for genre", &v).into()),
        }
    }
}

#[derive(Clone)]
pub enum PuzzleSub {
    Created(Puzzle),
    Updated(Puzzle, Puzzle),
}

#[Object]
impl PuzzleSub {
    async fn op(&self) -> DbOp {
        match &self {
            PuzzleSub::Created(_) => DbOp::Created,
            PuzzleSub::Updated(_, _) => DbOp::Updated,
        }
    }

    async fn data(&self) -> Puzzle {
        match &self {
            PuzzleSub::Created(puzzle) => puzzle.clone(),
            PuzzleSub::Updated(_, puzzle) => puzzle.clone(),
        }
    }
}

#[derive(QueryableByName, Clone, Debug)]
pub struct PuzzleCountByGenre {
    #[sql_type = "Integer"]
    pub genre: Genre,
    #[sql_type = "BigInt"]
    pub puzzle_count: i64,
}

#[Object]
impl PuzzleCountByGenre {
    async fn genre(&self) -> Genre {
        self.genre
    }
    async fn puzzle_count(&self) -> i64 {
        self.puzzle_count
    }
}

#[derive(QueryableByName, Clone, Debug)]
pub struct PuzzleStarAggrGroup {
    #[sql_type = "BigInt"]
    pub group: i64,
    #[sql_type = "BigInt"]
    pub puzzle_count: i64,
}

#[Object]
impl PuzzleStarAggrGroup {
    async fn group(&self) -> i64 {
        self.group
    }
    async fn puzzle_count(&self) -> i64 {
        self.puzzle_count
    }
}

#[derive(QueryableByName, Clone, Debug)]
pub struct PuzzleParticipant {
    /// User ID
    #[sql_type = "Int4"]
    pub id: ID,
    #[sql_type = "Text"]
    pub nickname: String,
    /// Whether user got at least one true answer.
    #[sql_type = "Bool"]
    pub true_answer: bool,
    #[sql_type = "BigInt"]
    pub dialogue_count: i64,
    #[sql_type = "BigInt"]
    pub answered_dialogue_count: i64,
}

#[Object]
impl PuzzleParticipant {
    async fn id(&self) -> ID {
        self.id
    }
    async fn nickname(&self) -> &str {
        &self.nickname
    }
    async fn true_answer(&self) -> bool {
        self.true_answer
    }
    async fn dialogue_count(&self) -> i64 {
        self.dialogue_count
    }
    async fn answered_dialogue_count(&self) -> i64 {
        self.answered_dialogue_count
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
#[derive(Queryable, Identifiable, Clone, Debug)]
#[table_name = "puzzle"]
pub struct Puzzle {
    pub id: ID,
    pub title: String,
    pub yami: Yami,
    pub genre: Genre,
    pub content: String,
    pub solution: String,
    pub created: Timestamptz,
    pub modified: Timestamptz,
    pub status: Status,
    pub memo: String,
    pub user_id: ID,
    pub anonymous: bool,
    pub dazed_on: Date,
    pub grotesque: bool,
}

#[Object]
impl Puzzle {
    async fn id(&self) -> ID {
        self.id
    }
    async fn title(&self) -> &str {
        &self.title
    }
    async fn yami(&self) -> Yami {
        self.yami
    }
    async fn genre(&self) -> Genre {
        self.genre
    }
    async fn content(&self) -> &str {
        &self.content
    }
    async fn solution(&self) -> &str {
        &self.solution
    }
    async fn created(&self) -> Timestamptz {
        self.created
    }
    async fn modified(&self) -> Timestamptz {
        self.modified
    }
    async fn status(&self) -> Status {
        self.status
    }
    async fn memo(&self) -> &str {
        &self.memo
    }
    async fn user_id(&self) -> ID {
        self.user_id
    }
    async fn anonymous(&self) -> bool {
        self.anonymous
    }
    async fn dazed_on(&self) -> Date {
        self.dazed_on
    }
    async fn grotesque(&self) -> bool {
        self.grotesque
    }

    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        use crate::schema::user;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user = user::table
            .filter(user::id.eq(self.user_id))
            .limit(1)
            .first(&conn)?;

        Ok(user)
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
                filter.puzzle_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| BookmarkFilter {
                puzzle_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = BookmarkQuery::default();
        query
            .bookmarks(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }

    async fn bookmark_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::bookmark::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = bookmark
            .filter(puzzle_id.eq(self.id))
            .count()
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
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
                filter.puzzle_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| CommentFilter {
                puzzle_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = CommentQuery::default();
        query
            .comments(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }

    async fn comment_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::comment::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = comment
            .filter(puzzle_id.eq(self.id))
            .count()
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn dialogues(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<DialogueFilter>,
        order: Option<Vec<DialogueOrder>>,
    ) -> async_graphql::Result<Vec<Dialogue>> {
        use crate::gql_schema::DialogueQuery;

        let filter = filter
            .map(|mut filter| {
                filter.puzzle_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| DialogueFilter {
                puzzle_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = DialogueQuery::default();
        query
            .dialogues(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }

    async fn dialogue_count(
        &self,
        ctx: &Context<'_>,
        answered: Option<bool>,
    ) -> async_graphql::Result<i64> {
        use crate::schema::dialogue::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = if let Some(value) = answered {
            if value {
                dialogue
                    .filter(puzzle_id.eq(self.id))
                    .filter(answeredtime.is_not_null())
                    .count()
                    .get_result(&conn)
                    .map_err(|err| async_graphql::Error::from(err))?
            } else {
                dialogue
                    .filter(puzzle_id.eq(self.id))
                    .filter(answeredtime.is_null())
                    .count()
                    .get_result(&conn)
                    .map_err(|err| async_graphql::Error::from(err))?
            }
        } else {
            dialogue
                .filter(puzzle_id.eq(self.id))
                .count()
                .get_result(&conn)
                .map_err(|err| async_graphql::Error::from(err))?
        };

        Ok(result)
    }

    async fn dialogue_max_answered_time(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<Timestamptz>> {
        use crate::schema::dialogue::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = dialogue
            .filter(puzzle_id.eq(self.id))
            .select(max(answeredtime))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn hints(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<HintFilter>,
        order: Option<Vec<HintOrder>>,
    ) -> async_graphql::Result<Vec<Hint>> {
        use crate::gql_schema::HintQuery;

        let filter = filter
            .map(|mut filter| {
                filter.puzzle_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| HintFilter {
                puzzle_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = HintQuery::default();
        query
            .hints(ctx, limit, offset, Some(vec![filter]), order)
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
                filter.puzzle_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| PuzzleTagFilter {
                puzzle_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = PuzzleTagQuery::default();
        query
            .puzzle_tags(ctx, limit, offset, Some(vec![filter]), order)
            .await
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
                filter.puzzle_id = Some(I32Filtering::eq(self.id));
                filter
            })
            .unwrap_or_else(|| StarFilter {
                puzzle_id: Some(I32Filtering::eq(self.id)),
                ..Default::default()
            });

        let query = StarQuery::default();
        query
            .stars(ctx, limit, offset, Some(vec![filter]), order)
            .await
    }

    async fn star_count(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::star::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = star
            .filter(puzzle_id.eq(self.id))
            .count()
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(result)
    }

    async fn star_sum(&self, ctx: &Context<'_>) -> async_graphql::Result<i64> {
        use crate::schema::star::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let result = star
            .filter(puzzle_id.eq(self.id))
            .select(sum(value))
            .get_result::<Option<i64>>(&conn)
            .map_err(|err| async_graphql::Error::from(err))?
            .unwrap_or(0i64);

        Ok(result)
    }
}
