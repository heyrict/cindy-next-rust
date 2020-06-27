use anyhow::{Context, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::db::{establish_connection, DbPool};

pub struct CindyContext {
    pool: DbPool,
    token: Option<String>,
}

impl Default for CindyContext {
    fn default() -> Self {
        let pool = establish_connection();

        Self { pool, token: None }
    }
}

impl CindyContext {
    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>> {
        let conn = self
            .pool
            .get()
            .context("Error getting connection to the database")?;
        Ok(conn)
    }
}
