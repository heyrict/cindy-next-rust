use chrono::{DateTime, NaiveDate, Utc};
use diesel::expression::BoxableExpression;
use diesel::sql_types::Bool;

#[async_graphql::Enum]
pub enum Ordering {
    Asc,
    Desc,
    AscNullsFirst,
    AscNullsLast,
    DescNullsFirst,
    DescNullsLast,
}

#[async_graphql::InputObject]
pub struct StringFiltering {
    pub eq: Option<String>,
    pub like: Option<String>,
    pub ilike: Option<String>,
}

#[async_graphql::InputObject]
pub struct I32Filtering {
    pub eq: Option<i32>,
    pub gt: Option<i32>,
    pub lt: Option<i32>,
    pub ge: Option<i32>,
    pub le: Option<i32>,
}

pub type DB = diesel::pg::Pg;
pub type ID = i32;

pub type Timestamptz = DateTime<Utc>;
pub type Date = NaiveDate;

pub trait CindyFilter<Table, DB> {
    fn as_expression(self) -> Option<Box<dyn BoxableExpression<Table, DB, SqlType = Bool>>>;
}

// TODO Rewrite all these macros with proc_macro

/// Generate filter for the query in a loop.
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
macro_rules! gen_number_filter {
    ($obj:ident: $ty:ident, $field:ident, $query:ident, $index:ident) => {
        if let Some($obj) = $obj {
            let $ty { eq, gt, ge, lt, le } = $obj;
            apply_filter!(eq, $field, $query, $index);
            apply_filter!(gt, $field, $query, $index);
            apply_filter!(ge, $field, $query, $index);
            apply_filter!(lt, $field, $query, $index);
            apply_filter!(le, $field, $query, $index);
        }
    };
}

/// Generate filter for the query in a loop.
macro_rules! gen_enum_filter {
    ($obj:ident: $ty:ident, $field:ident, $filt:ident) => {
        if let Some($obj) = $obj {
            let $ty {
                eq,
                ne,
                eq_any,
                ne_any,
            } = $obj;
            apply_filter!(eq, $field, $filt);
            apply_filter!(ne, $field, $filt);
            // eq_any
            if let Some(eq_any) = eq_any {
                $filt = Some(if let Some(filt_) = $filt {
                    Box::new(filt_.and($field.eq(diesel::dsl::any(eq_any))))
                } else {
                    Box::new($field.eq(diesel::dsl::any(eq_any)))
                });
            };
            // ne_any
            if let Some(ne_any) = ne_any {
                $filt = Some(if let Some(filt_) = $filt {
                    Box::new(filt_.and($field.ne(diesel::dsl::any(ne_any))))
                } else {
                    Box::new($field.ne(diesel::dsl::any(ne_any)))
                });
            };
        }
    };
}

/// Applies the filter to the query in a loop.
///
/// Due to limitation of the query builder, grouping `or` is not possible.
/// Thus only one arguments from the second element in the array will be accepted.
macro_rules! apply_filter {
    ($obj:ident, $field:ident, $filt:ident) => {
        if let Some($obj) = $obj {
            $filt = Some(if let Some(filt_) = $filt {
                Box::new(filt_.and($field.$obj($obj)))
            } else {
                Box::new($field.$obj($obj))
            });
        };
    };
    (($ty:ty) $obj:ident, $field:ident, $filt:ident) => {
        if let Some($obj) = $obj {
            $filt = Some(if let Some(filt_) = $filt {
                Box::new(filt_.and($field.$obj($obj as $ty)))
            } else {
                Box::new($field.$obj($obj as $ty))
            });
        };
    };
}

/// Generate order_by for the query in a loop.
macro_rules! gen_order {
    ($obj:ident, $field:ident, $query:ident, $flag:ident) => {
        if let Some(order) = $obj.$field {
            match order {
                Ordering::Asc => apply_order!($query, $flag, $field.asc()),
                Ordering::Desc => apply_order!($query, $flag, $field.desc()),
                Ordering::AscNullsFirst => apply_order!($query, $flag, $field.asc().nulls_first()),
                Ordering::DescNullsFirst => {
                    apply_order!($query, $flag, $field.desc().nulls_first())
                }
                Ordering::AscNullsLast => apply_order!($query, $flag, $field.asc().nulls_last()),
                Ordering::DescNullsLast => apply_order!($query, $flag, $field.desc().nulls_last()),
            }
        };
    };
}

/// Applies order_by statement to the query in a loop.
macro_rules! apply_order {
    ($query:ident, $flag:ident, $order:expr) => {
        if $flag {
            $query = ThenOrderDsl::then_order_by($query, $order);
        } else {
            $query = $query.order_by($order);
            $flag = true;
        }
    };
}
