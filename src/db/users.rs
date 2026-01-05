use crate::db::DbPool;
use crate::errors::AppResult;
use crate::models::{CreateUser, UpdateUserLogin, User};
use chrono::{DateTime, Utc};
use sqlx::Row;

pub async fn create_user(pool: &DbPool, user_data: CreateUser) -> AppResult<User> {
    let row = sqlx::query(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        RETURNING id, email, 
                 created_at::TIMESTAMPTZ as created_at,
                 last_login_at::TIMESTAMPTZ as last_login_at
        "#,
    )
    .bind(&user_data.email)
    .bind(&user_data.password_hash)
    .map(|row: sqlx::postgres::PgRow| {
        (
            row.get::<i64, _>("id"),
            row.get::<String, _>("email"),
            row.get::<DateTime<Utc>, _>("created_at"),
            row.get::<Option<DateTime<Utc>>, _>("last_login_at"),
        )
    })
    .fetch_one(pool)
    .await?;

    Ok(User::from_db(row.0, row.1, row.2, row.3))
}

pub async fn find_user_by_email(pool: &DbPool, email: &str) -> AppResult<Option<User>> {
    let row = sqlx::query(
        r#"
        SELECT id, email, 
               created_at::TIMESTAMPTZ as created_at,
               last_login_at::TIMESTAMPTZ as last_login_at
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email.to_lowercase())
    .map(|row: sqlx::postgres::PgRow| {
        (
            row.get::<i64, _>("id"),
            row.get::<String, _>("email"),
            row.get::<DateTime<Utc>, _>("created_at"),
            row.get::<Option<DateTime<Utc>>, _>("last_login_at"),
        )
    })
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| User::from_db(r.0, r.1, r.2, r.3)))
}

pub async fn find_user_by_id(pool: &DbPool, user_id: i64) -> AppResult<Option<User>> {
    let row = sqlx::query(
        r#"
        SELECT id, email,
               created_at::TIMESTAMPTZ as created_at,
               last_login_at::TIMESTAMPTZ as last_login_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .map(|row: sqlx::postgres::PgRow| {
        (
            row.get::<i64, _>("id"),
            row.get::<String, _>("email"),
            row.get::<DateTime<Utc>, _>("created_at"),
            row.get::<Option<DateTime<Utc>>, _>("last_login_at"),
        )
    })
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| User::from_db(r.0, r.1, r.2, r.3)))
}

pub async fn update_user_login(
    pool: &DbPool,
    user_id: i64,
    login_data: UpdateUserLogin,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE users
        SET last_login_at = $2
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(login_data.last_login_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_password_hash(
    pool: &DbPool,
    email: &str,
) -> AppResult<Option<(i64, String)>> {
    let row = sqlx::query!(
        r#"
        SELECT id, password_hash
        FROM users
        WHERE email = $1
        "#,
        email.to_lowercase()
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| (r.id, r.password_hash)))
}
