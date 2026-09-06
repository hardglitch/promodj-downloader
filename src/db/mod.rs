pub mod dbcore;

use crate::db::dbcore::{DBType, Database};
use sqlx_core::pool::PoolConnection;
use std::collections::HashMap;
use log::log;
use sqlx_core::sql_str::AssertSqlSafe;

impl Database {
    pub async fn create_tables(&self) -> Option<()> {
        let tx = async move |mut conn: PoolConnection<DBType>| -> Result<(), sqlx::Error> {
            sqlx::query("CREATE TABLE IF NOT EXISTS file_history(link TEXT NOT NULL UNIQUE)")
                .execute(&mut *conn).await?;
            Ok(())
        };
        self.call(tx).await
    }

    pub async fn write_file_history(&self, link: &str) -> Option<()> {
        let tx = async move |mut conn: PoolConnection<DBType>| -> Result<(), sqlx::Error> {
            let link = link
                .rsplit_once('.')
                .map(|(name, _ext)| name)
                .unwrap_or_default()
                .chars()
                .take(1000)
                .collect::<String>();

            if link.is_empty() {
                log!("Invalid link to insert into database");
                return Ok(())
            }

            sqlx::query("INSERT INTO file_history VALUES(?)")
                .bind(link)
                .execute(&mut *conn)
                .await?;
            Ok(())
        };
        self.call(tx).await
    }

    pub async fn filter_by_history(&self, unique_links: &HashMap<&str, &str>) -> Option<Vec<String>> {
        let tx = async move |mut conn: PoolConnection<DBType>| -> Result<Vec<String>, sqlx::Error> {

            // 1. Collect all the link values into a vector
            let links: Vec<&str> = unique_links.keys().cloned().collect();

            // 2. Create the placeholders string (?, ?, ?, ...)
            // We need one '?' for every link in the set.
            let placeholders = links.iter().map(|_| "?").collect::<Vec<&str>>().join(",");

            // 3. Construct the final SQL query
            let query = format!("SELECT link FROM file_history WHERE link IN ({placeholders})");

            // 4. Execute the query using sqlx::query_as
            // We select the 'link' column. If a link exists, it will be returned.
            let mut query = sqlx::query_scalar::<_, String>(AssertSqlSafe(query));
            for link in links.iter() {
                query = query.bind(link.to_owned());
            }
            let not_unique_links = query.fetch_all(&mut *conn).await?;

            // 5. Delete not unique links
            let links = unique_links.iter()
                .filter(|(name, _ext)| !not_unique_links.contains(&name.to_string()))
                .map(|(name, ext)| { format!("{name}.{ext}") })
                .collect::<Vec<String>>();

            Ok(links)
        };
        self.call(tx).await
    }
}