//! Bundle persistence. The edit token is never stored in cleartext.

use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleView {
    pub id: Uuid,
    pub owner_key: String,
    pub name: String,
    pub description: String,
    pub uris: Vec<String>,
    pub version: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedBundle {
    pub view: BundleView,
    pub edit_token: String,
}

fn hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

async fn uris_exist(conn: &PgPool, uris: &[String]) -> Result<(), Error> {
    let rows: Vec<(String,)> = sqlx::query_as("select uri from feeds where uri = any($1)")
        .bind(uris)
        .fetch_all(conn)
        .await?;
    if rows.len() != uris.len() {
        return Err(Error::Invalid("bundle references an unknown feed".into()));
    }
    Ok(())
}

async fn set_feeds(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: Uuid,
    uris: &[String],
) -> Result<(), Error> {
    sqlx::query("delete from bundle_feeds where bundle_id = $1")
        .bind(id)
        .execute(&mut **tx)
        .await?;
    for (position, uri) in uris.iter().enumerate() {
        sqlx::query("insert into bundle_feeds (bundle_id, feed_uri, position) values ($1,$2,$3)")
            .bind(id)
            .bind(uri)
            .bind(position as i32)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

pub async fn create(
    pool: &PgPool,
    owner_key: &str,
    name: &str,
    description: &str,
    uris: Vec<String>,
    parent_id: Option<Uuid>,
) -> Result<CreatedBundle, Error> {
    let provisional = loom_core::bundle::Bundle::new(name, uris.clone())
        .map_err(|e| Error::Invalid(e.to_string()))?;
    uris_exist(pool, &uris).await?;
    let id = Uuid::new_v4();
    let mut tx = pool.begin().await?;
    sqlx::query(
        "insert into bundles (id, owner_key, name, description, edit_token_hash, parent_id)
         values ($1,$2,$3,$4,$5,$6)",
    )
    .bind(id)
    .bind(owner_key)
    .bind(name)
    .bind(description)
    .bind(hash(&provisional.edit_token))
    .bind(parent_id)
    .execute(&mut *tx)
    .await?;
    set_feeds(&mut tx, id, &uris).await?;
    sqlx::query("insert into events (kind, payload) values ('bundle_created', $1)")
        .bind(serde_json::json!({ "id": id, "owner_key": owner_key }))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(CreatedBundle {
        view: BundleView {
            id,
            owner_key: owner_key.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            uris,
            version: 1,
        },
        edit_token: provisional.edit_token,
    })
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Option<BundleView>, Error> {
    let row: Option<(String, String, String, i32)> = sqlx::query_as(
        "select owner_key, name, description, version from bundles where id = $1 and deleted_at is null",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let Some((owner_key, name, description, version)) = row else {
        return Ok(None);
    };
    let uris: Vec<(String,)> =
        sqlx::query_as("select feed_uri from bundle_feeds where bundle_id = $1 order by position")
            .bind(id)
            .fetch_all(pool)
            .await?;
    Ok(Some(BundleView {
        id,
        owner_key,
        name,
        description,
        uris: uris.into_iter().map(|r| r.0).collect(),
        version,
    }))
}

pub async fn edit(
    pool: &PgPool,
    id: Uuid,
    token: &str,
    name: &str,
    description: &str,
    uris: Vec<String>,
) -> Result<BundleView, Error> {
    let stored: Option<Vec<u8>> = sqlx::query_scalar(
        "select edit_token_hash from bundles where id = $1 and deleted_at is null",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let Some(stored) = stored else {
        return Err(Error::NotFound);
    };
    if stored != hash(token) {
        return Err(Error::Auth);
    }
    loom_core::bundle::Bundle::new(name, uris.clone())
        .map_err(|e| Error::Invalid(e.to_string()))?;
    uris_exist(pool, &uris).await?;
    let replacement = loom_core::bundle::Bundle::new(name, uris.clone())
        .map_err(|e| Error::Invalid(e.to_string()))?;
    let mut tx = pool.begin().await?;
    sqlx::query(
        "update bundles set name=$2, description=$3, edit_token_hash=$4, version=version+1,
         updated_at=now() where id=$1",
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(hash(&replacement.edit_token))
    .execute(&mut *tx)
    .await?;
    set_feeds(&mut tx, id, &uris).await?;
    sqlx::query("insert into events (kind, payload) values ('bundle_edited', $1)")
        .bind(serde_json::json!({ "id": id }))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    let view = get(pool, id).await?.expect("bundle just edited");
    Ok(view)
}

pub async fn list(
    pool: &PgPool,
    owner_key: Option<&str>,
    cursor: Option<Uuid>,
    limit: i64,
) -> Result<Vec<BundleView>, Error> {
    let limit = limit.clamp(1, 200);
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "select id from bundles
         where deleted_at is null
           and ($1::text is null or owner_key = $1)
           and ($2::uuid is null or id > $2)
         order by id limit $3",
    )
    .bind(owner_key)
    .bind(cursor)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    let mut out = Vec::new();
    for (id,) in rows {
        if let Some(view) = get(pool, id).await? {
            out.push(view);
        }
    }
    Ok(out)
}
