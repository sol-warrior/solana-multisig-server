use actix_web::{HttpResponse, Responder, post, web};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize)]
struct Claims {
    sub: i64,
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

    // create JWT
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not set");
    let token = encode(
        &Header::default(),
        &Claims { sub: user_id },
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();

    HttpResponse::Created().json(json!({
        "user_id": user_id,
        "token": token
    }))
}
