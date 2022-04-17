use std::convert::TryFrom;
use std::env;

use async_graphql::Upload;
use async_graphql::{self, Context, InputObject, MaybeUndefined, Object};
use diesel::prelude::*;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use uuid::Uuid;

use crate::auth::Role;
use crate::context::{GlobalCtx, RequestCtx};
use crate::models::image::*;
use crate::models::*;
use crate::schema::image;

#[derive(Default)]
pub struct ImageQuery;
#[derive(Default)]
pub struct ImageMutation;

#[Object]
impl ImageQuery {
    pub async fn image(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Image> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let image = image::table.filter(image::id.eq(id)).first(&conn)?;

        Ok(image)
    }

    pub async fn images(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Vec<ImageFilter>>,
        order: Option<Vec<ImageOrder>>,
    ) -> async_graphql::Result<Vec<Image>> {
        use crate::schema::image::dsl::*;

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;

        let mut query = image.into_boxed();
        if let Some(order) = order {
            query = ImageOrders::new(order).apply_order(query);
        }
        if let Some(filter) = filter {
            if let Some(filter_exp) = filter.as_expression() {
                query = query.filter(filter_exp)
            }
        }
        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let images = query.load::<Image>(&conn)?;

        Ok(images)
    }
}

#[derive(InputObject)]
pub struct CreateImageInput {
    pub id: Option<Uuid>,
    pub user_id: Option<ID>,
    pub puzzle_id: MaybeUndefined<ID>,
    pub created: Option<Timestamptz>,
    #[graphql(visible = false)]
    pub content_type: Option<String>,
}

impl CreateImageInput {
    pub fn set_user_id(mut self, user_id: Option<i32>) -> Self {
        self.user_id = user_id;
        self
    }
    pub fn set_content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }
}

#[derive(InputObject)]
pub struct UploadImageInput {
    pub file: Upload,
    pub user_id: Option<ID>,
    pub puzzle_id: Option<ID>,
    #[graphql(visible = false)]
    pub content_type: Option<String>,
}

impl UploadImageInput {
    pub fn set_user_id(mut self, user_id: Option<i32>) -> Self {
        self.user_id = user_id;
        self
    }
}

impl TryFrom<UploadImageInput> for CreateImageData {
    type Error = async_graphql::Error;
    fn try_from(data: UploadImageInput) -> Result<Self, Self::Error> {
        Ok(Self {
            id: None,
            user_id: data.user_id,
            puzzle_id: Some(data.puzzle_id),
            created: None,
            content_type: data.content_type.ok_or(async_graphql::Error::new(
                "Failed to recognise the content type of the image",
            ))?,
        })
    }
}

#[derive(Insertable)]
#[table_name = "image"]
pub struct CreateImageData {
    pub id: Option<Uuid>,
    pub user_id: Option<ID>,
    pub puzzle_id: Option<Option<ID>>,
    pub created: Option<Timestamptz>,
    pub content_type: String,
}

impl TryFrom<CreateImageInput> for CreateImageData {
    type Error = async_graphql::Error;
    fn try_from(data: CreateImageInput) -> Result<Self, Self::Error> {
        Ok(Self {
            id: data.id,
            user_id: data.user_id,
            puzzle_id: data.puzzle_id.as_options(),
            created: data.created,
            content_type: data.content_type.ok_or(async_graphql::Error::new(
                "Failed to recognise the content type of the image",
            ))?,
        })
    }
}

#[derive(InputObject, AsChangeset)]
#[table_name = "image"]
pub struct UpdateImageInput {
    pub id: Option<Uuid>,
    pub user_id: Option<ID>,
    pub puzzle_id: Option<Option<ID>>,
    pub created: Option<Timestamptz>,
    pub content_type: Option<String>,
}

#[Object]
impl ImageMutation {
    // Update image
    // TODO Allow updating image files
    #[graphql(guard = "DenyRoleGuard::new(Role::Guest)")]
    pub async fn update_image(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        set: UpdateImageInput,
    ) -> async_graphql::Result<Image> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                // Assert that time-related are unset
                assert_eq_guard(set.created, None)?;
                // Assert user_id is set to the user
                if let Some(user_id) = set.user_id {
                    user_id_guard(ctx, user_id)?;
                };
                // Assert that puzzle_id used to be unset
                let image_inst: Image = image::table.find(id).first(&conn)?;
                if let Some(puzzle_id) = image_inst.puzzle_id {
                    use crate::schema::puzzle;
                    let puzzle_inst: Puzzle = puzzle::table.find(puzzle_id).first(&conn)?;
                    assert_eq_guard(puzzle_inst.status, Status::Undergoing)?;
                }
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let image: Image = diesel::update(image::table)
            .filter(image::id.eq(id))
            .set(set)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(image)
    }

    // Create image metadata (only for admin use)
    #[graphql(guard = "DenyRoleGuard::new(Role::User).and(DenyRoleGuard::new(Role::Guest))")]
    pub async fn create_image(
        &self,
        ctx: &Context<'_>,
        data: CreateImageInput,
    ) -> async_graphql::Result<Image> {
        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let user_id = reqctx.get_user_id();
        let role = reqctx.get_role();

        let insert_data = match role {
            Role::User => {
                // Assert that time-related are unset
                assert_eq_guard(data.created, None)?;
                // Assert user_id is set to the user
                let insert_data = if let Some(user_id) = data.user_id {
                    user_id_guard(ctx, user_id)?;
                    CreateImageData::try_from(data)?
                } else {
                    CreateImageData::try_from(data.set_user_id(user_id))?
                };

                insert_data
            }
            Role::Staff | Role::Admin => CreateImageData::try_from(data)?,
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        let image: Image = diesel::insert_into(image::table)
            .values(&insert_data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        Ok(image)
    }

    // Delete image
    #[graphql(guard = "DenyRoleGuard::new(Role::Guest)")]
    pub async fn delete_image(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Image> {
        dotenv::dotenv().ok();

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let role = reqctx.get_role();

        match role {
            Role::User => {
                let user_id = reqctx
                    .get_user_id()
                    .ok_or(async_graphql::Error::new("No user"))?;
                // User should be the owner of the image
                let image_inst: Image = image::table.find(id).first(&conn)?;
                assert_eq_guard(image_inst.user_id, user_id)?;
                assert_eq_guard(image_inst.puzzle_id, None)?;
            }
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
            _ => {}
        };

        let image: Image = diesel::delete(image::table.filter(image::id.eq(id)))
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        image.delete_file().await?;

        Ok(image)
    }

    // Upload image
    #[graphql(guard = "DenyRoleGuard::new(Role::Guest)")]
    pub async fn upload_image(
        &self,
        ctx: &Context<'_>,
        mut data: UploadImageInput,
    ) -> async_graphql::Result<Image> {
        dotenv::dotenv().ok();
        let default_content_type: String = String::from("image/jpeg");

        let conn = ctx.data::<GlobalCtx>()?.get_conn()?;
        let reqctx = ctx.data::<RequestCtx>()?;
        let user_id = reqctx.get_user_id();
        let role = reqctx.get_role();

        // Limit file size
        let image_value = data.file.value(&ctx)?;
        let image_type = image_value
            .content_type
            .as_ref()
            .unwrap_or(&default_content_type);
        let max_filesize = env::var("IMAGE_MAXSIZE")
            .unwrap_or("1048576".to_owned())
            .parse()
            .expect("Unexpected value for `IMAGE_MAXSIZE`");
        let image_size = image_value.size()?;
        if image_size > max_filesize {
            return Err(async_graphql::Error::new(format!(
                "Max file size exceeded. Expected < {}KB, found {}KB.",
                max_filesize / 1024,
                image_size / 1024,
            )));
        }

        data.content_type = Some(image_type.to_owned());
        let insert_data = match role {
            Role::User => {
                // Assert user_id is set to the user
                let insert_data = if let Some(user_id) = data.user_id {
                    user_id_guard(ctx, user_id)?;
                    CreateImageData::try_from(data)?
                } else {
                    CreateImageData::try_from(data.set_user_id(user_id))?
                };

                insert_data
            }
            Role::Staff | Role::Admin => CreateImageData::try_from(data)?,
            Role::Guest => return Err(async_graphql::Error::new("User not logged in")),
        };

        // Update database
        let image: Image = diesel::insert_into(image::table)
            .values(&insert_data)
            .get_result(&conn)
            .map_err(|err| async_graphql::Error::from(err))?;

        // Upload image
        let upload_dir: std::path::PathBuf = env::var("UPLOAD_FOLDER")
            .unwrap_or("upload_images".to_owned())
            .into();
        let upload_file = upload_dir.join(format!(
            "{}.{}",
            image
                .id
                .to_hyphenated_ref()
                .encode_lower(&mut Uuid::encode_buffer()),
            &image.ext()
        ));
        if !upload_dir.exists() {
            tokio::fs::create_dir_all(&upload_dir)
                .await
                .expect(&format!("Unable to create directory `{:?}`", &upload_dir));
        };
        let mut file = tokio::fs::File::create(&upload_file).await?;
        tokio::io::copy(&mut image_value.into_async_read().compat(), &mut file).await?;

        Ok(image)
    }
}
