pub mod dbcore;

use crate::db::dbcore::{DBType, Database};
use crate::log;
use sqlx_core::pool::PoolConnection;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

impl Database {
    pub async fn create_history_db(&self) -> Option<()> {
        let tx = async move |mut conn: PoolConnection<DBType>| -> Result<(), sqlx::Error> {
            let query = "CREATE TABLE IF NOT EXISTS file_history(link TEXT NOT NULL, date INTEGER NOT NULL);";
            sqlx::query(sqlx::AssertSqlSafe(query)).execute(&mut *conn).await?;
            Ok(())
        };
        self.call(tx).await
    }

    pub async fn write_file_history(&self, link: &str) -> Option<()> {
        let date =
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(d) => d.as_secs(),
                Err(e) => {
                    log!("{e}");
                    return None
                }
            };

        let tx = async move |mut conn: PoolConnection<DBType>| -> Result<(), sqlx::Error> {
            let query = "INSERT INTO file_history VALUES(?, ?);";
            sqlx::query(sqlx::AssertSqlSafe(query))
                .bind(link.chars().take(1000).collect::<String>())
                .bind(date as i64)
                .execute(&mut *conn)
                .await?;
            Ok(())
        };
        self.call(tx).await
    }

    pub async fn filter_by_history(&self, unique_links: &mut HashMap<&str, &str>) -> Option<()> {
        let tx = async move |mut conn: PoolConnection<DBType>| -> Result<(), sqlx::Error> {
            let query = "SELECT link FROM file_history LIMIT 100000;";
            let records: Vec<String> = sqlx::query_scalar(sqlx::AssertSqlSafe(query))
                .fetch_all(&mut *conn)
                .await?;
            let history_set = records.iter().map(|s| s.as_str()).collect::<HashSet<&str>>();
            let _ = unique_links.extract_if(|name, _ext| history_set.contains(name));
            Ok(())
        };
        self.call(tx).await
    }
}