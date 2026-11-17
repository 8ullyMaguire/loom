mod common;

async fn seed_feed(pool: &sqlx::PgPool, uri: &str) {
    sqlx::query(
        "insert into feeds (uri, did, rkey, display_name, source) values ($1,$2,$3,$4,'leftist')",
    )
    .bind(uri)
    .bind("did:plc:test")
    .bind(uri.rsplit('/').next().unwrap())
    .bind("Feed")
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn create_get_edit_and_token_rotation() {
    let s = common::scratch().await;
    let a = "at://did:plc:test/app.bsky.feed.generator/one".to_string();
    let b = "at://did:plc:test/app.bsky.feed.generator/two".to_string();
    seed_feed(s.db.pool(), &a).await;
    seed_feed(s.db.pool(), &b).await;

    let created = loom_store::bundles::create(
        s.db.pool(),
        "visitor-1",
        "My feeds",
        "a bundle",
        vec![b.clone(), a.clone()],
        None,
    )
    .await
    .unwrap();
    assert_eq!(created.edit_token.len(), 43);
    let view = loom_store::bundles::get(s.db.pool(), created.view.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(view.uris, vec![b.clone(), a.clone()], "order preserved");
    assert_eq!(view.version, 1);

    let wrong = loom_store::bundles::edit(
        s.db.pool(),
        created.view.id,
        "wrong-token",
        "Bad",
        "",
        vec![a.clone()],
    )
    .await;
    assert!(matches!(wrong, Err(loom_store::Error::Auth)));

    let edited = loom_store::bundles::edit(
        s.db.pool(),
        created.view.id,
        &created.edit_token,
        "Renamed",
        "new",
        vec![a.clone()],
    )
    .await
    .unwrap();
    assert_eq!(edited.name, "Renamed");
    assert_eq!(edited.version, 2);
    assert_eq!(edited.uris, vec![a.clone()]);

    let old = loom_store::bundles::edit(
        s.db.pool(),
        created.view.id,
        &created.edit_token,
        "Again",
        "",
        vec![a.clone()],
    )
    .await;
    assert!(
        matches!(old, Err(loom_store::Error::Auth)),
        "old token invalid after rotation"
    );
}

#[tokio::test]
async fn unknown_feed_is_rejected_and_list_scopes_to_owner() {
    let s = common::scratch().await;
    let a = "at://did:plc:test/app.bsky.feed.generator/one".to_string();
    seed_feed(s.db.pool(), &a).await;
    assert!(loom_store::bundles::create(
        s.db.pool(),
        "v",
        "bad",
        "",
        vec!["at://missing/feed/x".into()],
        None,
    )
    .await
    .is_err());
    let one = loom_store::bundles::create(s.db.pool(), "v1", "one", "", vec![a.clone()], None)
        .await
        .unwrap();
    let two = loom_store::bundles::create(s.db.pool(), "v2", "two", "", vec![a.clone()], None)
        .await
        .unwrap();
    let v1 = loom_store::bundles::list(s.db.pool(), Some("v1"), None, 50)
        .await
        .unwrap();
    assert_eq!(v1.len(), 1);
    assert_eq!(v1[0].id, one.view.id);
    let cursor = std::cmp::min(one.view.id, two.view.id);
    let expected = std::cmp::max(one.view.id, two.view.id);
    let page = loom_store::bundles::list(s.db.pool(), None, Some(cursor), 50)
        .await
        .unwrap();
    assert!(page.iter().any(|v| v.id == expected));
    assert!(!page.iter().any(|v| v.id == cursor));
}
