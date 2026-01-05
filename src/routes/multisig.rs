use actix_web::{HttpResponse, Result as ActixResult, get, post, web};
use serde::{Deserialize, Serialize};

use crate::auth_middleware::AuthUser;
use crate::db::DbPool;
use crate::models::CreateMultisig;
use crate::services::MultisigService;

#[derive(Deserialize)]
pub struct CreateMultisigRequest {
    pub name: String,
    pub description: Option<String>,
    pub owners: Vec<i64>,
    pub threshold: i32,
}

#[derive(Serialize)]
pub struct MultisigResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_by: i64,
    pub owners: Vec<i64>,
    pub threshold: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[post("")]
pub async fn create_multisig(
    pool: web::Data<DbPool>,
    user: AuthUser,
    req: web::Json<CreateMultisigRequest>,
) -> ActixResult<HttpResponse> {
    let create_data = CreateMultisig::new(
        req.name.clone(),
        req.description.clone(),
        req.owners.clone(),
        req.threshold,
    );

    let multisig = MultisigService::create_multisig(&pool, create_data, user.user_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response = MultisigResponse {
        id: multisig.id,
        name: multisig.name,
        description: multisig.description,
        created_by: multisig.created_by,
        owners: multisig.owners,
        threshold: multisig.threshold,
        created_at: multisig.created_at,
    };

    Ok(HttpResponse::Created().json(response))
}

#[get("")]
pub async fn list_multisigs(pool: web::Data<DbPool>, user: AuthUser) -> ActixResult<HttpResponse> {
    let multisigs = MultisigService::list_user_multisigs(&pool, user.user_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let responses: Vec<MultisigResponse> = multisigs
        .into_iter()
        .map(|m| MultisigResponse {
            id: m.id,
            name: m.name,
            description: m.description,
            created_by: m.created_by,
            owners: m.owners,
            threshold: m.threshold,
            created_at: m.created_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

#[get("/{id}")]
pub async fn get_multisig(
    pool: web::Data<DbPool>,
    user: AuthUser,
    path: web::Path<i64>,
) -> ActixResult<HttpResponse> {
    let multisig_id = path.into_inner();

    let multisig = MultisigService::check_user_is_owner(&pool, multisig_id, user.user_id)
        .await
        .map_err(actix_web::error::ErrorForbidden)?;

    let response = MultisigResponse {
        id: multisig.id,
        name: multisig.name,
        description: multisig.description,
        created_by: multisig.created_by,
        owners: multisig.owners,
        threshold: multisig.threshold,
        created_at: multisig.created_at,
    };

    Ok(HttpResponse::Ok().json(response))
}
