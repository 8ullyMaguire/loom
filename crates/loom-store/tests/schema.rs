mod common;

#[tokio::test]
async fn migration_creates_all_nine_tables() {
    let s = common::scratch().await;
    let rows: Vec<(String,)> = sqlx::query_as(
        "select tablename from pg_tables where schemaname = current_schema() order by tablename",
    )
    .fetch_all(s.db.pool())
    .await
    .unwrap();
    let names: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
    assert_eq!(names.len(), 9, "actual tables: {names:?}");
    for expected in [
        "bundle_feeds",
        "bundle_links",
        "bundles",
        "events",
        "feeds",
        "idempotency_keys",
        "reader_signals",
        "reports",
        "tag_votes",
    ] {
        assert!(names.contains(&expected), "missing {expected}");
    }
}

#[tokio::test]
async fn migration_is_idempotent() {
    let s = common::scratch().await;
    s.db.migrate().await.expect("second migrate");
    let n: (i64,) =
        sqlx::query_as("select count(*) from pg_tables where schemaname = current_schema()")
            .fetch_one(s.db.pool())
            .await
            .unwrap();
    assert_eq!(n.0, 9);
}
