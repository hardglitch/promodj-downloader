use sqlx_core::pool::PoolConnection;
use crate::db::dbcore::{DBType, Database};

#[ignore]
#[tokio::test]
async fn main() {

    // 1. Open old db
    let db = Database::create_or_open("history.db").await.unwrap();
    let tx = async move |mut conn: PoolConnection<DBType>| -> Result<Vec<String>, sqlx::Error> {
        let links = sqlx::query_scalar::<_, String>("SELECT link FROM file_history")
            .fetch_all(&mut *conn).await?;
        Ok(links)
    };
    let links: Vec<String> = db.call(tx).await.unwrap();

    // 2. Create new db
    let db = Database::create_or_open("history_new.db").await.unwrap();
    db.create_tables().await;
    for link in links {
        db.write_file_history(&link).await;
    }
}