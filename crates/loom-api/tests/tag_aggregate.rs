mod common;

#[tokio::test]
async fn confidence_publishes_only_at_five_independent_votes() {
    let f = common::fixture().await;
    let client = reqwest::Client::new();
    let feed = "at://did:plc:test/app.bsky.feed.generator/one";
    let url = format!("{}/api/v1/tags", f.base);

    for i in 0..4 {
        let status = client
            .post(&url)
            .header("X-Visitor-Key", format!("voter-{i}"))
            .json(&serde_json::json!({"feed": feed, "tag": "marxism", "confidence": 80}))
            .send()
            .await
            .unwrap()
            .status();
        assert_eq!(status, 201, "vote {i} accepted");
    }
    let four: serde_json::Value = client
        .get(format!("{}/api/v1/tags?feed={}&tag=marxism", f.base, feed))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(four["status"], "insufficient_votes");
    assert_eq!(four["n"], 4);
    assert!(
        four.get("mean").is_none(),
        "no fabricated number below the floor"
    );

    client
        .post(&url)
        .header("X-Visitor-Key", "voter-4")
        .json(&serde_json::json!({"feed": feed, "tag": "marxism", "confidence": 60}))
        .send()
        .await
        .unwrap()
        .status();
    let five: serde_json::Value = client
        .get(format!("{}/api/v1/tags?feed={}&tag=marxism", f.base, feed))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(five["status"], "published");
    assert_eq!(five["n"], 5);
    let mean = five["mean"].as_f64().expect("mean");
    let low = five["ci_low"].as_f64().expect("ci_low");
    let high = five["ci_high"].as_f64().expect("ci_high");
    assert!((mean - 76.0).abs() < 1e-6);
    assert!(low <= mean && mean <= high);

    // A re-vote by the same contributor replaces, it does not add a voter.
    client
        .post(&url)
        .header("X-Visitor-Key", "voter-4")
        .json(&serde_json::json!({"feed": feed, "tag": "marxism", "confidence": 100}))
        .send()
        .await
        .unwrap()
        .status();
    let again: serde_json::Value = client
        .get(format!("{}/api/v1/tags?feed={}&tag=marxism", f.base, feed))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(again["n"], 5, "re-votes keep the independent count");
    f.server.abort();
}
