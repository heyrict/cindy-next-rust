use actix_web::{cookie::Cookie, HttpResponse, Result};
use frank_jwt::{decode, encode, Algorithm, ValidationOptions};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use time::{Duration, OffsetDateTime};

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

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Role {
    Guest,
    User,
    Admin,
}

impl From<&str> for Role {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "user" => Role::User,
            "admin" => Role::Admin,
            _ => Role::Guest,
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Role::Guest => "Guest",
                Role::User => "User",
                Role::Admin => "Admin",
            }
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct JwtPayloadUser {
    id: crate::models::ID,
    icon: Option<String>,
    username: String,
    nickname: String,
}

#[derive(Deserialize, Debug)]
pub struct JwtPayload {
    user: JwtPayloadUser,
    role: Role,
}

impl JwtPayload {
    pub fn get_user(&self) -> &JwtPayloadUser {
        &self.user
    }

    pub fn get_role(&self) -> &Role {
        &self.role
    }

    pub fn get_user_id(&self) -> crate::models::ID {
        self.user.id
    }
}

pub fn parse_jwt(token: &str) -> Result<JwtPayload, anyhow::Error> {
    let result = if let Some(keypath) = dotenv::var("KEYPATH").ok() {
        decode(
            &token,
            &PathBuf::from(keypath),
            Algorithm::RS256,
            &ValidationOptions::default(),
        )
    } else {
        let secret = dotenv::var("SECRET").unwrap_or(DEFAULT_SECRET.to_string());
        decode(
            &token,
            &secret,
            Algorithm::HS256,
            &ValidationOptions::default(),
        )
    };
    result
        .map(|(_, payload)| payload)
        .map_err(anyhow::Error::from)
        .and_then(|val| serde_json::from_value(val).map_err(anyhow::Error::from))
}

fn get_jwt(user: &User) -> String {
    let iat = OffsetDateTime::now_utc();
    let exp: OffsetDateTime = iat + Duration::days(30);
    let header = json!({});
    let payload = json!({
        "iat": iat.timestamp(),
        "exp": exp.timestamp(),
        "user": {
            "id": user.id,
            "icon": user.icon,
            "username": user.username,
            "nickname": user.nickname,
        },
        "role": Role::User
    });

    if let Some(keypath) = dotenv::var("KEYPATH").ok() {
        encode(header, &PathBuf::from(keypath), &payload, Algorithm::RS256)
            .expect("Error encoding jwt with RS256.")
    } else {
        let secret = dotenv::var("SECRET").unwrap_or(DEFAULT_SECRET.to_string());
        encode(header, &secret, &payload, Algorithm::HS256).expect("Error encoding jwt with HS256.")
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
        .expires(OffsetDateTime::now_utc() + max_age)
        .max_age(max_age)
        .http_only(true)
        .path("/")
        .finish()
}
