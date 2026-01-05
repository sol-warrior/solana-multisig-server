-- Initial database schema for Solana Multisig Server

-- Create custom enum type for proposal status
CREATE TYPE proposal_status AS ENUM ('draft', 'active', 'approved', 'executed', 'expired', 'rejected');

-- Multisigs table
CREATE TABLE multisigs (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_by BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    owners BIGINT[] NOT NULL CHECK (array_length(owners, 1) > 0),
    threshold INTEGER NOT NULL CHECK (threshold > 0),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Ensure threshold doesn't exceed number of owners
    CONSTRAINT threshold_not_exceeds_owners CHECK (threshold <= array_length(owners, 1)),
    -- Ensure created_by is in owners list
    CONSTRAINT creator_is_owner CHECK (created_by = ANY(owners))
);

-- Create index for multisig owners lookup
CREATE INDEX idx_multisigs_owners ON multisigs USING GIN (owners);
CREATE INDEX idx_multisigs_created_by ON multisigs (created_by);

-- Proposals table
CREATE TABLE proposals (
    id BIGSERIAL PRIMARY KEY,
    multisig_id BIGINT NOT NULL REFERENCES multisigs(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    status proposal_status NOT NULL DEFAULT 'draft',
    created_by BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    executed_at TIMESTAMP WITH TIME ZONE,
    transaction_data TEXT,

    -- Ensure creator is a multisig owner
    CONSTRAINT creator_is_multisig_owner CHECK (
        created_by IN (
            SELECT unnest(owners) FROM multisigs WHERE id = multisig_id
        )
    )
);

-- Create indexes for proposals
CREATE INDEX idx_proposals_multisig_id ON proposals (multisig_id);
CREATE INDEX idx_proposals_status ON proposals (status);
CREATE INDEX idx_proposals_created_by ON proposals (created_by);

-- Proposal approvals table
CREATE TABLE proposal_approvals (
    id BIGSERIAL PRIMARY KEY,
    proposal_id BIGINT NOT NULL REFERENCES proposals(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    approved_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Ensure approver is a multisig owner
    CONSTRAINT approver_is_multisig_owner CHECK (
        user_id IN (
            SELECT unnest(owners)
            FROM multisigs
            WHERE id = (SELECT multisig_id FROM proposals WHERE id = proposal_id)
        )
    ),

    -- One approval per user per proposal
    UNIQUE(proposal_id, user_id)
);

-- Create indexes for proposal approvals
CREATE INDEX idx_proposal_approvals_proposal_id ON proposal_approvals (proposal_id);
CREATE INDEX idx_proposal_approvals_user_id ON proposal_approvals (user_id);

