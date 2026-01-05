use crate::db::{DbPool, create_multisig, find_multisig_by_id, list_user_multisigs};
use crate::errors::{AppError, AppResult};
use crate::models::{CreateMultisig, Multisig};

pub struct MultisigService;

impl MultisigService {
    pub async fn create_multisig(
        pool: &DbPool,
        multisig_data: CreateMultisig,
        created_by: i64,
    ) -> AppResult<Multisig> {
        if let Err(msg) = multisig_data.validate(created_by) {
            return Err(AppError::Validation(msg));
        }

        let multisig = create_multisig(pool, multisig_data, created_by).await?;

        if !multisig.is_valid_threshold() {
            return Err(AppError::Internal(
                "Created multisig has invalid threshold".to_string(),
            ));
        }

        Ok(multisig)
    }

    pub async fn get_multisig(pool: &DbPool, multisig_id: i64) -> AppResult<Multisig> {
        find_multisig_by_id(pool, multisig_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Multisig not found".to_string()))
    }

    pub async fn list_user_multisigs(pool: &DbPool, user_id: i64) -> AppResult<Vec<Multisig>> {
        list_user_multisigs(pool, user_id).await
    }

    pub async fn check_user_is_owner(
        pool: &DbPool,
        multisig_id: i64,
        user_id: i64,
    ) -> AppResult<Multisig> {
        let multisig = Self::get_multisig(pool, multisig_id).await?;

        if !multisig.is_owner(user_id) {
            return Err(AppError::Authorization(
                "User is not an owner of this multisig".to_string(),
            ));
        }

        Ok(multisig)
    }
}
