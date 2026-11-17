mod common;

#[tokio::test]
async fn duplicate_position_is_rejected() {
    let s = common::scratch().await;
    let uri = "at://did:plc:test/app.bsky.feed.generator/one";
    sqlx::query(
        "insert into feeds (uri, did, rkey, display_name, source) values ($1,$2,$3,$4,'leftist')",
    )
    .bind(uri)
    .bind("did:plc:test")
    .bind("one")
    .bind("Test feed")
    .execute(s.db.pool())
    .await
    .unwrap();
    let id = uuid::Uuid::new_v4();
    sqlx::query("insert into bundles (id, owner_key, name, edit_token_hash) values ($1,$2,$3,$4)")
        .bind(id)
        .bind("visitor-1")
        .bind("Test bundle")
        .bind(vec![0u8; 32])
        .execute(s.db.pool())
        .await
        .unwrap();
    let insert = "insert into bundle_feeds (bundle_id, feed_uri, position) values ($1,$2,$3)";
    sqlx::query(insert)
        .bind(id)
        .bind(uri)
        .bind(0)
        .execute(s.db.pool())
        .await
        .unwrap();
    assert!(
        sqlx::query(insert)
            .bind(id)
            .bind(uri)
            .bind(0)
            .execute(s.db.pool())
            .await
            .is_err(),
        "duplicate position must fail"
    );
}
