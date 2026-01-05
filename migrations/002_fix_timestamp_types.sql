-- Fix timestamp columns to use TIMESTAMP WITH TIME ZONE

-- Fix users table
ALTER TABLE users 
  ALTER COLUMN created_at TYPE TIMESTAMP WITH TIME ZONE USING created_at AT TIME ZONE 'UTC',
  ALTER COLUMN last_login_at TYPE TIMESTAMP WITH TIME ZONE USING last_login_at AT TIME ZONE 'UTC';

-- Fix multisigs table (if it exists)
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'multisigs') THEN
    ALTER TABLE multisigs 
      ALTER COLUMN created_at TYPE TIMESTAMP WITH TIME ZONE USING created_at AT TIME ZONE 'UTC';
  END IF;
END $$;

-- Fix proposals table (if it exists)
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'proposals') THEN
    ALTER TABLE proposals 
      ALTER COLUMN created_at TYPE TIMESTAMP WITH TIME ZONE USING created_at AT TIME ZONE 'UTC',
      ALTER COLUMN executed_at TYPE TIMESTAMP WITH TIME ZONE USING executed_at AT TIME ZONE 'UTC';
  END IF;
END $$;

-- Fix proposal_approvals table (if it exists)
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'proposal_approvals') THEN
    ALTER TABLE proposal_approvals 
      ALTER COLUMN approved_at TYPE TIMESTAMP WITH TIME ZONE USING approved_at AT TIME ZONE 'UTC';
  END IF;
END $$;

