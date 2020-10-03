use actix_web::{web, HttpRequest, HttpResponse, Result};
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
    req: HttpRequest,
) -> Result<HttpResponse> {
    let headers = req.headers();
    let connection_info = req.connection_info();
    let ip_addr = if let Some(header_real_ip) = dotenv::var("HEADER_REAL_IP").ok() {
        headers
            .get(header_real_ip)
            .and_then(|ip| ip.to_str().ok())
            .or_else(|| connection_info.remote_addr())
    } else {
        connection_info.remote_addr()
    };

    let conn = ctx.get_conn().expect("Error getting connection");
    let user: User = match User::local_auth(&item.username, &item.password, conn).await {
        Ok(user) => user,
        Err(error) => {
            info!(
                "({}) /login: Auth failed: username = '{}'",
                ip_addr.unwrap_or_default(),
                &item.username
            );
            return error_response::<LoginResponse, _>(format!("{}", error));
        }
    };

    // Logging
    info!(
        "({}) /login: User<{}:{}>",
        ip_addr.unwrap_or_default(),
        &user.id,
        &user.nickname
    );

    Ok(HttpResponse::Ok()
        .cookie(gen_cookie(&user))
        .json(LoginResponse::default().data(LoginResponseData {
            id: user.id,
            username: user.username,
        })))
}
