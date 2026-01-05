use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn from_db(
        id: i64,
        email: String,
        created_at: DateTime<Utc>,
        last_login_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            email,
            created_at,
            last_login_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateUser {
    pub email: String,
    pub password_hash: String,
}

impl CreateUser {
    pub fn new(email: String, password_hash: String) -> Self {
        Self {
            email: email.to_lowercase(),
            password_hash,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpdateUserLogin {
    pub last_login_at: DateTime<Utc>,
}
