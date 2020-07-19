use actix_web::{web, HttpResponse, Result};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::context::CindyContext;
use crate::models::User;

use super::{error_response, gen_cookie, AuthResponse};

#[derive(Deserialize)]
pub struct SignupBody {
    nickname: String,
    username: String,
    password: String,
}

#[derive(Serialize, Default)]
pub struct SignupResponse {
    error: Option<String>,
    data: Option<SignupResponseData>,
}

impl AuthResponse for SignupResponse {
    type Data = SignupResponseData;
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
pub struct SignupResponseData {
    id: i32,
    username: String,
}

pub async fn signup(
    item: web::Json<SignupBody>,
    ctx: web::Data<CindyContext>,
) -> Result<HttpResponse> {
    use crate::schema::user;

    let username = item.username.trim();
    let nickname = item.nickname.trim();
    let password = &item.password;

    if username.is_empty() {
        return error_response::<SignupResponse, _>("Username cannot be blank!");
    }
    if nickname.is_empty() {
        return error_response::<SignupResponse, _>("Nickname cannot be blank!");
    }
    if password.is_empty() {
        return error_response::<SignupResponse, _>("Password cannot be blank!");
    }

    if username.len() >= 32 {
        return error_response::<SignupResponse, _>("Username should be at most 32 characters");
    }
    if nickname.len() >= 32 {
        return error_response::<SignupResponse, _>("Nickname should be at most 32 characters");
    }
    if password.len() < 6 {
        return error_response::<SignupResponse, _>("Password must be at least 6 characters long");
    }

    let conn = ctx.get_conn().expect("Error getting connection");

    // Sign up the user
    let credential = User::derive_credential(password);

    let user_query = diesel::insert_into(user::table)
        .values((
            user::username.eq(username),
            user::nickname.eq(nickname),
            user::password.eq(&credential),
        ))
        .get_results::<User>(&conn);

    let mut usr = match user_query {
        Ok(usr) => usr,
        Err(error) => {
            return error_response::<SignupResponse, _>(format!("{}", error));
        }
    };

    let usr = if usr.len() > 1 {
        return error_response::<SignupResponse, _>("Internal Server Error: Multiple user created");
    } else if usr.len() == 0 {
        return error_response::<SignupResponse, _>("Internal Server Error: Failed to create user");
    } else {
        usr.pop().unwrap()
    };

    Ok(HttpResponse::Ok()
        .cookie(gen_cookie(&usr))
        .json(SignupResponse::default().data(SignupResponseData {
            id: usr.id,
            username: usr.username,
        })))
}
