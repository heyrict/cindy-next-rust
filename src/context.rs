use anyhow::{Context, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::db::{establish_connection, DbPool};

#[derive(Clone)]
pub struct CindyContext {
    pool: DbPool,
}

impl Default for CindyContext {
    fn default() -> Self {
        let pool = establish_connection();

        Self { pool }
    }
}

impl CindyContext {
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

pub struct CindyQueryContext {
    token: Option<String>,
}

impl Default for CindyQueryContext {
    fn default() -> Self {
        Self { token: None }
    }
}

impl CindyQueryContext {
    pub fn with_token(mut self, token: Option<String>) -> Self {
        self.token = token;
        self
    }
}
