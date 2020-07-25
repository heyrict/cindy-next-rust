use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

use crate::context::GlobalCtx;
use crate::models::User;

use super::{error_response, gen_cookie, AuthResponse};

#[derive(Deserialize)]
pub struct LoginBody {
    username: String,
    password: String,
}

#[derive(Serialize, Default)]
pub struct LoginResponse {
    error: Option<String>,
    data: Option<LoginResponseData>,
}

impl AuthResponse for LoginResponse {
    type Data = LoginResponseData;
    fn data(&mut self, data: Self::Data) -> &mut Self {
        self.data = Some(data);
        self
    }
    fn error(&mut self, error: String) -> &mut Self {
        self.error = Some(error);
        self
    }
}

#[derive(Serialize)]
pub struct LoginResponseData {
    id: i32,
    username: String,
}

pub async fn login(
    item: web::Json<LoginBody>,
    ctx: web::Data<GlobalCtx>,
) -> Result<HttpResponse> {
    let conn = ctx.get_conn().expect("Error getting connection");
    let user: User = match User::local_auth(&item.username, &item.password, conn).await {
        Ok(user) => user,
        Err(error) => {
            return error_response::<LoginResponse, _>(format!("{}", error));
        }
    };

    Ok(HttpResponse::Ok()
        .cookie(gen_cookie(&user))
        .json(LoginResponse::default().data(LoginResponseData {
            id: user.id,
            username: user.username,
        })))
}

