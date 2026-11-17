mod common;

async fn seed(f: &common::Fixture) {
    let leftist =
        loom_api::ingest::parse("leftist", include_str!("fixtures/leftist-feeds.json")).unwrap();
    let bnwo = loom_api::ingest::parse("bnwo", include_str!("fixtures/bnwo-feeds.json")).unwrap();
    loom_api::ingest::upsert(&f.pool, &leftist).await.unwrap();
    loom_api::ingest::upsert(&f.pool, &bnwo).await.unwrap();
}

async fn uris(base: &str, reader: &str) -> Vec<String> {
    let body: serde_json::Value = reqwest::Client::new()
        .get(format!("{base}/api/v1/recommendations"))
        .query(&[("surface", "api"), ("reader", reader)])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    body["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|i| i["uri"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn ramp() {
    let f = common::fixture().await;
    seed(&f).await;
    let base = uris(&f.base, "cold").await;
    assert_eq!(base.len(), 26);

    // Sixty signals on one feed by the same reader.
    let target = "at://did:plc:bnjxoctfnukack6evxsbexc4/app.bsky.feed.generator/leftist-4h";
    for i in 0..60 {
        let status = reqwest::Client::new()
            .post(format!("{}/api/v1/tags", f.base))
            .header("X-Visitor-Key", "warm")
            .json(
                &serde_json::json!({"feed": target, "tag": format!("topic-{i}"), "confidence": 80}),
            )
            .send()
            .await
            .unwrap()
            .status();
        assert_eq!(status, 201);
    }
    let warm = uris(&f.base, "warm").await;
    assert_eq!(warm.len(), 26);
    assert_ne!(
        base, warm,
        "a sixty-signal history must change the served order"
    );
    assert_eq!(warm[0], target, "the signalled feed should lead the list");
    println!("ramp: PASS ({} vs {} differ)", base.len(), warm.len());
    f.server.abort();
}

fn strip_user_text(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.remove("description");
            map.remove("name");
            for v in map.values_mut() {
                strip_user_text(v);
            }
        }
        serde_json::Value::Array(items) => {
            for v in items {
                strip_user_text(v);
            }
        }
        _ => {}
    }
}

fn scrub(body: &str, path: &str) -> String {
    if path.starts_with("/b/") {
        // The share page's <p> is the publisher's own description, not our copy.
        if let Some(start) = body.find("<p>") {
            if let Some(end) = body[start..].find("</p>") {
                return format!("{}{}", &body[..start], &body[start + end + 4..]);
            }
        }
        body.to_string()
    } else {
        let mut value: serde_json::Value =
            serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
        strip_user_text(&mut value);
        value.to_string()
    }
}

#[tokio::test]
async fn secrecy() {
    let f = common::fixture().await;
    seed(&f).await;
    let created: serde_json::Value = reqwest::Client::new()
        .post(format!("{}/api/v1/bundles", f.base))
        .json(&serde_json::json!({"name":"Check","description":"","feeds":["at://did:plc:bnjxoctfnukack6evxsbexc4/app.bsky.feed.generator/leftist-4h"]}))
        .send().await.unwrap().json().await.unwrap();
    let id = created["id"].as_str().unwrap();
    let table = include_str!("../../../docs/ROUTE_TABLE.md");
    let mut checked = 0;
    let forbidden = [
        "gravity",
        "prior",
        "taste",
        "attuned",
        "affinity",
        "taste-source",
        "lens",
    ];
    for line in table.lines().filter(|l| l.starts_with('|')) {
        let Some(path) = line.split('|').map(str::trim).find(|c| c.starts_with('/')) else {
            continue;
        };
        let path = path.replace("{id}", id);
        let body = reqwest::Client::new()
            .get(format!("{}{}", f.base, path))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert!(!body.is_empty(), "empty body on {path}");
        let lower = scrub(&body, &path).to_lowercase();
        for word in forbidden {
            assert!(
                !lower.contains(word),
                "hidden mechanism leaked on {path}: {word}"
            );
        }
        checked += 1;
    }
    assert_eq!(checked, 6, "every row of ROUTE_TABLE.md must be checked");
    println!("secrecy: PASS ({checked} paths)");
    f.server.abort();
}
