use axum::serve;
use loom_api::openapi;
use loom_api::state::AppState;
use loom_store::Database;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--print-openapi") {
        println!("{}", serde_json::to_string_pretty(&openapi::document())?);
        return Ok(());
    }
    let dsn = std::env::var("LOOM_DSN")
        .unwrap_or_else(|_| "postgres://darkhold:darkhold@127.0.0.1:55433/loom_test".into());
    let db = Database::connect(&dsn).await?;
    let state = AppState::new(db);
    let listener = TcpListener::bind("127.0.0.1:8079").await?;
    serve(listener, loom_api::router(state)).await?;
    Ok(())
}
