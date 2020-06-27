use chrono::{NaiveDate, NaiveDateTime};

#[async_graphql::Enum]
pub enum Ordering {
    Asc,
    Desc,
    AscNullsFirst,
    AscNullsLast,
    DescNullsFirst,
    DescNullsLast,
}

pub type DB = diesel::pg::Pg;
pub type ID = i32;

pub type Timestamptz = NaiveDateTime;
pub type Date = NaiveDate;

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
