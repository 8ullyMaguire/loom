mod common;

#[tokio::test]
async fn recorded_feeds_ingest_read_only_and_idempotently() {
    let leftist = loom_api::ingest::parse("leftist", include_str!("fixtures/leftist-feeds.json"))
        .expect("leftist fixture parses");
    let bnwo = loom_api::ingest::parse("bnwo", include_str!("fixtures/bnwo-feeds.json"))
        .expect("bnwo fixture parses");
    assert_eq!(leftist.len(), 14);
    assert_eq!(bnwo.len(), 12);
    assert_eq!(leftist[0].did, "did:plc:bnjxoctfnukack6evxsbexc4");
    assert!(leftist[0].uri.starts_with("at://did:plc:"));

    let f = common::fixture().await;
    let first = loom_api::ingest::upsert(&f.pool, &leftist).await.unwrap();
    assert_eq!(first.inserted, 14);
    let alive: (i64,) =
        sqlx::query_as("select count(*) from feeds where source='leftist' and alive=true")
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert_eq!(alive.0, 14);

    let second = loom_api::ingest::upsert(&f.pool, &leftist).await.unwrap();
    assert_eq!(second.inserted, 0, "re-ingest adds no rows");
    let bnwo_first = loom_api::ingest::upsert(&f.pool, &bnwo).await.unwrap();
    assert_eq!(bnwo_first.inserted, 12);
    let live: (i64,) = sqlx::query_as(
        "select count(*) from feeds where source in ('leftist','bnwo') and alive=true",
    )
    .fetch_one(&f.pool)
    .await
    .unwrap();
    assert_eq!(
        live.0, 26,
        "all live feeds from both generators must be indexed"
    );
    let total: (i64,) = sqlx::query_as("select count(*) from feeds")
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert_eq!(total.0, 28, "26 live feeds plus 2 seeded fixture feeds");

    let mut shortened = leftist.clone();
    shortened.pop();
    let retired = loom_api::ingest::upsert(&f.pool, &shortened).await.unwrap();
    assert_eq!(retired.retired, 1);
    f.server.abort();
}

#[test]
fn ingest_contains_no_atproto_write_verbs() {
    let source = include_str!("../src/ingest.rs");
    for verb in ["createRecord", "putRecord", "deleteRecord"] {
        assert!(
            !source.contains(verb),
            "ingest must never write to a PDS: {verb}"
        );
    }
}
