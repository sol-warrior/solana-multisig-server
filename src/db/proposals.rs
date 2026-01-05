use crate::db::DbPool;
use crate::errors::{AppError, AppResult};
use crate::models::{
    CreateProposal, Proposal, ProposalApproval, ProposalStatus, UpdateProposalStatus,
};
use chrono::{DateTime, Utc};
use sqlx::Row;

pub async fn create_proposal(
    pool: &DbPool,
    proposal_data: CreateProposal,
    multisig_id: i64,
    created_by: i64,
) -> AppResult<Proposal> {
    let row = sqlx::query(
        r#"
        INSERT INTO proposals (multisig_id, title, description, status, created_by, transaction_data)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, multisig_id, title, description, status, created_by, 
                 created_at::TIMESTAMPTZ as created_at, 
                 executed_at::TIMESTAMPTZ as executed_at, 
                 transaction_data
        "#,
    )
    .bind(multisig_id)
    .bind(&proposal_data.title)
    .bind(&proposal_data.description)
    .bind(ProposalStatus::Draft as ProposalStatus)
    .bind(created_by)
    .bind(&proposal_data.transaction_data)
    .map(|row: sqlx::postgres::PgRow| {
        (
            row.get::<i64, _>("id"),
            row.get::<i64, _>("multisig_id"),
            row.get::<String, _>("title"),
            row.get::<Option<String>, _>("description"),
            row.get::<ProposalStatus, _>("status"),
            row.get::<i64, _>("created_by"),
            row.get::<DateTime<Utc>, _>("created_at"),
            row.get::<Option<DateTime<Utc>>, _>("executed_at"),
            row.get::<Option<String>, _>("transaction_data"),
        )
    })
    .fetch_one(pool)
    .await?;

    Ok(Proposal::from_db(
        row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8,
    ))
}

pub async fn find_proposal_by_id(pool: &DbPool, proposal_id: i64) -> AppResult<Option<Proposal>> {
    let row = sqlx::query(
        r#"
        SELECT id, multisig_id, title, description, status, created_by, 
               created_at::TIMESTAMPTZ as created_at, 
               executed_at::TIMESTAMPTZ as executed_at, 
               transaction_data
        FROM proposals
        WHERE id = $1
        "#,
    )
    .bind(proposal_id)
    .map(|row: sqlx::postgres::PgRow| {
        (
            row.get::<i64, _>("id"),
            row.get::<i64, _>("multisig_id"),
            row.get::<String, _>("title"),
            row.get::<Option<String>, _>("description"),
            row.get::<ProposalStatus, _>("status"),
            row.get::<i64, _>("created_by"),
            row.get::<DateTime<Utc>, _>("created_at"),
            row.get::<Option<DateTime<Utc>>, _>("executed_at"),
            row.get::<Option<String>, _>("transaction_data"),
        )
    })
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Proposal::from_db(r.0, r.1, r.2, r.3, r.4, r.5, r.6, r.7, r.8)))
}

pub async fn list_multisig_proposals(pool: &DbPool, multisig_id: i64) -> AppResult<Vec<Proposal>> {
    let rows = sqlx::query(
        r#"
        SELECT id, multisig_id, title, description, status, created_by, 
               created_at::TIMESTAMPTZ as created_at, 
               executed_at::TIMESTAMPTZ as executed_at, 
               transaction_data
        FROM proposals
        WHERE multisig_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(multisig_id)
    .map(|row: sqlx::postgres::PgRow| {
        (
            row.get::<i64, _>("id"),
            row.get::<i64, _>("multisig_id"),
            row.get::<String, _>("title"),
            row.get::<Option<String>, _>("description"),
            row.get::<ProposalStatus, _>("status"),
            row.get::<i64, _>("created_by"),
            row.get::<DateTime<Utc>, _>("created_at"),
            row.get::<Option<DateTime<Utc>>, _>("executed_at"),
            row.get::<Option<String>, _>("transaction_data"),
        )
    })
    .fetch_all(pool)
    .await?;

    let proposals = rows
        .into_iter()
        .map(|r| Proposal::from_db(r.0, r.1, r.2, r.3, r.4, r.5, r.6, r.7, r.8))
        .collect();

    Ok(proposals)
}

pub async fn update_proposal_status(
    pool: &DbPool,
    proposal_id: i64,
    status_update: UpdateProposalStatus,
) -> AppResult<()> {
    let current_status = sqlx::query(
        r#"
        SELECT status FROM proposals WHERE id = $1
        "#,
    )
    .bind(proposal_id)
    .map(|row: sqlx::postgres::PgRow| row.get::<ProposalStatus, _>("status"))
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Proposal not found".to_string()))?;

    if !current_status.can_transition_to(status_update.status) {
        return Err(AppError::Validation(format!(
            "Invalid status transition from {:?} to {:?}",
            current_status, status_update.status
        )));
    }

    sqlx::query(
        r#"
        UPDATE proposals
        SET status = $2, executed_at = $3
        WHERE id = $1
        "#,
    )
    .bind(proposal_id)
    .bind(status_update.status as ProposalStatus)
    .bind(status_update.executed_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn approve_proposal(
    pool: &DbPool,
    proposal_id: i64,
    user_id: i64,
) -> AppResult<ProposalApproval> {
    let existing = sqlx::query(
        r#"
        SELECT id FROM proposal_approvals
        WHERE proposal_id = $1 AND user_id = $2
        "#,
    )
    .bind(proposal_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict(
            "User has already approved this proposal".to_string(),
        ));
    }

    let row = sqlx::query(
        r#"
        INSERT INTO proposal_approvals (proposal_id, user_id)
        VALUES ($1, $2)
        RETURNING id, proposal_id, user_id, approved_at::TIMESTAMPTZ as approved_at
        "#,
    )
    .bind(proposal_id)
    .bind(user_id)
    .map(|row: sqlx::postgres::PgRow| {
        (
            row.get::<i64, _>("id"),
            row.get::<i64, _>("proposal_id"),
            row.get::<i64, _>("user_id"),
            row.get::<DateTime<Utc>, _>("approved_at"),
        )
    })
    .fetch_one(pool)
    .await?;

    Ok(ProposalApproval::from_db(row.0, row.1, row.2, row.3))
}

pub async fn get_proposal_approvals(
    pool: &DbPool,
    proposal_id: i64,
) -> AppResult<Vec<ProposalApproval>> {
    let rows = sqlx::query(
        r#"
        SELECT id, proposal_id, user_id, approved_at::TIMESTAMPTZ as approved_at
        FROM proposal_approvals
        WHERE proposal_id = $1
        ORDER BY approved_at ASC
        "#,
    )
    .bind(proposal_id)
    .map(|row: sqlx::postgres::PgRow| {
        (
            row.get::<i64, _>("id"),
            row.get::<i64, _>("proposal_id"),
            row.get::<i64, _>("user_id"),
            row.get::<DateTime<Utc>, _>("approved_at"),
        )
    })
    .fetch_all(pool)
    .await?;

    let approvals = rows
        .into_iter()
        .map(|r| ProposalApproval::from_db(r.0, r.1, r.2, r.3))
        .collect();

    Ok(approvals)
}

pub async fn count_proposal_approvals(pool: &DbPool, proposal_id: i64) -> AppResult<i64> {
    let count = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM proposal_approvals
        WHERE proposal_id = $1
        "#,
    )
    .bind(proposal_id)
    .map(|row: sqlx::postgres::PgRow| row.get::<i64, _>("count"))
    .fetch_one(pool)
    .await?;

    Ok(count)
}
