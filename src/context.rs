use anyhow::{Context, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use super::ADMIN_SECRET;
use crate::auth::{parse_jwt, JwtPayload, JwtPayloadUser, Role};
use crate::db::{establish_connection, DbPool};

#[derive(Clone)]
pub struct GlobalCtx {
    pool: DbPool,
}

impl Default for GlobalCtx {
    fn default() -> Self {
        let pool = establish_connection();

        Self { pool }
    }
}

impl GlobalCtx {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>> {
        let conn = self
            .pool
            .get()
            .context("Error getting connection to the database")?;
        Ok(conn)
    }
}

#[derive(Default)]
pub struct RequestCtx {
    jwt_payload: Option<JwtPayload>,
    admin_secret: Option<String>,
}

impl RequestCtx {
    pub fn with_token(mut self, token: Option<String>) -> Self {
        self.jwt_payload = token.and_then(|token| parse_jwt(&token).ok());
        self
    }

    pub fn with_secret(mut self, secret: Option<String>) -> Self {
        self.admin_secret = secret;
        self
    }

    pub fn get_role(&self) -> Role {
        if self.admin_secret.as_ref() == Some(&ADMIN_SECRET) {
            Role::Admin
        } else if let Some(jwt) = self.jwt_payload.as_ref() {
            *jwt.get_role()
        } else {
            Role::Guest
        }
    }

    pub fn get_user(&self) -> Option<&JwtPayloadUser> {
        self.jwt_payload.as_ref().map(|jwt| jwt.get_user())
    }

    pub fn get_user_id(&self) -> Option<crate::models::ID> {
        self.jwt_payload.as_ref().map(|jwt| jwt.get_user_id())
    }
}
