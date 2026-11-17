//! Postgres store for Loom.
//!
//! SQL is executed with typed parameters and typed row structs. Compile-time
//! checked macros are deferred until sqlx-cli and sqlx crate versions match;
//! D01 in docs/DECISIONS.md records the reason.

use sqlx::{PgPool, Pool, Postgres};

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let pool = Pool::<Postgres>::connect(url).await?;
        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::raw_sql(include_str!("../migrations/0001_baseline.sql"))
            .execute(&self.pool)
            .await?;
        sqlx::raw_sql(include_str!("../migrations/0002_feed_window.sql"))
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

pub mod bundles;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("not found")]
    NotFound,
    #[error("authentication required")]
    Auth,
}

pub mod tags;
