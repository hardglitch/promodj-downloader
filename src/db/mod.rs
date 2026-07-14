pub mod dbcore;

use crate::db::dbcore::{DBType, Database};
use crate::log;
use sqlx_core::pool::PoolConnection;
use std::collections::HashMap;
use std::io::Write;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use crate::data::consts::{LOSSLESS_COMPRESSED_FORMATS, LOSSLESS_UNCOMPRESSED_FORMATS, LOSSY_FORMATS};

impl Database {
    pub async fn create_history_db(&self) -> Option<()> {
        let tx = async move |mut conn: PoolConnection<DBType>| -> Result<(), sqlx::Error> {
            let query = "CREATE TABLE IF NOT EXISTS file_history(link TEXT NOT NULL);";
            sqlx::query(sqlx::AssertSqlSafe(query)).execute(&mut *conn).await?;
            Ok(())
        };
        self.call(tx).await
    }

    pub async fn write_file_history(&self, link: &str) -> Option<()> {
        let tx = async move |mut conn: PoolConnection<DBType>| -> Result<(), sqlx::Error> {
            let link = link
                .rsplit_once("%2E")
                .map(|(_ext, name)| name)
                .unwrap_or_default()
                .chars()
                .take(1000)
                .collect::<String>();

            if link.is_empty() {
                log!("Invalid link to insert into database");
                return Ok(())
            }

            let query = format!("INSERT INTO file_history VALUES({link});");
            sqlx::query(sqlx::AssertSqlSafe(query))
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
            let _ = unique_links.extract_if(|name, _ext|
                records.contains(&name.to_string())
                    ||
                // for old databases created the app version <= 1.5.7
                records.iter().any(|rec| {
                    rec
                        .rsplit_once('.')
                        .into_iter()
                        .filter_map(|(ext_, name_)| {
                            if LOSSLESS_UNCOMPRESSED_FORMATS.contains(&ext_) ||
                               LOSSLESS_COMPRESSED_FORMATS.contains(&ext_) ||
                               LOSSY_FORMATS.contains(&ext_)
                            { Some(name_) }
                            else { None }
                        })
                        .any(|name_| {
                            let encoded_name_ = percent_encode(name_.as_bytes(), NON_ALPHANUMERIC).to_string();
                            &encoded_name_ == name
                        })
                })
            );
            Ok(())
        };
        self.call(tx).await
    }
}