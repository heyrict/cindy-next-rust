use anyhow::{Context, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::db::{establish_connection, DbPool};

pub struct CindyContext {
    pool: DbPool,
}

impl CindyContext {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let pool = establish_connection();

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
