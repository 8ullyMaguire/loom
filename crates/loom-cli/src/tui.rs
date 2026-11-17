//! Small TUI. In a pipe it becomes a deterministic line script for CI.

use std::io::{self, BufRead, IsTerminal};

use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

pub async fn run(base: &str) -> u8 {
    if io::stdin().is_terminal() {
        interactive(base)
    } else {
        scripted(base).await
    }
}

async fn scripted(base: &str) -> u8 {
    let mut lines = io::stdin().lock().lines();
    let Some(command) = lines.next().and_then(Result::ok) else {
        return 2;
    };
    match command.trim() {
        "list" => match crate::call("GET", base, "/api/v1/bundles", None, None).await {
            Ok(body) => {
                println!("{body}");
                0
            }
            Err(code) => code,
        },
        "create" => {
            let Some(name) = lines.next().and_then(Result::ok) else {
                return 2;
            };
            let Some(feeds) = lines.next().and_then(Result::ok) else {
                return 2;
            };
            let list: Vec<String> = feeds
                .split(',')
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            let body = serde_json::json!({"name": name, "description": "", "feeds": list});
            match crate::call("POST", base, "/api/v1/bundles", Some(&body), None).await {
                Ok(text) => {
                    let value: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
                    match value["id"].as_str() {
                        Some(id) => {
                            println!("{id}");
                            0
                        }
                        None => 1,
                    }
                }
                Err(code) => code,
            }
        }
        _ => 2,
    }
}

fn interactive(base: &str) -> u8 {
    let mut terminal = match Terminal::new(CrosstermBackend::new(io::stdout())) {
        Ok(t) => t,
        Err(_) => return 1,
    };
    if enable_raw_mode().is_err() {
        return 1;
    }
    let mut message = String::from("n: create from clipboard-free prompt  q: quit");
    loop {
        let _ = terminal.draw(|frame| {
            let area = frame.area();
            let widget = Paragraph::new(message.clone())
                .block(Block::default().borders(Borders::ALL).title("loom"));
            frame.render_widget(widget, area);
        });
        if let Ok(Event::Key(key)) = read() {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('r') => {
                    let rt = tokio::runtime::Runtime::new().ok();
                    message = match rt.and_then(|rt| {
                        rt.block_on(async {
                            crate::call("GET", base, "/api/v1/bundles", None, None)
                                .await
                                .ok()
                        })
                    }) {
                        Some(body) if body.len() < 90 => body,
                        Some(_) => "bundle list loaded".into(),
                        None => "service unreachable".into(),
                    };
                }
                _ => {}
            }
        }
    }
    let _ = disable_raw_mode();
    0
}
