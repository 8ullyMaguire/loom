use loom_api::state::AppState;
use loom_store::Database;
use rand::RngCore;
use sqlx::{Executor, PgPool, Pool, Postgres};

pub struct Fixture {
    #[allow(dead_code)]
    pub base: String,
    #[allow(dead_code)]
    pub pool: PgPool,
    pub server: tokio::task::JoinHandle<()>,
}

pub fn dsn() -> String {
    std::env::var("LOOM_TEST_DSN")
        .unwrap_or_else(|_| "postgres://darkhold:darkhold@127.0.0.1:55433/loom_test".into())
}

pub async fn fixture() -> Fixture {
    let mut bytes = [0u8; 6];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let schema = format!(
        "a_{}",
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    );
    let admin = Pool::<Postgres>::connect(&dsn()).await.unwrap();
    admin
        .execute(format!("CREATE SCHEMA {schema}").as_str())
        .await
        .unwrap();
    let path = schema.clone();
    let options = sqlx::postgres::PgPoolOptions::new().after_connect(move |conn, _| {
        let stmt = format!("SET search_path TO {path}");
        Box::pin(async move { conn.execute(stmt.as_str()).await.map(|_| ()) })
    });
    let pool = options.connect(&dsn()).await.unwrap();
    let db = Database::from_pool(pool.clone());
    db.migrate().await.unwrap();
    for (uri, rkey) in [
        ("at://did:plc:test/app.bsky.feed.generator/one", "one"),
        ("at://did:plc:test/app.bsky.feed.generator/two", "two"),
    ] {
        sqlx::query(
            "insert into feeds (uri,did,rkey,display_name,source) values ($1,$2,$3,$4,'leftist')",
        )
        .bind(uri)
        .bind("did:plc:test")
        .bind(rkey)
        .bind("Test feed")
        .execute(&pool)
        .await
        .unwrap();
    }
    let state = AppState {
        db,
        sunset_threshold: 20,
    };
    let app = loom_api::router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    Fixture { base, pool, server }
}
