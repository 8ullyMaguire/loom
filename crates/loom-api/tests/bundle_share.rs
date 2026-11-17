mod common;

#[tokio::test]
async fn anonymous_create_returns_token_and_one_link_per_feed() {
    let f = common::fixture().await;
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/v1/bundles", f.base))
        .json(&serde_json::json!({
            "name": "My feeds",
            "description": "a small bundle",
            "feeds": [
                "at://did:plc:test/app.bsky.feed.generator/one",
                "at://did:plc:test/app.bsky.feed.generator/two"
            ]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 201, "anonymous create must succeed");
    let body: serde_json::Value = response.json().await.unwrap();
    let token = body["edit_token"].as_str().expect("edit token").to_string();
    assert_eq!(token.len(), 43);
    let id = body["id"].as_str().unwrap();

    let html = client
        .get(format!("{}/b/{}", f.base, id))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(html.contains("data-loom-ssr"), "real HTML first paint");
    assert_eq!(html.matches("<li>").count(), 2, "one link per feed");
    assert!(html.contains("https://bsky.app/profile/did:plc:test/feed/one"));

    let edit_url = format!("{}/api/v1/bundles/{}", f.base, id);
    let payload = serde_json::json!({"name":"Renamed","description":"","feeds":[
        "at://did:plc:test/app.bsky.feed.generator/one"]});
    assert_eq!(
        client
            .patch(&edit_url)
            .header("X-Edit-Token", "wrong")
            .json(&payload)
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        client
            .patch(&edit_url)
            .header("X-Edit-Token", &token)
            .json(&payload)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    f.server.abort();
}
