use actix_web::{HttpResponse, Responder, get, post, web};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

use crate::auth_middleware::AuthUser;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,   // user_id
    pub exp: usize, // expiration timestamp
}

#[post("/auth/register")]
pub async fn register(pool: web::Data<PgPool>, body: web::Json<RegisterRequest>) -> impl Responder {
    let email = body.email.to_lowercase();

    // hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed = argon2
        .hash_password(body.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    // insert into DB
    let row = sqlx::query!(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        RETURNING id
        "#,
        email,
        hashed
    )
    .fetch_one(pool.get_ref())
    .await;

    if let Err(_) = row {
        return HttpResponse::Conflict().json(json!({"error": "Email already exists"}));
    }

    let user_id = row.unwrap().id;
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    // create JWT
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not set");
    let token = encode(
        &Header::default(),
        &Claims {
            sub: user_id,
            exp: expiration,
        },
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();

    HttpResponse::Created().json(json!({
        "user_id": user_id,
        "token": token
    }))
}

#[post("/auth/login")]
pub async fn login(pool: web::Data<PgPool>, body: web::Json<LoginRequest>) -> impl Responder {
    let email = body.email.to_lowercase();

    // 1. Fetch user from DB
    let row = sqlx::query!(
        r#"
        SELECT id, password_hash
        FROM users
        WHERE email = $1
        "#,
        email
    )
    .fetch_optional(pool.get_ref())
    .await;

    if let Err(_) = row {
        return HttpResponse::InternalServerError().json(json!({"error": "Server error"}));
    }

    let user = match row.unwrap() {
        Some(u) => u,
        None => {
            return HttpResponse::Unauthorized()
                .json(json!({"error": "Invalid email or password"}));
        }
    };

    // 2. Verify password
    let parsed_hash = argon2::PasswordHash::new(&user.password_hash).unwrap();
    if Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return HttpResponse::Unauthorized().json(json!({"error": "Invalid email or password"}));
    }

    // 3. Update last_login_at
    let _ = sqlx::query!(
        r#"
         UPDATE users
         SET last_login_at = NOW()
         WHERE id = $1
         "#,
        user.id
    )
    .execute(pool.get_ref())
    .await;

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    // 4. Create JWT
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &Claims {
            sub: user.id,
            exp: expiration,
        },
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();
    HttpResponse::Ok().json(json!({
        "user_id": user.id,
        "token": token
    }))
}

#[get("/auth/me")]
pub async fn me(pool: web::Data<PgPool>, user: AuthUser) -> impl Responder {
    let row = sqlx::query!(
        r#"
        SELECT id, email, created_at, last_login_at
        FROM users
        WHERE id = $1
        "#,
        user.user_id
    )
    .fetch_one(pool.get_ref())
    .await;

    if let Err(_) = row {
        return HttpResponse::InternalServerError().finish();
    }

    let u = row.unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "id": u.id,
        "email": u.email,
        "created_at": u.created_at,
        "last_login_at": u.last_login_at
    }))
}
