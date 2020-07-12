use actix_web::{http::Cookie, web, HttpResponse, Result};
use frank_jwt::encode;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::env;
use std::path::PathBuf;
use time::{now_utc, Duration};

use crate::context::CindyContext;
use crate::models::User;

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

impl LoginResponse {
    pub fn error(&mut self, error: String) -> &mut Self {
        self.error = Some(error);
        self
    }

    pub fn data(&mut self, data: LoginResponseData) -> &mut Self {
        self.data = Some(data);
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
    ctx: web::Data<CindyContext>,
) -> Result<HttpResponse> {
    let conn = ctx.get_conn().expect("Error getting connection");
    let user: User = match User::local_auth(&item.username, &item.password, conn).await {
        Ok(user) => user,
        Err(error) => {
            return Ok(HttpResponse::Ok().json(LoginResponse::default().error(format!("{}", error))))
        }
    };

    // Expires in 30 days
    let max_age = Duration::days(30);

    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("cindy-jwt-token", get_jwt(&user))
                .expires(now_utc() + max_age)
                .max_age_time(max_age)
                .http_only(true)
                .path("/")
                .finish(),
        )
        .json(LoginResponse::default().data(LoginResponseData {
            id: user.id,
            username: user.username,
        })))
}

pub fn get_jwt(user: &User) -> String {
    let iat = now_utc().to_timespec();
    let exp = iat + Duration::days(30);
    let header = json!({
        "iat": iat.sec,
        "exp": exp.sec,
    });
    let payload = json!({
        "user": {
            "id": user.id,
            "icon": user.icon,
            "username": user.username,
            "nickname": user.nickname,
        }
    });

    if let Some(keypath) = env::var("KEYPATH").ok() {
        encode(
            header,
            &PathBuf::from(keypath),
            &payload,
            frank_jwt::Algorithm::RS256,
        )
        .expect("Error encoding jwt with RS256.")
    } else {
        let secret = env::var("SECRET").unwrap_or("CINDYTHINK_HEYRICT".to_string());
        encode(header, &secret, &payload, frank_jwt::Algorithm::HS256)
            .expect("Error encoding jwt with HS256.")
    }
}
