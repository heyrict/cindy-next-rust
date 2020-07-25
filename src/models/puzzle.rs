use async_graphql::{
    Context, FieldResult, InputValueError, InputValueResult, Scalar, ScalarType, Value,
};
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

use crate::schema::puzzle;
use crate::context::GlobalCtx;

use super::generics::*;
use super::user::*;

/// Available orders for users query
#[async_graphql::InputObject]
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
#[async_graphql::InputObject]
pub struct PuzzleFilter {
    title: Option<StringFiltering>,
    genre: Option<GenreFiltering>,
    yami: Option<YamiFiltering>,
    content: Option<StringFiltering>,
    solution: Option<StringFiltering>,
}

/// Helper object to apply the filtering to the query
pub struct PuzzleFilters(Vec<PuzzleFilter>);

impl Default for PuzzleFilters {
    fn default() -> Self {
        Self(vec![])
    }
}

impl PuzzleFilters {
    pub fn new(orders: Vec<PuzzleFilter>) -> Self {
        Self(orders)
    }

    pub fn apply_filter<'a>(
        self,
        query_dsl: crate::schema::puzzle::BoxedQuery<'a, DB>,
    ) -> crate::schema::puzzle::BoxedQuery<'a, DB> {
        use crate::schema::puzzle::dsl::*;

        let mut query = query_dsl;

        for (index, obj) in self.0.into_iter().enumerate() {
            let PuzzleFilter {
                title: obj_title,
                genre: obj_genre,
                yami: obj_yami,
                content: obj_content,
                solution: obj_solution,
            } = obj;
            gen_string_filter!(obj_title, title, query, index);
            gen_enum_filter!(obj_genre: GenreFiltering, genre, query, index);
            gen_enum_filter!(obj_yami: YamiFiltering, yami, query, index);
            gen_string_filter!(obj_content, content, query, index);
            gen_string_filter!(obj_solution, solution, query, index);
        }

        query
    }
}

#[async_graphql::InputObject]
pub struct YamiFiltering {
    pub eq: Option<Yami>,
    pub ne: Option<Yami>,
    pub eq_any: Option<Vec<Yami>>,
}

#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow)]
pub enum Yami {
    None = 0,
    Normal = 1,
    Longterm = 2,
}

#[Scalar]
impl ScalarType for Yami {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::Int(value) = value {
            match value {
                0 => Ok(Yami::None),
                1 => Ok(Yami::Normal),
                2 => Ok(Yami::Longterm),
                _ => Err(InputValueError::Custom(format!(
                    "Expect Yami in 0-2, found `{}`",
                    value
                ))),
            }
        } else {
            Err(InputValueError::ExpectedType(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::Int(*self as i32)
    }
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

#[async_graphql::InputObject]
pub struct GenreFiltering {
    pub eq: Option<Genre>,
    pub ne: Option<Genre>,
    pub eq_any: Option<Vec<Genre>>,
}

#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow)]
pub enum Genre {
    Classic = 0,
    TwentyQuestions = 1,
    LittleAlbat = 2,
    Others = 3,
}

#[Scalar]
impl ScalarType for Genre {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::Int(value) = value {
            match value {
                0 => Ok(Genre::Classic),
                1 => Ok(Genre::TwentyQuestions),
                2 => Ok(Genre::LittleAlbat),
                3 => Ok(Genre::Others),
                _ => Err(InputValueError::Custom(format!(
                    "Expect Genre in 0-3, found `{}`",
                    value
                ))),
            }
        } else {
            Err(InputValueError::ExpectedType(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::Int(*self as i32)
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

/// Object for user table
#[derive(Queryable, Identifiable, Debug)]
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
    pub status: i32,
    pub memo: String,
    pub user_id: i32,
    pub anonymous: bool,
    pub dazed_on: Date,
    pub grotesque: bool,
}

#[async_graphql::Object]
impl Puzzle {
    async fn id(&self) -> ID {
        self.id
    }
    async fn title(&self) -> &str {
        &self.title
    }
    async fn yami(&self) -> i32 {
        self.yami as i32
    }
    async fn genre(&self) -> i32 {
        self.genre as i32
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
    async fn status(&self) -> i32 {
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

    async fn user(&self, ctx: &Context<'_>) -> FieldResult<User> {
        use crate::schema::user;

        let conn = ctx.data::<GlobalCtx>().get_conn()?;

        let user = user::table
            .filter(user::id.eq(self.user_id))
            .limit(1)
            .first(&conn)?;

        Ok(user)
    }
}
