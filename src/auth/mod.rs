use actix_web::{cookie::Cookie, HttpResponse, Result};
use frank_jwt::encode;
use serde::Serialize;
use std::env;
use std::path::PathBuf;
use time::{now_utc, Duration};

use crate::models::User;

mod login;
mod signup;

pub use login::login;
pub use signup::signup;

pub trait AuthResponse {
    type Data;

    fn data(&mut self, data: Self::Data) -> &mut Self;
    fn error(&mut self, error: String) -> &mut Self;
}

const DEFAULT_SECRET: &'static str = "CINDYTHINK_HEYRICT";

fn get_jwt(user: &User) -> String {
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
        let secret = env::var("SECRET").unwrap_or(DEFAULT_SECRET.to_string());
        encode(header, &secret, &payload, frank_jwt::Algorithm::HS256)
            .expect("Error encoding jwt with HS256.")
    }
}

fn error_response<T, E>(error: E) -> Result<HttpResponse>
where
    T: Default + AuthResponse + Serialize,
    E: Into<String>,
{
    Ok(HttpResponse::BadRequest().json(T::default().error(error.into())))
}

fn gen_cookie(user: &User) -> Cookie {
    // Expires in 30 days
    let max_age = Duration::days(30);

    Cookie::build("cindy-jwt-token", get_jwt(user))
        .expires(now_utc() + max_age)
        .max_age_time(max_age)
        .http_only(true)
        .path("/")
        .finish()
}
