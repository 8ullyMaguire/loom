use std::io::Write;
use std::process::{Command, Stdio};

async fn mock_server() -> String {
    async fn ok() -> &'static str {
        "{\"items\":[]}"
    }
    async fn created() -> &'static str {
        "{\"id\":\"11111111-1111-1111-1111-111111111111\",\"edit_token\":\"tok\",\"url\":\"/b/1\"}"
    }
    async fn not_found() -> axum::response::Response {
        (
            axum::http::StatusCode::NOT_FOUND,
            "{\"error\":\"not_found\"}",
        )
            .into_response()
    }
    async fn denied() -> axum::response::Response {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            "{\"error\":\"auth_required\"}",
        )
            .into_response()
    }
    async fn limited() -> axum::response::Response {
        (
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            "{\"error\":\"rate_limited\"}",
        )
            .into_response()
    }
    use axum::response::IntoResponse;
    let app = axum::Router::new()
        .route("/api/v1/bundles", axum::routing::get(ok).post(created))
        .route("/api/v1/never", axum::routing::get(not_found))
        .route("/api/v1/denied", axum::routing::get(denied))
        .route("/api/v1/limited", axum::routing::get(limited));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    base
}

fn run(base: &str, args: &[&str]) -> (u8, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_loomctl"))
        .args(args)
        .env("LOOM_API", base)
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(99) as u8;
    (code, String::from_utf8_lossy(&output.stdout).to_string())
}

#[tokio::test(flavor = "multi_thread")]
async fn exit_codes_are_stable_across_every_failure_mode() {
    let base = mock_server().await;
    assert_eq!(run(&base, &["bundle", "list"]).0, 0, "ok = 0");
    assert_eq!(run("not a url", &["bundle", "list"]).0, 1, "env = 1");
    assert_eq!(run(&base, &["nonsense"]).0, 2, "usage = 2");
    assert_eq!(
        run(&base, &["request", "get", "/api/v1/never"]).0,
        3,
        "not found = 3"
    );
    assert_eq!(
        run(&base, &["request", "get", "/api/v1/denied"]).0,
        4,
        "auth = 4"
    );
    assert_eq!(
        run(&base, &["request", "get", "/api/v1/limited"]).0,
        5,
        "rate = 5"
    );

    // TUI in scripted mode prints the created bundle id on stdout.
    let mut child = Command::new(env!("CARGO_BIN_EXE_loomctl"))
        .args(["tui"])
        .env("LOOM_API", &base)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"create\nScripted bundle\nat://did:plc:test/app.bsky.feed.generator/one\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code().unwrap(), 0);
    assert!(
        text.contains("11111111-1111-1111-1111-111111111111"),
        "TUI printed {text}"
    );
    println!("exit codes 0..5: PASS; tui: PASS");
}
