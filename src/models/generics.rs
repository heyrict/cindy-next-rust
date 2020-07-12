use chrono::{DateTime, Utc};

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

pub type DB = diesel::pg::Pg;
pub type ID = i32;

pub type Timestamptz = DateTime<Utc>;

// TODO Rewrite all these macros with proc_macro

/// Generate filter for the query in a loop.
macro_rules! gen_string_filter {
    ($obj:ident, $field:ident, $query:ident, $index:ident) => {
        if let Some($obj) = $obj {
            let StringFiltering { eq, like, ilike } = $obj;
            apply_filter!(eq, $field, $query, $index);
            apply_filter!(like, $field, $query, $index);
            apply_filter!(ilike, $field, $query, $index);
        }
    };
}

/// Applies the filter to the query in a loop.
///
/// Due to limitation of the query builder, grouping `or` is not possible.
/// Thus only one arguments from the second element in the array will be accepted.
macro_rules! apply_filter {
    ($obj:ident, $field:ident, $query:ident, $index:ident) => {
        if let Some($obj) = $obj {
            if $index == 0 {
                $query = $query.filter($field.$obj($obj));
            } else {
                $query = $query.or_filter($field.$obj($obj));
                continue;
            }
        }
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
