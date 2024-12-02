use std::sync::Arc;

use sqlx::pool::PoolOptions;

// Database connection to PostgreSQL + type definitions for database types.

pub type DbError = sqlx::Error;
pub type DbPool = sqlx::PgPool;

pub type DatabaseConnectionRef = Arc<DatabaseConnection>;

pub struct DatabaseConnection {
    pool: DbPool,
}

impl DatabaseConnection {
    pub fn connect(url: &str) -> Result<Self, DbError> {
        Ok(Self {
            pool: PoolOptions::new().connect_lazy(url)?,
        })
    }

    pub fn as_pool(&self) -> &DbPool {
        &self.pool
    }
}
