use sqlx_core::migrate::MigrateDatabase;
use sqlx_core::pool::{Pool, PoolConnection};
use crate::log;
use std::io::Write;

pub const DB_NAME: &str = "history.db";
pub type DBType = sqlx::Sqlite;

#[derive(Clone)]
pub struct Database {
    pool: Pool<DBType>,
}
unsafe impl Send for Database {}
unsafe impl Sync for Database {}

impl Database {
    async fn open(addr: &str) -> Option<Self> {
        match DBType::database_exists(addr).await {
            Ok(is) => {
                if is && let Some(pool) = Self::pool(addr).await {
                    Some(Self { pool })
                } else {
                    log!("Failed to open the '{addr}' database");
                    None
                }
            },
            Err(e) => {
                log!("The Database '{}' doesn't exist: {}", addr, e);
                None
            },
        }
    }

    pub async fn create_or_open(addr: &str) -> Option<Database> {
        if !DBType::database_exists(addr).await.unwrap_or(false) &&
           let Err(e) = DBType::create_database(addr).await
        {
            log!("Failed to create the '{}' database: {}", addr, e);
            std::process::exit(1);
        }
        Self::open(addr).await
    }

    async fn pool(addr: &str) -> Option<Pool<DBType>> {
        match Pool::<DBType>::connect(addr).await {
            Ok(pool) => Some(pool),
            Err(e) => {
                log!("Could not establish connection with the '{}' database: {}", addr, e);
                None
            }
        }
    }

    pub async fn call<F, R>(&self, func: F) -> Option<R>
    where
        F: (AsyncFnOnce(PoolConnection<DBType>) -> Result<R, sqlx::Error>) + Sized + Send + Sync,
        R: Send + Sync
    {
        match self.pool.acquire().await {
            Ok(conn) => {
                match func(conn,).await {
                    Ok(r) => { Some(r) },
                    Err(e) => {
                        log!("Database error: {e}");
                        None
                    },
                }
            },
            Err(e) => {
                log!("Failed to connect to the Database: {e}");
                None
            },
        }
    }
}
