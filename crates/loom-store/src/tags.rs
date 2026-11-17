//! One confidence vote per contributor × feed × tag; a re-vote replaces it.

use sqlx::PgPool;

use crate::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct TagAggregate {
    pub status: &'static str,
    pub n: usize,
    pub mean: Option<f64>,
    pub ci_low: Option<f64>,
    pub ci_high: Option<f64>,
}

pub async fn vote(
    pool: &PgPool,
    contributor_key: &str,
    feed_uri: &str,
    tag: &str,
    confidence: i32,
) -> Result<(), Error> {
    sqlx::query(
        "insert into tag_votes (contributor_key, feed_uri, tag, confidence, updated_at)
         values ($1,$2,$3,$4,now())
         on conflict (contributor_key, feed_uri, tag)
         do update set confidence = excluded.confidence, updated_at = now()",
    )
    .bind(contributor_key)
    .bind(feed_uri)
    .bind(tag)
    .bind(confidence)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn aggregate(pool: &PgPool, feed_uri: &str, tag: &str) -> Result<TagAggregate, Error> {
    let rows: Vec<(String, i32)> = sqlx::query_as(
        "select contributor_key, confidence from tag_votes where feed_uri=$1 and tag=$2",
    )
    .bind(feed_uri)
    .bind(tag)
    .fetch_all(pool)
    .await?;
    let votes: Vec<loom_core::confidence::Vote> = rows
        .into_iter()
        .map(
            |(contributor_key, confidence)| loom_core::confidence::Vote {
                contributor_key,
                confidence: confidence as u8,
            },
        )
        .collect();
    match loom_core::confidence::aggregate(&votes) {
        Some(a) => Ok(TagAggregate {
            status: "published",
            n: a.n,
            mean: Some(a.mean),
            ci_low: Some(a.ci_low),
            ci_high: Some(a.ci_high),
        }),
        None => Ok(TagAggregate {
            status: "insufficient_votes",
            n: votes.len(),
            mean: None,
            ci_low: None,
            ci_high: None,
        }),
    }
}
