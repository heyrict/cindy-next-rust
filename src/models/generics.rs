use async_graphql::{self, async_trait, guard::Guard, Context, Enum, InputObject};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::{
    backend::Backend,
    expression::BoxableExpression,
    prelude::*,
    sql_types::{Bool, Nullable},
};

use crate::auth::Role;
use crate::context::RequestCtx;

/// A filter available to check raw values
pub trait RawFilter<T> {
    /// Check if item matches the filter condition
    fn check(&self, item: &T) -> bool;
}

impl<U: RawFilter<T>, T> RawFilter<T> for Vec<U> {
    fn check(&self, item: &T) -> bool {
        !self.iter().any(|u| u.check(item) == false)
    }
}

#[derive(Enum, Eq, PartialEq, Clone, Copy, Debug)]
pub enum DbOp {
    Created,
    Updated,
    Deleted,
}

#[derive(Enum, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Ordering {
    Asc,
    Desc,
    AscNullsFirst,
    AscNullsLast,
    DescNullsFirst,
    DescNullsLast,
}

#[derive(InputObject, Clone, Debug)]
pub struct StringFiltering {
    pub eq: Option<String>,
    pub like: Option<String>,
    pub ilike: Option<String>,
}

impl RawFilter<&str> for StringFiltering {
    fn check(&self, item: &&str) -> bool {
        if let Some(eq) = self.eq.as_ref() {
            item == eq
        } else {
            // TODO like && ilike unimplemented
            true
        }
    }
}

#[derive(InputObject, Clone, Debug, Eq, PartialEq)]
pub struct I32Filtering {
    pub eq: Option<i32>,
    pub gt: Option<i32>,
    pub lt: Option<i32>,
    pub ge: Option<i32>,
    pub le: Option<i32>,
}

impl RawFilter<i32> for I32Filtering {
    fn check(&self, item: &i32) -> bool {
        if let Some(eq) = self.eq.as_ref() {
            item == eq
        } else if let Some(gt) = self.gt.as_ref() {
            item > gt
        } else if let Some(lt) = self.lt.as_ref() {
            item < lt
        } else if let Some(ge) = self.ge.as_ref() {
            item >= ge
        } else if let Some(le) = self.le.as_ref() {
            item <= le
        } else {
            true
        }
    }
}

#[derive(InputObject, Clone, Debug)]
pub struct NullableI32Filtering {
    pub is_null: Option<bool>,
    pub eq: Option<i32>,
    pub gt: Option<i32>,
    pub lt: Option<i32>,
    pub ge: Option<i32>,
    pub le: Option<i32>,
}

impl RawFilter<Option<i32>> for NullableI32Filtering {
    fn check(&self, item: &Option<i32>) -> bool {
        if let Some(item) = item {
            if self.is_null == Some(true) {
                false
            } else if let Some(eq) = self.eq.as_ref() {
                item == eq
            } else if let Some(gt) = self.gt.as_ref() {
                item > gt
            } else if let Some(lt) = self.lt.as_ref() {
                item < lt
            } else if let Some(ge) = self.ge.as_ref() {
                item >= ge
            } else if let Some(le) = self.le.as_ref() {
                item <= le
            } else {
                true
            }
        } else {
            if self.is_null == Some(false)
                || self.eq.is_some()
                || self.gt.is_some()
                || self.lt.is_some()
                || self.ge.is_some()
                || self.le.is_some()
            {
                false
            } else {
                true
            }
        }
    }
}

#[derive(InputObject, Clone, Debug)]
pub struct TimestamptzFiltering {
    pub eq: Option<Timestamptz>,
    pub gt: Option<Timestamptz>,
    pub lt: Option<Timestamptz>,
    pub ge: Option<Timestamptz>,
    pub le: Option<Timestamptz>,
}

impl RawFilter<Timestamptz> for TimestamptzFiltering {
    fn check(&self, item: &Timestamptz) -> bool {
        if let Some(eq) = self.eq.as_ref() {
            item == eq
        } else if let Some(gt) = self.gt.as_ref() {
            item > gt
        } else if let Some(lt) = self.lt.as_ref() {
            item < lt
        } else if let Some(ge) = self.ge.as_ref() {
            item >= ge
        } else if let Some(le) = self.le.as_ref() {
            item <= le
        } else {
            true
        }
    }
}

#[derive(InputObject, Clone, Debug)]
pub struct DateFiltering {
    pub eq: Option<Date>,
    pub gt: Option<Date>,
    pub lt: Option<Date>,
    pub ge: Option<Date>,
    pub le: Option<Date>,
}

impl RawFilter<Date> for DateFiltering {
    fn check(&self, item: &Date) -> bool {
        if let Some(eq) = self.eq.as_ref() {
            item == eq
        } else if let Some(gt) = self.gt.as_ref() {
            item > gt
        } else if let Some(lt) = self.lt.as_ref() {
            item < lt
        } else if let Some(ge) = self.ge.as_ref() {
            item >= ge
        } else if let Some(le) = self.le.as_ref() {
            item <= le
        } else {
            true
        }
    }
}

#[derive(InputObject, Clone, Debug)]
pub struct NullableTimestamptzFiltering {
    pub is_null: Option<bool>,
    pub eq: Option<Timestamptz>,
    pub gt: Option<Timestamptz>,
    pub lt: Option<Timestamptz>,
    pub ge: Option<Timestamptz>,
    pub le: Option<Timestamptz>,
}

impl RawFilter<Option<Timestamptz>> for NullableTimestamptzFiltering {
    fn check(&self, item: &Option<Timestamptz>) -> bool {
        if let Some(item) = item {
            if self.is_null == Some(true) {
                false
            } else if let Some(eq) = self.eq.as_ref() {
                item == eq
            } else if let Some(gt) = self.gt.as_ref() {
                item > gt
            } else if let Some(lt) = self.lt.as_ref() {
                item < lt
            } else if let Some(ge) = self.ge.as_ref() {
                item >= ge
            } else if let Some(le) = self.le.as_ref() {
                item <= le
            } else {
                true
            }
        } else {
            if self.is_null == Some(false)
                || self.eq.is_some()
                || self.gt.is_some()
                || self.lt.is_some()
                || self.ge.is_some()
                || self.le.is_some()
            {
                false
            } else {
                true
            }
        }
    }
}

pub type DB = diesel::pg::Pg;
pub type ID = i32;

pub type Timestamptz = DateTime<Utc>;
pub type Date = NaiveDate;

pub trait CindyFilter<Table: Send, DB> {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<Table, DB, SqlType = Nullable<Bool>> + Send>>;
}

impl<T: 'static, DB: 'static, F> CindyFilter<T, DB> for Vec<F>
where
    T: Send,
    DB: Backend,
    F: CindyFilter<T, DB>,
{
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<T, DB, SqlType = Nullable<Bool>> + Send>> {
        let mut filter: Option<Box<dyn BoxableExpression<T, DB, SqlType = Nullable<Bool>> + Send>> =
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

/// Make sure that req_value be consistent with value, otherwise throws an error.
pub fn assert_eq_guard<T: PartialEq>(a: T, b: T) -> async_graphql::Result<()> {
    if a != b {
        Err(async_graphql::Error::new("Assertion failed".to_string()))
    } else {
        Ok(())
    }
}

/// Make sure that req_value be consistent with value, otherwise throws an error message `msg`.
pub fn assert_eq_guard_msg<T: PartialEq>(
    a: T,
    b: T,
    msg: impl AsRef<str>,
) -> async_graphql::Result<()> {
    if a != b {
        Err(async_graphql::Error::new(format!(
            "Assertion failed: {}",
            msg.as_ref()
        )))
    } else {
        Ok(())
    }
}

pub struct DenyRoleGuard {
    pub role: Role,
}

#[async_trait::async_trait]
impl Guard for DenyRoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
        if let Some(reqctx) = ctx.data_opt::<RequestCtx>() {
            if reqctx.get_role() == self.role {
                Err("Forbidden: No enough privileges".into())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

/// Guard guests, limit users with same user id, allow admins
pub fn user_id_guard(ctx: &Context<'_>, user_id: ID) -> async_graphql::Result<()> {
    let role = ctx.data::<RequestCtx>()?.get_role();
    match role {
        Role::Admin => Ok(()),
        Role::User => assert_eq_guard(
            ctx.data::<RequestCtx>()?
                .get_user_id()
                .ok_or(async_graphql::Error::new("No user"))?,
            user_id,
        ),
        Role::Guest => Err(async_graphql::Error::new("Not logged in")),
    }
}

// TODO Rewrite all these macros with proc_macro

/// Generate filter for the query in a loop.
#[macro_export]
macro_rules! gen_string_filter {
    ($obj:ident, $field:ident, $filt:ident) => {
        if let Some($obj) = $obj {
            let StringFiltering { eq, like, ilike } = $obj;
            apply_filter!(eq, $field, $filt);
            apply_filter!(like, $field, $filt);
            apply_filter!(ilike, $field, $filt);
        }
    };
}

/// Generate filter for the query in a loop.
#[macro_export]
macro_rules! gen_bool_filter {
    ($obj:ident, $field:ident, $filt:ident) => {
        if let Some($obj) = $obj {
            $filt = Some(if let Some(filt_) = $filt {
                Box::new(filt_.and($field.eq($obj).nullable()))
            } else {
                Box::new($field.eq($obj).nullable())
            });
        }
    };
}

/// Generate filter for the query in a loop.
#[macro_export]
macro_rules! gen_number_filter {
    ($obj:ident: $ty:ident, $field:ident, $filt:ident) => {
        if let Some($obj) = $obj {
            let $ty { eq, gt, ge, lt, le } = $obj;
            apply_filter!(eq, $field, $filt);
            apply_filter!(gt, $field, $filt);
            apply_filter!(ge, $field, $filt);
            apply_filter!(lt, $field, $filt);
            apply_filter!(le, $field, $filt);
        }
    };
}

/// Generate filter for the query in a loop.
#[macro_export]
macro_rules! gen_nullable_number_filter {
    ($obj:ident: $ty:ident, $field:ident, $filt:ident) => {
        if let Some($obj) = $obj {
            let $ty {
                is_null,
                eq,
                gt,
                ge,
                lt,
                le,
            } = $obj;
            if let Some(is_null) = is_null {
                $filt = Some(if is_null {
                    Box::new($field.is_null().nullable())
                } else {
                    Box::new($field.is_not_null().nullable())
                });
            };
            apply_filter!(eq, $field, $filt);
            apply_filter!(gt, $field, $filt);
            apply_filter!(ge, $field, $filt);
            apply_filter!(lt, $field, $filt);
            apply_filter!(le, $field, $filt);
        }
    };
}

/// Generate filter for the query in a loop.
#[macro_export]
macro_rules! gen_enum_filter {
    ($obj:ident: $ty:ident, $field:ident, $filt:ident) => {
        if let Some($obj) = $obj {
            let $ty {
                eq,
                ne,
                eq_any,
                ne_all,
            } = $obj;
            apply_filter!(eq, $field, $filt);
            apply_filter!(ne, $field, $filt);
            // eq_any
            if let Some(eq_any) = eq_any {
                $filt = Some(if let Some(filt_) = $filt {
                    Box::new(filt_.and($field.eq(diesel::dsl::any(eq_any)).nullable()))
                } else {
                    Box::new($field.eq(diesel::dsl::any(eq_any)).nullable())
                });
            };
            // ne_all
            if let Some(ne_all) = ne_all {
                $filt = Some(if let Some(filt_) = $filt {
                    Box::new(filt_.and($field.ne(diesel::dsl::all(ne_all)).nullable()))
                } else {
                    Box::new($field.ne(diesel::dsl::all(ne_all)).nullable())
                });
            };
        }
    };
}

/// Applies the filter to the query in a loop.
///
/// Due to limitation of the query builder, grouping `or` is not possible.
/// Thus only one arguments from the second element in the array will be accepted.
#[macro_export]
macro_rules! apply_filter {
    ($obj:ident, $field:ident, $filt:ident) => {
        if let Some($obj) = $obj {
            $filt = Some(if let Some(filt_) = $filt {
                Box::new(filt_.and($field.$obj($obj).nullable()))
            } else {
                Box::new($field.$obj($obj).nullable())
            });
        };
    };
    (($ty:ty) $obj:ident, $field:ident, $filt:ident) => {
        if let Some($obj) = $obj {
            $filt = Some(if let Some(filt_) = $filt {
                Box::new(filt_.and($field.$obj($obj as $ty).nullable()))
            } else {
                Box::new($field.$obj($obj as $ty).nullable())
            });
        };
    };
}

/// Generate order_by for the query in a loop.
#[macro_export]
macro_rules! gen_order {
    ($obj:ident, $field:ident, $query:ident) => {
        if let Some(order) = $obj.$field {
            match order {
                Ordering::Asc => apply_order!($query, $field.asc()),
                Ordering::Desc => apply_order!($query, $field.desc()),
                Ordering::AscNullsFirst => apply_order!($query, $field.asc().nulls_first()),
                Ordering::DescNullsFirst => apply_order!($query, $field.desc().nulls_first()),
                Ordering::AscNullsLast => apply_order!($query, $field.asc().nulls_last()),
                Ordering::DescNullsLast => apply_order!($query, $field.desc().nulls_last()),
            }
        };
    };
}

/// Applies order_by statement to the query in a loop.
#[macro_export]
macro_rules! apply_order {
    ($query:ident, $order:expr) => {
        $query = $query.then_order_by($order);
    };
}
