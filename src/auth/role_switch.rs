use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};

use crate::context::RequestCtx;

use super::{AuthResponse, Role};

#[derive(Deserialize)]
pub struct RoleBody {
    role: String,
}

#[derive(Serialize, Default)]
pub struct RoleResponse {
    error: Option<String>,
    data: Option<RoleResponseData>,
}

impl AuthResponse for RoleResponse {
    type Data = RoleResponseData;
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
pub struct RoleResponseData {
    auth_token: String,
}

pub async fn role_switch(item: web::Json<RoleBody>, req: HttpRequest) -> Result<HttpResponse> {
    let headers = req.headers();
    let connection_info = req.connection_info();
    let ip_addr = if let Some(header_real_ip) = dotenv::var("HEADER_REAL_IP").ok() {
        headers
            .get(header_real_ip)
            .and_then(|ip| ip.to_str().ok())
            .or_else(|| connection_info.peer_addr())
    } else {
        connection_info.peer_addr()
    };

    // Authorization info
    let token = headers.get("Authorization").and_then(|value| {
        value
            .to_str()
            .ok()
            // Drop `Bearer `
            .and_then(|v| v.splitn(2, ' ').nth(1))
            .map(|v| v.to_string())
    });
    let admin_secret = headers
        .get("X-CINDY-ADMIN-SECRET")
        .and_then(|value| value.to_str().map(|v| v.to_owned()).ok());
    let ctx = RequestCtx::default()
        .with_token(token)
        .with_secret(admin_secret);
    let user = ctx.get_user();
    let role = ctx.get_role();

    // Logging
    if let Some(user) = user {
        info!(
            "({}) /roleSwitch: {}<{}:{}>",
            ip_addr.unwrap_or_default(),
            &role,
            &user.id,
            &user.nickname
        );
    } else {
        info!(
            "({}) /roleSwitch: {}<?>",
            ip_addr.unwrap_or_default(),
            &role,
        );
    }

    let jwt = ctx.switch_role(Role::from(item.role.as_ref()))?;

    Ok(HttpResponse::Ok()
        //.cookie(gen_cookie(&user))
        .json(RoleResponse::default().data(RoleResponseData { auth_token: jwt })))
}
