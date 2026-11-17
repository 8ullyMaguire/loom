//! loomctl — power-user client. Exit codes: 0 ok, 1 env, 2 usage, 3 not found,
//! 4 auth, 5 rate-limited.

mod tui;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "loomctl", version, about = "Loom command-line client")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    /// Edit token for `bundle edit` and `tag add`.
    #[arg(long, global = true)]
    token: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    #[command(subcommand)]
    Bundle(BundleCommand),
    Tag {
        feed: String,
        tag: String,
        confidence: i32,
    },
    Search {
        #[arg(long)]
        q: Option<String>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        limit: Option<i64>,
    },
    #[command(subcommand)]
    Ingest(IngestCommand),
    /// Typed raw request against any documented path.
    Request {
        method: String,
        path: String,
        #[arg(long)]
        json: Option<String>,
    },
    Tui,
}

#[derive(Subcommand)]
enum BundleCommand {
    Create {
        name: String,
        #[arg(long, default_value = "")]
        description: String,
        feeds: Vec<String>,
    },
    List,
    Edit {
        id: String,
        name: String,
        #[arg(long, default_value = "")]
        description: String,
        feeds: Vec<String>,
    },
    Export {
        id: String,
    },
    Import {
        path: String,
    },
}

#[derive(Subcommand)]
enum IngestCommand {
    Status,
}

fn api_base() -> Result<String, u8> {
    let base = std::env::var("LOOM_API").map_err(|_| 1)?;
    if !(base.starts_with("http://") || base.starts_with("https://")) {
        return Err(1);
    }
    Ok(base.trim_end_matches('/').to_string())
}

pub async fn call(
    method: &str,
    base: &str,
    path: &str,
    body: Option<&serde_json::Value>,
    token: Option<&str>,
) -> Result<String, u8> {
    let client = reqwest::Client::new();
    let mut request = client.request(
        method.to_ascii_uppercase().parse().map_err(|_| 2)?,
        format!("{base}{path}"),
    );
    if let Some(body) = body {
        request = request.json(body);
    }
    if let Some(token) = token {
        request = request.header("X-Edit-Token", token);
    }
    let response = request.send().await.map_err(|_| 1)?;
    match response.status().as_u16() {
        200..=299 => response.text().await.map_err(|_| 1),
        404 => Err(3),
        401 | 403 => Err(4),
        429 => Err(5),
        _ => Err(1),
    }
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    let code = dispatch(cli).await;
    std::process::ExitCode::from(code)
}

async fn dispatch(cli: Cli) -> u8 {
    let base = match api_base() {
        Ok(base) => base,
        Err(code) => return code,
    };
    let token = cli.token.as_deref();
    let result = match cli.command {
        Command::Bundle(BundleCommand::Create {
            name,
            description,
            feeds,
        }) => {
            let body =
                serde_json::json!({"name": name, "description": description, "feeds": feeds});
            call("POST", &base, "/api/v1/bundles", Some(&body), None).await
        }
        Command::Bundle(BundleCommand::List) => {
            call("GET", &base, "/api/v1/bundles", None, token).await
        }
        Command::Bundle(BundleCommand::Edit {
            id,
            name,
            description,
            feeds,
        }) => {
            let body =
                serde_json::json!({"name": name, "description": description, "feeds": feeds});
            call(
                "PATCH",
                &base,
                &format!("/api/v1/bundles/{id}"),
                Some(&body),
                token,
            )
            .await
        }
        Command::Bundle(BundleCommand::Export { id }) => {
            call("GET", &base, &format!("/api/v1/bundles/{id}"), None, None).await
        }
        Command::Bundle(BundleCommand::Import { path }) => {
            let Ok(text) = std::fs::read_to_string(path) else {
                return 1;
            };
            let Ok(body) = serde_json::from_str::<serde_json::Value>(&text) else {
                return 2;
            };
            call("POST", &base, "/api/v1/bundles", Some(&body), None).await
        }
        Command::Tag {
            feed,
            tag,
            confidence,
        } => {
            let body = serde_json::json!({"feed": feed, "tag": tag, "confidence": confidence});
            call("POST", &base, "/api/v1/tags", Some(&body), None).await
        }
        Command::Search { q, source, limit } => {
            let mut params: Vec<(String, String)> = vec![];
            if let Some(q) = q {
                params.push(("q".into(), q));
            }
            if let Some(source) = source {
                params.push(("source".into(), source));
            }
            if let Some(limit) = limit {
                params.push(("limit".into(), limit.to_string()));
            }
            let suffix = if params.is_empty() {
                String::new()
            } else {
                let encoded = params
                    .iter()
                    .map(|(k, v)| format!("{k}={}", v.replace(' ', "+")))
                    .collect::<Vec<_>>()
                    .join("&");
                format!("?{encoded}")
            };
            call("GET", &base, &format!("/api/v1/search{suffix}"), None, None).await
        }
        Command::Ingest(IngestCommand::Status) => {
            call("GET", &base, "/api/v1/capabilities", None, None).await
        }
        Command::Request { method, path, json } => {
            let body = json
                .as_deref()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());
            call(&method, &base, &path, body.as_ref(), token).await
        }
        Command::Tui => return tui::run(&base).await,
    };
    match result {
        Ok(body) => {
            println!("{body}");
            0
        }
        Err(code) => code,
    }
}
