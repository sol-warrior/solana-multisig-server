use actix_web::{Error, FromRequest, HttpRequest, dev::Payload};
use futures_util::future::{Ready, ready};
use jsonwebtoken::{DecodingKey, Validation, decode};

#[derive(Debug)]
pub struct AuthUser {
    pub user_id: i64,
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let header = req.headers().get("Authorization");

        if header.is_none() {
            return ready(Err(actix_web::error::ErrorUnauthorized("Missing token")));
        }

        let auth_header = header.unwrap().to_str().unwrap_or("");
        if !auth_header.starts_with("Bearer ") {
            return ready(Err(actix_web::error::ErrorUnauthorized(
                "Invalid token format",
            )));
        }

        let token = auth_header.trim_start_matches("Bearer ").trim();
        let secret = std::env::var("JWT_SECRET").unwrap();
        let validation = Validation::default();

        let decoded = decode::<crate::routes::auth::Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        );

        match decoded {
            Ok(data) => ready(Ok(AuthUser {
                user_id: data.claims.sub,
            })),
            Err(_) => ready(Err(actix_web::error::ErrorUnauthorized(
                "Invalid or expired token",
            ))),
        }
    }
}
