use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "proposal_status", rename_all = "snake_case")]
pub enum ProposalStatus {
    Draft,
    Active,
    Approved,
    Executed,
    Expired,
    Rejected,
}

impl ProposalStatus {
    pub fn valid_transitions(&self) -> Vec<ProposalStatus> {
        match self {
            ProposalStatus::Draft => vec![ProposalStatus::Active, ProposalStatus::Rejected],
            ProposalStatus::Active => vec![
                ProposalStatus::Approved,
                ProposalStatus::Expired,
                ProposalStatus::Rejected,
            ],
            ProposalStatus::Approved => vec![ProposalStatus::Executed],
            ProposalStatus::Executed => vec![],
            ProposalStatus::Expired => vec![],
            ProposalStatus::Rejected => vec![],
        }
    }

    pub fn can_transition_to(&self, new_status: ProposalStatus) -> bool {
        self.valid_transitions().contains(&new_status)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: i64,
    pub multisig_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: ProposalStatus,
    pub created_by: i64,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub transaction_data: Option<String>,
}

impl Proposal {
    pub fn from_db(
        id: i64,
        multisig_id: i64,
        title: String,
        description: Option<String>,
        status: ProposalStatus,
        created_by: i64,
        created_at: DateTime<Utc>,
        executed_at: Option<DateTime<Utc>>,
        transaction_data: Option<String>,
    ) -> Self {
        Self {
            id,
            multisig_id,
            title,
            description,
            status,
            created_by,
            created_at,
            executed_at,
            transaction_data,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            ProposalStatus::Executed | ProposalStatus::Expired | ProposalStatus::Rejected
        )
    }

    pub fn can_be_approved(&self) -> bool {
        self.status == ProposalStatus::Active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalApproval {
    pub id: i64,
    pub proposal_id: i64,
    pub user_id: i64,
    pub approved_at: DateTime<Utc>,
}

impl ProposalApproval {
    pub fn from_db(id: i64, proposal_id: i64, user_id: i64, approved_at: DateTime<Utc>) -> Self {
        Self {
            id,
            proposal_id,
            user_id,
            approved_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProposal {
    pub title: String,
    pub description: Option<String>,
    pub transaction_data: Option<String>,
}

impl CreateProposal {
    pub fn new(
        title: String,
        description: Option<String>,
        transaction_data: Option<String>,
    ) -> Self {
        Self {
            title,
            description,
            transaction_data,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Proposal title cannot be empty".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UpdateProposalStatus {
    pub status: ProposalStatus,
    pub executed_at: Option<DateTime<Utc>>,
}
