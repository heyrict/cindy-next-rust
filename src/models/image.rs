use async_graphql::{self, Context, InputObject, Object};
use diesel::{prelude::*, query_dsl::QueryDsl, sql_types::Bool};
use std::env;
use uuid::Uuid;

use crate::context::GlobalCtx;
use crate::schema::image;

use super::*;

/// Available orders for image query
#[derive(InputObject, Clone)]
pub struct ImageOrder {
    created: Option<Ordering>,
}

/// Helper object to apply the order to the query
pub struct ImageOrders(Vec<ImageOrder>);

impl Default for ImageOrders {
    fn default() -> Self {
        Self(vec![])
    }
}

impl ImageOrders {
    pub fn new(orders: Vec<ImageOrder>) -> Self {
        Self(orders)
    }

    pub fn apply_order<'a>(
        self,
        query_dsl: image::BoxedQuery<'a, DB>,
    ) -> image::BoxedQuery<'a, DB> {
        use crate::schema::image::dsl::*;

        let mut query = query_dsl;

        for obj in self.0 {
            gen_order!(obj, created, query);
        }

        query
    }
}

/// Available filters for image query
#[derive(InputObject, Clone)]
pub struct ImageFilter {
    user_id: Option<I32Filtering>,
    puzzle_id: Option<NullableI32Filtering>,
    created: Option<TimestamptzFiltering>,
}

impl CindyFilter<image::table> for ImageFilter {
    fn as_expression(
        self,
    ) -> Option<Box<dyn BoxableExpression<image::table, DB, SqlType = Bool> + Send>> {
        use crate::schema::image::dsl::*;

        let mut filter: Option<Box<dyn BoxableExpression<image, DB, SqlType = Bool> + Send>> = None;
        let ImageFilter {
            user_id: obj_user_id,
            puzzle_id: obj_puzzle_id,
            created: obj_created,
        } = self;
        gen_number_filter!(obj_user_id: I32Filtering, user_id, filter);
        gen_nullable_number_filter!(obj_puzzle_id: NullableI32Filtering, puzzle_id, filter);
        gen_number_filter!(obj_created: TimestamptzFiltering, created, filter);
        filter
    }
}

/// Object for image table
#[derive(Queryable, Identifiable, Clone, Debug)]
#[diesel(table_name = image)]
pub struct Image {
    pub id: Uuid,
    pub user_id: ID,
    pub puzzle_id: Option<ID>,
    pub created: Timestamptz,
    pub content_type: String,
}

impl Image {
    pub fn ext(&self) -> &str {
        match self.content_type.as_ref() {
            "image/png" => "png",
            "image/gif" => "gif",
            "image/svg+xml" => "svg",
            "image/tiff" => "tiff",
            _ => "jpg",
        }
    }

    pub async fn delete_file(&self) -> async_graphql::Result<()> {
        // Delete image file
        dotenv::dotenv().ok();
        let upload_dir: std::path::PathBuf = env::var("UPLOAD_FOLDER")
            .unwrap_or("upload_images".to_owned())
            .into();
        let upload_file = upload_dir.join(format!(
            "{}.{}",
            self.id
                .to_hyphenated_ref()
                .encode_lower(&mut Uuid::encode_buffer()),
            &self.ext()
        ));
        if upload_file.exists() {
            tokio::fs::remove_file(&upload_file).await?;
        };

        Ok(())
    }
}

#[Object]
impl Image {
    async fn id(&self) -> Uuid {
        self.id
    }
    async fn user_id(&self) -> ID {
        self.user_id
    }
    async fn puzzle_id(&self) -> Option<ID> {
        self.puzzle_id
    }
    async fn created(&self) -> Timestamptz {
        self.created
    }
    async fn content_type(&self) -> &str {
        self.content_type.as_ref()
    }

    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        use crate::schema::user;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let user = user::table
            .filter(user::id.eq(self.user_id))
            .limit(1)
            .first(&mut conn)?;

        Ok(user)
    }

    async fn puzzle(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Puzzle>> {
        use crate::schema::puzzle;

        let mut conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        self.puzzle_id
            .map(|puzzle_id| puzzle::table.find(puzzle_id).first(&mut conn))
            .transpose()
            .map_err(|err| async_graphql::ServerError::new(err.to_string(), None).into())
    }
}
