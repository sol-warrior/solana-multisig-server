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
use crate::db::{create_user, find_user_by_id, get_user_password_hash, update_user_login};
use crate::models::{CreateUser, UpdateUserLogin};
use chrono::Utc;

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
    pub sub: i64,
    pub exp: usize,
}

#[post("/register")]
pub async fn register(pool: web::Data<PgPool>, body: web::Json<RegisterRequest>) -> impl Responder {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|_| {
            HttpResponse::InternalServerError().json(json!({"error": "Password hashing failed"}))
        })
        .unwrap()
        .to_string();

    let create_user_data = CreateUser::new(body.email.clone(), password_hash);

    match create_user(&pool, create_user_data).await {
        Ok(user) => {
            let expiration = Utc::now()
                .checked_add_signed(chrono::Duration::hours(24))
                .unwrap()
                .timestamp() as usize;

            let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not set");
            let token = encode(
                &Header::default(),
                &Claims {
                    sub: user.id,
                    exp: expiration,
                },
                &EncodingKey::from_secret(secret.as_bytes()),
            )
            .map_err(|_| {
                HttpResponse::InternalServerError().json(json!({"error": "Token creation failed"}))
            })
            .unwrap();

            HttpResponse::Created().json(json!({
                "user_id": user.id,
                "token": token
            }))
        }
        Err(crate::errors::AppError::Database(sqlx::Error::Database(db_err))) => {
            if db_err.constraint().is_some() {
                HttpResponse::Conflict().json(json!({"error": "Email already exists"}))
            } else {
                HttpResponse::InternalServerError().json(json!({"error": "Database error"}))
            }
        }
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Registration failed"})),
    }
}

#[post("/login")]
pub async fn login(pool: web::Data<PgPool>, body: web::Json<LoginRequest>) -> impl Responder {
    let user_data = match get_user_password_hash(&pool, &body.email).await {
        Ok(Some((user_id, password_hash))) => (user_id, password_hash),
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(json!({"error": "Invalid email or password"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error": "Server error"}));
        }
    };

    let parsed_hash = match PasswordHash::new(&user_data.1) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error": "Server error"}));
        }
    };

    if Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return HttpResponse::Unauthorized().json(json!({"error": "Invalid email or password"}));
    }

    let login_update = UpdateUserLogin {
        last_login_at: Utc::now(),
    };

    if update_user_login(&pool, user_data.0, login_update)
        .await
        .is_err()
    {
        eprintln!("Failed to update last_login_at for user {}", user_data.0);
    }

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &Claims {
            sub: user_data.0,
            exp: expiration,
        },
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    ) {
        Ok(token) => token,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Token creation failed"}));
        }
    };

    HttpResponse::Ok().json(json!({
        "user_id": user_data.0,
        "token": token
    }))
}

#[get("/me")]
pub async fn me(pool: web::Data<PgPool>, user: AuthUser) -> impl Responder {
    match find_user_by_id(&pool, user.user_id).await {
        Ok(Some(user_data)) => HttpResponse::Ok().json(serde_json::json!({
            "id": user_data.id,
            "email": user_data.email,
            "created_at": user_data.created_at,
            "last_login_at": user_data.last_login_at
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "User not found"})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Server error"})),
    }
}
