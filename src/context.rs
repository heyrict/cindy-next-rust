use anyhow::{Context, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};

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

pub struct RequestCtx {
    token: Option<String>,
}

impl Default for RequestCtx {
    fn default() -> Self {
        Self { token: None }
    }
}

impl RequestCtx {
    pub fn with_token(mut self, token: Option<String>) -> Self {
        self.token = token;
        self
    }
}
