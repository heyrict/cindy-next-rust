use async_graphql::{self, Context, Enum, InputObject, Object};
use diesel::sql_types::Bool;
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    expression::{helper_types::AsExprOf, AsExpression},
    prelude::*,
    query_dsl::methods::ThenOrderDsl,
    query_dsl::QueryDsl,
    serialize::{self, Output, ToSql},
    sql_types::Integer,
};
use std::io;

use crate::context::GlobalCtx;
use crate::schema::puzzle;

use super::generics::*;
use super::user::*;

/// Available orders for users query
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
        let mut flag = false;

        for obj in self.0 {
            gen_order!(obj, id, query, flag);
            gen_order!(obj, yami, query, flag);
            gen_order!(obj, genre, query, flag);
            gen_order!(obj, created, query, flag);
            gen_order!(obj, modified, query, flag);
            gen_order!(obj, status, query, flag);
        }

        query
    }
}

/// Available filters for users query
#[derive(InputObject, Clone)]
pub struct PuzzleFilter {
    title: Option<StringFiltering>,
    genre: Option<GenreFiltering>,
    yami: Option<YamiFiltering>,
    status: Option<StatusFiltering>,
    content: Option<StringFiltering>,
    solution: Option<StringFiltering>,
}

impl CindyFilter<puzzle::table, DB> for PuzzleFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<puzzle::table, DB, SqlType = Bool>>> {
        use crate::schema::puzzle::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<puzzle, DB, SqlType = Bool>>> = None;
        let PuzzleFilter {
            title: obj_title,
            genre: obj_genre,
            yami: obj_yami,
            status: obj_status,
            content: obj_content,
            solution: obj_solution,
        } = self;
        gen_string_filter!(obj_title, title, filter);
        gen_enum_filter!(obj_genre: GenreFiltering, genre, filter);
        gen_enum_filter!(obj_yami: YamiFiltering, yami, filter);
        gen_enum_filter!(obj_status: StatusFiltering, status, filter);
        gen_string_filter!(obj_content, content, filter);
        gen_string_filter!(obj_solution, solution, filter);
        filter
    }
}

impl CindyFilter<puzzle::table, DB> for Vec<PuzzleFilter> {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<puzzle::table, DB, SqlType = Bool>>> {
        let mut filter: Option<Box<dyn BoxableExpression<puzzle::table, DB, SqlType = Bool>>> =
            None;
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
    pub user_id: i32,
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
}
