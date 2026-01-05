use actix_web::{HttpResponse, Result as ActixResult, get, post, web};
use serde::{Deserialize, Serialize};

use crate::auth_middleware::AuthUser;
use crate::db::DbPool;
use crate::models::{CreateProposal, ProposalStatus};
use crate::services::ProposalService;

#[derive(Deserialize)]
pub struct CreateProposalRequest {
    pub title: String,
    pub description: Option<String>,
    pub transaction_data: Option<String>,
}

#[derive(Serialize)]
pub struct ProposalResponse {
    pub id: i64,
    pub multisig_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: ProposalStatus,
    pub created_by: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub executed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub transaction_data: Option<String>,
}

#[derive(Serialize)]
pub struct ProposalApprovalResponse {
    pub id: i64,
    pub proposal_id: i64,
    pub user_id: i64,
    pub approved_at: chrono::DateTime<chrono::Utc>,
}

#[post("")]
pub async fn create_proposal(
    pool: web::Data<DbPool>,
    user: AuthUser,
    path: web::Path<i64>,
    req: web::Json<CreateProposalRequest>,
) -> ActixResult<HttpResponse> {
    let multisig_id = path.into_inner();

    let create_data = CreateProposal::new(
        req.title.clone(),
        req.description.clone(),
        req.transaction_data.clone(),
    );

    let proposal = ProposalService::create_proposal(&pool, create_data, multisig_id, user.user_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response = ProposalResponse {
        id: proposal.id,
        multisig_id: proposal.multisig_id,
        title: proposal.title,
        description: proposal.description,
        status: proposal.status,
        created_by: proposal.created_by,
        created_at: proposal.created_at,
        executed_at: proposal.executed_at,
        transaction_data: proposal.transaction_data,
    };

    Ok(HttpResponse::Created().json(response))
}

#[get("")]
pub async fn list_proposals(
    pool: web::Data<DbPool>,
    user: AuthUser,
    path: web::Path<i64>,
) -> ActixResult<HttpResponse> {
    let multisig_id = path.into_inner();

    let proposals = ProposalService::list_multisig_proposals(&pool, multisig_id, user.user_id)
        .await
        .map_err(actix_web::error::ErrorForbidden)?;

    let responses: Vec<ProposalResponse> = proposals
        .into_iter()
        .map(|p| ProposalResponse {
            id: p.id,
            multisig_id: p.multisig_id,
            title: p.title,
            description: p.description,
            status: p.status,
            created_by: p.created_by,
            created_at: p.created_at,
            executed_at: p.executed_at,
            transaction_data: p.transaction_data,
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

#[get("/{id}")]
pub async fn get_proposal(
    pool: web::Data<DbPool>,
    user: AuthUser,
    path: web::Path<i64>,
) -> ActixResult<HttpResponse> {
    let proposal_id = path.into_inner();

    let proposal = ProposalService::get_proposal(&pool, proposal_id)
        .await
        .map_err(actix_web::error::ErrorNotFound)?;

    crate::services::MultisigService::check_user_is_owner(
        &pool,
        proposal.multisig_id,
        user.user_id,
    )
    .await
    .map_err(actix_web::error::ErrorForbidden)?;

    let response = ProposalResponse {
        id: proposal.id,
        multisig_id: proposal.multisig_id,
        title: proposal.title,
        description: proposal.description,
        status: proposal.status,
        created_by: proposal.created_by,
        created_at: proposal.created_at,
        executed_at: proposal.executed_at,
        transaction_data: proposal.transaction_data,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[post("/{id}/activate")]
pub async fn activate_proposal(
    pool: web::Data<DbPool>,
    user: AuthUser,
    path: web::Path<i64>,
) -> ActixResult<HttpResponse> {
    let proposal_id = path.into_inner();

    let proposal = ProposalService::activate_proposal(&pool, proposal_id, user.user_id)
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    let response = ProposalResponse {
        id: proposal.id,
        multisig_id: proposal.multisig_id,
        title: proposal.title,
        description: proposal.description,
        status: proposal.status,
        created_by: proposal.created_by,
        created_at: proposal.created_at,
        executed_at: proposal.executed_at,
        transaction_data: proposal.transaction_data,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[post("/{id}/approve")]
pub async fn approve_proposal(
    pool: web::Data<DbPool>,
    user: AuthUser,
    path: web::Path<i64>,
) -> ActixResult<HttpResponse> {
    let proposal_id = path.into_inner();

    let (approval, proposal) = ProposalService::approve_proposal(&pool, proposal_id, user.user_id)
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    let approval_response = ProposalApprovalResponse {
        id: approval.id,
        proposal_id: approval.proposal_id,
        user_id: approval.user_id,
        approved_at: approval.approved_at,
    };

    let proposal_response = ProposalResponse {
        id: proposal.id,
        multisig_id: proposal.multisig_id,
        title: proposal.title,
        description: proposal.description,
        status: proposal.status,
        created_by: proposal.created_by,
        created_at: proposal.created_at,
        executed_at: proposal.executed_at,
        transaction_data: proposal.transaction_data,
    };

    let response = serde_json::json!({
        "approval": approval_response,
        "proposal": proposal_response
    });

    Ok(HttpResponse::Ok().json(response))
}

#[post("/{id}/execute")]
pub async fn execute_proposal(
    pool: web::Data<DbPool>,
    user: AuthUser,
    path: web::Path<i64>,
) -> ActixResult<HttpResponse> {
    let proposal_id = path.into_inner();

    let proposal = ProposalService::execute_proposal(&pool, proposal_id, user.user_id)
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    let response = ProposalResponse {
        id: proposal.id,
        multisig_id: proposal.multisig_id,
        title: proposal.title,
        description: proposal.description,
        status: proposal.status,
        created_by: proposal.created_by,
        created_at: proposal.created_at,
        executed_at: proposal.executed_at,
        transaction_data: proposal.transaction_data,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[post("/{id}/reject")]
pub async fn reject_proposal(
    pool: web::Data<DbPool>,
    user: AuthUser,
    path: web::Path<i64>,
) -> ActixResult<HttpResponse> {
    let proposal_id = path.into_inner();

    let proposal = ProposalService::reject_proposal(&pool, proposal_id, user.user_id)
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    let response = ProposalResponse {
        id: proposal.id,
        multisig_id: proposal.multisig_id,
        title: proposal.title,
        description: proposal.description,
        status: proposal.status,
        created_by: proposal.created_by,
        created_at: proposal.created_at,
        executed_at: proposal.executed_at,
        transaction_data: proposal.transaction_data,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/{id}/approvals")]
pub async fn get_proposal_approvals(
    pool: web::Data<DbPool>,
    user: AuthUser,
    path: web::Path<i64>,
) -> ActixResult<HttpResponse> {
    let proposal_id = path.into_inner();

    let approvals = ProposalService::get_proposal_approvals(&pool, proposal_id, user.user_id)
        .await
        .map_err(actix_web::error::ErrorForbidden)?;

    let responses: Vec<ProposalApprovalResponse> = approvals
        .into_iter()
        .map(|a| ProposalApprovalResponse {
            id: a.id,
            proposal_id: a.proposal_id,
            user_id: a.user_id,
            approved_at: a.approved_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}
