use anyhow::{Context, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::auth::{parse_jwt, JwtPayload};
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
}

impl RequestCtx {
    pub fn with_token(mut self, token: Option<String>) -> Self {
        self.jwt_payload = token.and_then(|token| parse_jwt(&token).ok());
        self
    }
}
