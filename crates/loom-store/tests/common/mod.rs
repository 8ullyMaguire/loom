use loom_store::Database;
use rand::RngCore;
use sqlx::{Executor, Pool, Postgres};

pub struct Scratch {
    pub db: Database,
    #[allow(dead_code)]
    pub schema: String,
}

pub fn base_dsn() -> String {
    std::env::var("LOOM_TEST_DSN")
        .unwrap_or_else(|_| "postgres://darkhold:darkhold@127.0.0.1:55433/loom_test".into())
}

pub async fn scratch() -> Scratch {
    let mut bytes = [0u8; 6];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let schema = format!(
        "t_{}",
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    );
    let admin = Pool::<Postgres>::connect(&base_dsn())
        .await
        .expect("admin connect");
    admin
        .execute(format!("CREATE SCHEMA {schema}").as_str())
        .await
        .expect("schema");
    let search_path = schema.clone();
    let options = sqlx::postgres::PgPoolOptions::new().after_connect(move |conn, _| {
        let stmt = format!("SET search_path TO {search_path}");
        Box::pin(async move { conn.execute(stmt.as_str()).await.map(|_| ()) })
    });
    let pool = options.connect(&base_dsn()).await.expect("pool");
    let db = Database::from_pool(pool);
    db.migrate().await.expect("migrate");
    Scratch { db, schema }
}
