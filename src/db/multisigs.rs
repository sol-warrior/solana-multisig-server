use crate::db::DbPool;
use crate::errors::AppResult;
use crate::models::{CreateMultisig, Multisig};
use chrono::{DateTime, Utc};
use sqlx::Row;

pub async fn create_multisig(
    pool: &DbPool,
    multisig_data: CreateMultisig,
    created_by: i64,
) -> AppResult<Multisig> {
    let row = sqlx::query(
        r#"
        INSERT INTO multisigs (name, description, created_by, owners, threshold)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, description, created_by, owners, threshold, created_at::TIMESTAMPTZ as created_at
        "#,
    )
    .bind(&multisig_data.name)
    .bind(&multisig_data.description)
    .bind(created_by)
    .bind(&multisig_data.owners)
    .bind(multisig_data.threshold)
    .map(|row: sqlx::postgres::PgRow| {
        (
            row.get::<i64, _>("id"),
            row.get::<String, _>("name"),
            row.get::<Option<String>, _>("description"),
            row.get::<i64, _>("created_by"),
            row.get::<Vec<i64>, _>("owners"),
            row.get::<i32, _>("threshold"),
            row.get::<DateTime<Utc>, _>("created_at"),
        )
    })
    .fetch_one(pool)
    .await?;

    Ok(Multisig::from_db(
        row.0, row.1, row.2, row.3, row.4, row.5, row.6,
    ))
}

pub async fn find_multisig_by_id(pool: &DbPool, multisig_id: i64) -> AppResult<Option<Multisig>> {
    let row = sqlx::query(
        r#"
        SELECT id, name, description, created_by, owners, threshold, created_at::TIMESTAMPTZ as created_at
        FROM multisigs
        WHERE id = $1
        "#,
    )
    .bind(multisig_id)
    .map(|row: sqlx::postgres::PgRow| {
        (
            row.get::<i64, _>("id"),
            row.get::<String, _>("name"),
            row.get::<Option<String>, _>("description"),
            row.get::<i64, _>("created_by"),
            row.get::<Vec<i64>, _>("owners"),
            row.get::<i32, _>("threshold"),
            row.get::<DateTime<Utc>, _>("created_at"),
        )
    })
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Multisig::from_db(r.0, r.1, r.2, r.3, r.4, r.5, r.6)))
}

pub async fn list_user_multisigs(pool: &DbPool, user_id: i64) -> AppResult<Vec<Multisig>> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, description, created_by, owners, threshold, created_at::TIMESTAMPTZ as created_at
        FROM multisigs
        WHERE $1 = ANY(owners)
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .map(|row: sqlx::postgres::PgRow| {
        (
            row.get::<i64, _>("id"),
            row.get::<String, _>("name"),
            row.get::<Option<String>, _>("description"),
            row.get::<i64, _>("created_by"),
            row.get::<Vec<i64>, _>("owners"),
            row.get::<i32, _>("threshold"),
            row.get::<DateTime<Utc>, _>("created_at"),
        )
    })
    .fetch_all(pool)
    .await?;

    let multisigs = rows
        .into_iter()
        .map(|r| Multisig::from_db(r.0, r.1, r.2, r.3, r.4, r.5, r.6))
        .collect();

    Ok(multisigs)
}
