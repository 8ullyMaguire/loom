//! Faceted, deterministic search over indexed feeds.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub tag: Option<String>,
    pub lang: Option<String>,
    pub source: Option<String>,
    pub mode: Option<String>,
    pub min_votes: Option<i64>,
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SearchHit {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub source: String,
    pub ranking_mode: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchPage {
    pub items: Vec<SearchHit>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HiddenGems {
    pub impression_share: f64,
    pub items: Vec<SearchHit>,
}

fn decode_cursor(cursor: &str) -> Option<(f64, &str)> {
    let (score, uri) = cursor.split_once(':')?;
    Some((score.parse().ok()?, uri))
}

pub async fn search(pool: &PgPool, query: &SearchQuery) -> Result<SearchPage, sqlx::Error> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let min_votes = query
        .min_votes
        .unwrap_or(loom_core::confidence::MIN_VOTES as i64);
    let pattern = query.q.as_ref().map(|q| format!("%{q}%"));
    let (cursor_score, cursor_uri) = query
        .cursor
        .as_deref()
        .and_then(decode_cursor)
        .map(|(s, u)| (Some(s), Some(u.to_string())))
        .unwrap_or((None, None));

    let rows: Vec<(String, String, String, String, String, f64)> = sqlx::query_as(
        "with scored as (
            select f.uri, f.display_name, f.description, f.source, f.ranking_mode,
                   ((case when $1::text is null then false
                          else f.display_name ilike $1 or f.description ilike $1 end)::int::float8
                    + (case when $1::text is null then false
                            else f.display_name ilike $1 end)::int::float8) as score
            from feeds f
            where f.alive = true
              and ($2::text is null or f.source = $2)
              and ($3::text is null or f.ranking_mode = $3)
              and ($4::text is null or $4 = any(f.lang))
              and ($1::text is null or f.display_name ilike $1 or f.description ilike $1)
              and ($5::text is null or exists (
                    select 1 from tag_votes t
                    where t.feed_uri = f.uri and t.tag = $5 and t.confidence >= 60
                    group by t.feed_uri
                    having count(distinct t.contributor_key) >= $6))
        )
        select uri, display_name, description, source, ranking_mode, score
        from scored
        where ($7::float8 is null or score < $7
               or (score = $7 and uri > $8))
        order by score desc, uri asc
        limit $9",
    )
    .bind(pattern.as_deref())
    .bind(query.source.as_deref())
    .bind(query.mode.as_deref())
    .bind(query.lang.as_deref())
    .bind(query.tag.as_deref())
    .bind(min_votes)
    .bind(cursor_score)
    .bind(cursor_uri)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let items: Vec<SearchHit> = rows
        .into_iter()
        .map(
            |(uri, name, description, source, ranking_mode, score)| SearchHit {
                uri,
                name,
                description,
                source,
                ranking_mode,
                score,
            },
        )
        .collect();
    let next_cursor = if items.len() as i64 == limit {
        let last = items.last().expect("non-empty page");
        Some(format!("{}:{}", last.score, last.uri))
    } else {
        None
    };
    Ok(SearchPage { items, next_cursor })
}

pub async fn hidden_gems(pool: &PgPool, limit: i64) -> Result<HiddenGems, sqlx::Error> {
    let median: (Option<f64>,) = sqlx::query_as(
        "select percentile_cont(0.5) within group (order by coalesce(max_posts,0))
         from feeds where alive = true",
    )
    .fetch_one(pool)
    .await?;
    let cutoff = median.0.unwrap_or(0.0);
    let total: (i64,) = sqlx::query_as("select count(*) from feeds where alive = true")
        .fetch_one(pool)
        .await?;
    let tail: (i64,) = sqlx::query_as(
        "select count(*) from feeds where alive = true and coalesce(max_posts,0) <= $1",
    )
    .bind(cutoff)
    .fetch_one(pool)
    .await?;
    let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        "select uri, display_name, description, source, ranking_mode from feeds
         where alive = true and coalesce(max_posts,0) <= $1
         order by uri asc limit $2",
    )
    .bind(cutoff)
    .bind(limit.clamp(1, 200))
    .fetch_all(pool)
    .await?;
    Ok(HiddenGems {
        impression_share: if total.0 == 0 {
            0.0
        } else {
            tail.0 as f64 / total.0 as f64
        },
        items: rows
            .into_iter()
            .map(|(uri, name, description, source, ranking_mode)| SearchHit {
                uri,
                name,
                description,
                source,
                ranking_mode,
                score: 0.0,
            })
            .collect(),
    })
}
