use crate::db::{
    DbPool, approve_proposal, count_proposal_approvals, create_proposal, find_proposal_by_id,
    get_proposal_approvals, list_multisig_proposals, update_proposal_status,
};
use crate::errors::{AppError, AppResult};
use crate::models::{
    CreateProposal, Proposal, ProposalApproval, ProposalStatus, UpdateProposalStatus,
};
use crate::services::MultisigService;
use chrono::Utc;

pub struct ProposalService;

impl ProposalService {
    pub async fn create_proposal(
        pool: &DbPool,
        proposal_data: CreateProposal,
        multisig_id: i64,
        created_by: i64,
    ) -> AppResult<Proposal> {
        if let Err(msg) = proposal_data.validate() {
            return Err(AppError::Validation(msg));
        }

        MultisigService::check_user_is_owner(pool, multisig_id, created_by).await?;

        create_proposal(pool, proposal_data, multisig_id, created_by).await
    }

    pub async fn get_proposal(pool: &DbPool, proposal_id: i64) -> AppResult<Proposal> {
        find_proposal_by_id(pool, proposal_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Proposal not found".to_string()))
    }

    pub async fn list_multisig_proposals(
        pool: &DbPool,
        multisig_id: i64,
        user_id: i64,
    ) -> AppResult<Vec<Proposal>> {
        MultisigService::check_user_is_owner(pool, multisig_id, user_id).await?;

        list_multisig_proposals(pool, multisig_id).await
    }

    pub async fn activate_proposal(
        pool: &DbPool,
        proposal_id: i64,
        user_id: i64,
    ) -> AppResult<Proposal> {
        let proposal = Self::get_proposal(pool, proposal_id).await?;

        MultisigService::check_user_is_owner(pool, proposal.multisig_id, user_id).await?;

        if proposal.status != ProposalStatus::Draft {
            return Err(AppError::Validation(format!(
                "Cannot activate proposal with status {:?}",
                proposal.status
            )));
        }

        let status_update = UpdateProposalStatus {
            status: ProposalStatus::Active,
            executed_at: None,
        };

        update_proposal_status(pool, proposal_id, status_update).await?;

        Self::get_proposal(pool, proposal_id).await
    }

    pub async fn approve_proposal(
        pool: &DbPool,
        proposal_id: i64,
        user_id: i64,
    ) -> AppResult<(ProposalApproval, Proposal)> {
        let proposal = Self::get_proposal(pool, proposal_id).await?;

        MultisigService::check_user_is_owner(pool, proposal.multisig_id, user_id).await?;

        if !proposal.can_be_approved() {
            return Err(AppError::Validation(format!(
                "Proposal with status {:?} cannot be approved",
                proposal.status
            )));
        }

        let approval = approve_proposal(pool, proposal_id, user_id).await?;

        let multisig = MultisigService::get_multisig(pool, proposal.multisig_id).await?;
        let approval_count = count_proposal_approvals(pool, proposal_id).await?;

        let updated_proposal = if approval_count >= multisig.threshold as i64 {
            let status_update = UpdateProposalStatus {
                status: ProposalStatus::Approved,
                executed_at: None,
            };
            update_proposal_status(pool, proposal_id, status_update).await?;
            Self::get_proposal(pool, proposal_id).await?
        } else {
            proposal
        };

        Ok((approval, updated_proposal))
    }

    pub async fn execute_proposal(
        pool: &DbPool,
        proposal_id: i64,
        user_id: i64,
    ) -> AppResult<Proposal> {
        let proposal = Self::get_proposal(pool, proposal_id).await?;

        MultisigService::check_user_is_owner(pool, proposal.multisig_id, user_id).await?;

        if proposal.status != ProposalStatus::Approved {
            return Err(AppError::Validation(format!(
                "Cannot execute proposal with status {:?}",
                proposal.status
            )));
        }

        let status_update = UpdateProposalStatus {
            status: ProposalStatus::Executed,
            executed_at: Some(Utc::now()),
        };

        update_proposal_status(pool, proposal_id, status_update).await?;

        Self::get_proposal(pool, proposal_id).await
    }

    pub async fn reject_proposal(
        pool: &DbPool,
        proposal_id: i64,
        user_id: i64,
    ) -> AppResult<Proposal> {
        let proposal = Self::get_proposal(pool, proposal_id).await?;

        MultisigService::check_user_is_owner(pool, proposal.multisig_id, user_id).await?;

        if !matches!(
            proposal.status,
            ProposalStatus::Draft | ProposalStatus::Active
        ) {
            return Err(AppError::Validation(format!(
                "Cannot reject proposal with status {:?}",
                proposal.status
            )));
        }

        let status_update = UpdateProposalStatus {
            status: ProposalStatus::Rejected,
            executed_at: None,
        };

        update_proposal_status(pool, proposal_id, status_update).await?;

        Self::get_proposal(pool, proposal_id).await
    }

    pub async fn get_proposal_approvals(
        pool: &DbPool,
        proposal_id: i64,
        user_id: i64,
    ) -> AppResult<Vec<ProposalApproval>> {
        let proposal = Self::get_proposal(pool, proposal_id).await?;
        MultisigService::check_user_is_owner(pool, proposal.multisig_id, user_id).await?;

        get_proposal_approvals(pool, proposal_id).await
    }
}
