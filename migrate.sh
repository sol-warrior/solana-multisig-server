#!/bin/bash

set -e

if [ -z "$DATABASE_URL" ]; then
    echo "Error: DATABASE_URL environment variable is not set"
    echo "Example: export DATABASE_URL=postgresql://postgres:password123@localhost:5432/multisig_db"
    exit 1
fi

echo "Running migrations..."

psql "$DATABASE_URL" -f migrations/000_users_table.sql
psql "$DATABASE_URL" -f migrations/001_initial_schema.sql
psql "$DATABASE_URL" -f migrations/002_fix_timestamp_types.sql

echo "Migrations completed successfully!"

