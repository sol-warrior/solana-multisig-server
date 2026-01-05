use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Multisig {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_by: i64,
    pub owners: Vec<i64>,
    pub threshold: i32,
    pub created_at: DateTime<Utc>,
}

impl Multisig {
    pub fn from_db(
        id: i64,
        name: String,
        description: Option<String>,
        created_by: i64,
        owners: Vec<i64>,
        threshold: i32,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            created_by,
            owners,
            threshold,
            created_at,
        }
    }

    pub fn is_owner(&self, user_id: i64) -> bool {
        self.owners.contains(&user_id)
    }

    pub fn is_valid_threshold(&self) -> bool {
        self.threshold > 0 && self.threshold as usize <= self.owners.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMultisig {
    pub name: String,
    pub description: Option<String>,
    pub owners: Vec<i64>,
    pub threshold: i32,
}

impl CreateMultisig {
    pub fn new(
        name: String,
        description: Option<String>,
        owners: Vec<i64>,
        threshold: i32,
    ) -> Self {
        Self {
            name,
            description,
            owners,
            threshold,
        }
    }

    pub fn validate(&self, creator_id: i64) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Multisig name cannot be empty".to_string());
        }

        if self.owners.is_empty() {
            return Err("Multisig must have at least one owner".to_string());
        }

        if !self.owners.contains(&creator_id) {
            return Err("Creator must be included in owners list".to_string());
        }

        if self.threshold <= 0 {
            return Err("Threshold must be greater than 0".to_string());
        }

        if self.threshold as usize > self.owners.len() {
            return Err("Threshold cannot exceed number of owners".to_string());
        }

        Ok(())
    }
}
