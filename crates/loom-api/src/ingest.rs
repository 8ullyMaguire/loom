//! Read-only ingest of the two generators' public /feeds.json documents.
//!
//! No ATProto write verb appears in this module: the directory never writes
//! to a publisher's repo, and the gate in tests/ingest.rs proves it.

use serde::Deserialize;
use sqlx::PgPool;

#[derive(Debug, Clone, Deserialize)]
struct RawFeed {
    name: String,
    #[serde(default)]
    description: String,
    feed_uri: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    ranking: Option<String>,
    #[serde(default)]
    window_hours: Option<i32>,
    #[serde(default)]
    board_size: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedRecord {
    pub source: String,
    pub uri: String,
    pub did: String,
    pub rkey: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub ranking: String,
    pub window_hours: Option<i32>,
    pub board_size: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestSummary {
    pub inserted: usize,
    pub updated: usize,
    pub retired: usize,
}

pub fn parse(source: &str, json: &str) -> Result<Vec<FeedRecord>, serde_json::Error> {
    let doc: serde_json::Value = serde_json::from_str(json)?;
    let raw: Vec<RawFeed> =
        serde_json::from_value(doc.get("feeds").cloned().unwrap_or(serde_json::Value::Null))?;
    Ok(raw
        .into_iter()
        .map(|f| {
            let did = f
                .feed_uri
                .strip_prefix("at://")
                .and_then(|rest| rest.split('/').next())
                .unwrap_or("")
                .to_string();
            let rkey = f.feed_uri.rsplit('/').next().unwrap_or("").to_string();
            FeedRecord {
                rkey,
                source: source.to_string(),
                uri: f.feed_uri,
                did,
                name: f.name,
                description: f.description,
                language: f.language.unwrap_or_default(),
                ranking: f.ranking.unwrap_or_default(),
                window_hours: f.window_hours,
                board_size: f.board_size,
            }
        })
        .collect())
}

pub async fn upsert(pool: &PgPool, records: &[FeedRecord]) -> Result<IngestSummary, sqlx::Error> {
    let Some(source) = records.first().map(|r| r.source.clone()) else {
        return Ok(IngestSummary {
            inserted: 0,
            updated: 0,
            retired: 0,
        });
    };
    let mut inserted = 0;
    let mut updated = 0;
    for record in records {
        let langs: Vec<String> = if record.language.is_empty() {
            vec![]
        } else {
            vec![record.language.clone()]
        };
        let rows: Vec<(bool,)> = sqlx::query_as(
            "insert into feeds
               (uri, did, rkey, display_name, description, publisher, source, lang,
                ranking_mode, window_hours, max_posts, last_seen_at, alive)
             values ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,now(),true)
             on conflict (uri) do update set
               display_name=excluded.display_name, description=excluded.description,
               lang=excluded.lang, ranking_mode=excluded.ranking_mode,
               window_hours=excluded.window_hours, max_posts=excluded.max_posts,
               last_seen_at=now(), alive=true
             returning (xmax = 0)",
        )
        .bind(&record.uri)
        .bind(&record.did)
        .bind(&record.rkey)
        .bind(&record.name)
        .bind(&record.description)
        .bind(&record.did)
        .bind(&record.source)
        .bind(&langs)
        .bind(&record.ranking)
        .bind(record.window_hours)
        .bind(record.board_size)
        .fetch_all(pool)
        .await?;
        for (was_insert,) in rows {
            if was_insert {
                inserted += 1
            } else {
                updated += 1
            }
        }
    }
    let uris: Vec<String> = records.iter().map(|r| r.uri.clone()).collect();
    let result = sqlx::query(
        "update feeds set alive=false where source=$1 and uri <> all($2) and alive=true",
    )
    .bind(&source)
    .bind(&uris)
    .execute(pool)
    .await?;
    Ok(IngestSummary {
        inserted,
        updated,
        retired: result.rows_affected() as usize,
    })
}

pub const LEFTIST_FEEDS_URL: &str = "https://leftist.polarisocial.xyz/feeds.json";
pub const BNWO_FEEDS_URL: &str = "https://bnwo-feed.polarisocial.xyz/feeds.json";

/// Fetch one public feed document. GET only; no credentials, no writes.
pub async fn fetch(
    client: &reqwest::Client,
    source: &str,
    url: &str,
) -> Result<Vec<FeedRecord>, Box<dyn std::error::Error + Send + Sync>> {
    let text = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    Ok(parse(source, &text)?)
}

/// Refresh both deployments from their public JSON. Returns the combined counts.
pub async fn refresh(
    client: &reqwest::Client,
    pool: &PgPool,
) -> Result<(IngestSummary, IngestSummary), Box<dyn std::error::Error + Send + Sync>> {
    let leftist = fetch(client, "leftist", LEFTIST_FEEDS_URL).await?;
    let bnwo = fetch(client, "bnwo", BNWO_FEEDS_URL).await?;
    let a = upsert(pool, &leftist).await?;
    let b = upsert(pool, &bnwo).await?;
    Ok((a, b))
}
