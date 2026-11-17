mod common;

async fn seed(f: &common::Fixture) {
    let leftist =
        loom_api::ingest::parse("leftist", include_str!("fixtures/leftist-feeds.json")).unwrap();
    let bnwo = loom_api::ingest::parse("bnwo", include_str!("fixtures/bnwo-feeds.json")).unwrap();
    loom_api::ingest::upsert(&f.pool, &leftist).await.unwrap();
    loom_api::ingest::upsert(&f.pool, &bnwo).await.unwrap();
}

#[tokio::test]
async fn faceted_search_is_deterministic_and_pages_by_cursor() {
    let f = common::fixture().await;
    seed(&f).await;
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/search", f.base);

    let first: serde_json::Value = client
        .get(&url)
        .query(&[("source", "leftist"), ("limit", "5")])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let again: serde_json::Value = client
        .get(&url)
        .query(&[("source", "leftist"), ("limit", "5")])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(first, again, "same query must produce the same page");
    let rows = first["items"].as_array().expect("items");
    assert_eq!(rows.len(), 5);
    assert!(rows.iter().all(|r| r["source"] == "leftist"));
    assert!(first["next_cursor"].is_string());

    let cursor = first["next_cursor"].as_str().unwrap();
    let second: serde_json::Value = client
        .get(&url)
        .query(&[("source", "leftist"), ("limit", "5"), ("cursor", cursor)])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let a: Vec<&str> = rows.iter().map(|r| r["uri"].as_str().unwrap()).collect();
    let b: Vec<&str> = second["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["uri"].as_str().unwrap())
        .collect();
    assert!(
        a.iter().all(|x| !b.contains(x)),
        "cursor must not repeat rows"
    );

    let empty: serde_json::Value = client
        .get(&url)
        .query(&[("source", "nosuchsource")])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(empty["items"].as_array().unwrap().len(), 0);
    f.server.abort();
}

#[tokio::test]
async fn hidden_gems_reports_a_measured_tail_share() {
    let f = common::fixture().await;
    seed(&f).await;
    let gems: serde_json::Value = reqwest::Client::new()
        .get(format!("{}/api/v1/hidden-gems", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let share = gems["impression_share"].as_f64().expect("measured share");
    assert!((0.0..=1.0).contains(&share));
    assert!(!gems["items"].as_array().unwrap().is_empty());
    println!("hidden-gems impression_share={share}");
    f.server.abort();
}
