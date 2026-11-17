mod common;

async fn add_reports(f: &common::Fixture, n: usize) {
    for i in 0..n {
        sqlx::query(
            "insert into reports (kind,target_id,reason,reporter_key) values ('bundle','x','spam',$1)",
        )
        .bind(format!("reporter-{i}"))
        .execute(&f.pool)
        .await
        .unwrap();
    }
}

fn create(client: &reqwest::Client, base: &str) -> reqwest::RequestBuilder {
    client
        .post(format!("{base}/api/v1/bundles"))
        .json(&serde_json::json!({
            "name": "Sunset bundle", "description": "",
            "feeds": ["at://did:plc:test/app.bsky.feed.generator/one"]
        }))
}

#[tokio::test]
async fn sunset_closes_anonymous_creation_but_not_existing_edit_tokens() {
    let f = common::fixture().await;
    let client = reqwest::Client::new();
    let before: serde_json::Value = client
        .get(format!("{}/api/v1/capabilities", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(before["bundle_create_anonymous"], true);

    add_reports(&f, 19).await;
    let created: serde_json::Value = create(&client, &f.base)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = created["id"].as_str().unwrap().to_string();
    let token = created["edit_token"].as_str().unwrap().to_string();

    add_reports(&f, 1).await;
    let blocked = create(&client, &f.base).send().await.unwrap();
    assert_eq!(blocked.status(), 401);
    let error: serde_json::Value = blocked.json().await.unwrap();
    assert_eq!(error["error"], "auth_required");

    let after: serde_json::Value = client
        .get(format!("{}/api/v1/capabilities", f.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(after["bundle_create_anonymous"], false);
    assert_eq!(after["reports_30d"], 20);
    assert_eq!(after["threshold"], 20);
    assert_eq!(after["tags"], true);
    assert_eq!(after["recommendations"], true);
    assert_eq!(after["tui"], true, "the CLI/TUI row shipped");

    let patch = client
        .patch(format!("{}/api/v1/bundles/{id}", f.base))
        .header("X-Edit-Token", &token)
        .json(&serde_json::json!({
            "name": "Still editable", "description": "",
            "feeds": ["at://did:plc:test/app.bsky.feed.generator/one"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        patch.status(),
        200,
        "existing edit tokens survive the sunset"
    );
    println!("sunset: PASS (19 -> 201, 20 -> 401, capabilities closed)");
    f.server.abort();
}
