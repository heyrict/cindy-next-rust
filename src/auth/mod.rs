use actix_web::cookie::time::{Duration, OffsetDateTime};
use actix_web::{cookie::Cookie, HttpResponse, Result};
use frank_jwt::{decode, encode, Algorithm, ValidationOptions};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

use crate::models::User;

mod login;
mod role_switch;
mod signup;

pub use login::login;
pub use role_switch::role_switch;
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
    Staff,
    Admin,
}

impl From<&str> for Role {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "user" => Role::User,
            "admin" => Role::Admin,
            "staff" => Role::Staff,
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
                Role::Staff => "Staff",
                Role::Admin => "Admin",
            }
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct JwtPayloadUser {
    pub id: crate::models::ID,
    pub icon: Option<String>,
    pub username: String,
    pub nickname: String,
}

#[derive(Deserialize, Debug)]
pub struct JwtPayload {
    user: JwtPayloadUser,
    role: Role,
    allowed_roles: Vec<Role>,
}

impl JwtPayload {
    pub fn get_user(&self) -> &JwtPayloadUser {
        &self.user
    }

    pub fn get_role(&self) -> &Role {
        &self.role
    }

    pub fn get_roles(&self) -> &Vec<Role> {
        &self.allowed_roles
    }

    pub fn get_user_id(&self) -> crate::models::ID {
        self.user.id
    }
}

pub fn parse_jwt(token: &str) -> Result<JwtPayload, anyhow::Error> {
    let result = if let Some(keypath) = dotenv::var("PUBLIC_KEY_PATH").ok() {
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

fn get_allowed_roles(user: &User) -> Vec<Role> {
    let mut returns = vec![Role::User];
    if user.is_staff {
        returns.push(Role::Staff);
    }
    returns
}

pub fn get_jwt(user: &User, role: Option<Role>) -> String {
    let max_age = Duration::days(
        dotenv::var("LOGIN_MAX_AGE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30),
    );

    let iat = OffsetDateTime::now_utc();
    let exp: OffsetDateTime = iat + Duration::days(max_age);
    let header = json!({});
    let allowed_roles = get_allowed_roles(&user);
    let role = if let Some(role) = role {
        if allowed_roles.contains(&role) {
            role
        } else {
            Role::User
        }
    } else {
        Role::User
    };
    let payload = json!({
        "iat": iat.unix_timestamp(),
        "exp": exp.unix_timestamp(),
        "user": {
            "id": user.id,
            "icon": user.icon,
            "username": user.username,
            "nickname": user.nickname,
        },
        "role": role,
        "allowed_roles": allowed_roles
    });

    if let Some(keypath) = dotenv::var("PRIVATE_KEY_PATH").ok() {
        encode(header, &PathBuf::from(keypath), &payload, Algorithm::RS256)
            .expect("Error encoding jwt with RS256.")
    } else {
        let secret = dotenv::var("SECRET").unwrap_or(DEFAULT_SECRET.to_string());
        encode(header, &secret, &payload, Algorithm::HS256).expect("Error encoding jwt with HS256.")
    }
}

pub fn switch_jwt_role(payload: &JwtPayload, role: Role) -> String {
    let max_age = Duration::days(
        dotenv::var("LOGIN_MAX_AGE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30),
    );

    let iat = OffsetDateTime::now_utc();
    let exp: OffsetDateTime = iat + Duration::days(max_age);
    let header = json!({});
    let user = &payload.user;
    let allowed_roles = &payload.allowed_roles;
    let role = if allowed_roles.contains(&role) {
        role
    } else {
        Role::User
    };
    let payload = json!({
        "iat": iat.unix_timestamp(),
        "exp": exp.unix_timestamp(),
        "user": {
            "id": user.id,
            "icon": user.icon,
            "username": user.username,
            "nickname": user.nickname,
        },
        "role": role,
        "allowed_roles": allowed_roles
    });

    if let Some(keypath) = dotenv::var("PRIVATE_KEY_PATH").ok() {
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
    let max_age = Duration::days(
        dotenv::var("SUBSCRIPTION_MAX_CACHE_TIME")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30),
    );

    Cookie::build("cindy-jwt-token", get_jwt(user, None))
        .expires(OffsetDateTime::now_utc() + max_age)
        .max_age(max_age)
        .http_only(true)
        .path("/")
        .finish()
}
