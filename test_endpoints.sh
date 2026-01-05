#!/bin/bash

BASE_URL="http://127.0.0.1:8080"

echo "=== Testing Solana Multisig Server API ==="
echo ""

echo "1. Register User 1"
USER1_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user1@example.com",
    "password": "password123"
  }')
echo "$USER1_RESPONSE" | jq '.'
USER1_TOKEN=$(echo "$USER1_RESPONSE" | jq -r '.token')
USER1_ID=$(echo "$USER1_RESPONSE" | jq -r '.user_id')
echo "User 1 Token: $USER1_TOKEN"
echo "User 1 ID: $USER1_ID"
echo ""

echo "2. Register User 2"
USER2_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user2@example.com",
    "password": "password123"
  }')
echo "$USER2_RESPONSE" | jq '.'
USER2_TOKEN=$(echo "$USER2_RESPONSE" | jq -r '.token')
USER2_ID=$(echo "$USER2_RESPONSE" | jq -r '.user_id')
echo "User 2 Token: $USER2_TOKEN"
echo "User 2 ID: $USER2_ID"
echo ""

echo "3. Register User 3"
USER3_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user3@example.com",
    "password": "password123"
  }')
echo "$USER3_RESPONSE" | jq '.'
USER3_TOKEN=$(echo "$USER3_RESPONSE" | jq -r '.token')
USER3_ID=$(echo "$USER3_RESPONSE" | jq -r '.user_id')
echo "User 3 Token: $USER3_TOKEN"
echo "User 3 ID: $USER3_ID"
echo ""

echo "4. Login User 1"
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user1@example.com",
    "password": "password123"
  }')
echo "$LOGIN_RESPONSE" | jq '.'
echo ""

echo "5. Get Current User (User 1)"
curl -s -X GET "$BASE_URL/auth/me" \
  -H "Authorization: Bearer $USER1_TOKEN" | jq '.'
echo ""

echo "6. Create Multisig (User 1)"
MULTISIG_RESPONSE=$(curl -s -X POST "$BASE_URL/multisigs" \
  -H "Authorization: Bearer $USER1_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Team Multisig\",
    \"description\": \"Main team wallet\",
    \"owners\": [$USER1_ID, $USER2_ID, $USER3_ID],
    \"threshold\": 2
  }")
echo "$MULTISIG_RESPONSE" | jq '.'
MULTISIG_ID=$(echo "$MULTISIG_RESPONSE" | jq -r '.id')
echo "Multisig ID: $MULTISIG_ID"
echo ""

echo "7. List User's Multisigs (User 1)"
curl -s -X GET "$BASE_URL/multisigs" \
  -H "Authorization: Bearer $USER1_TOKEN" | jq '.'
echo ""

echo "8. Get Multisig Details"
curl -s -X GET "$BASE_URL/multisigs/$MULTISIG_ID" \
  -H "Authorization: Bearer $USER1_TOKEN" | jq '.'
echo ""

echo "9. Create Proposal (User 1)"
PROPOSAL_RESPONSE=$(curl -s -X POST "$BASE_URL/multisigs/$MULTISIG_ID/proposals" \
  -H "Authorization: Bearer $USER1_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Transfer 100 SOL",
    "description": "Transfer funds to new wallet",
    "transaction_data": "base64_encoded_transaction_data_here"
  }')
echo "$PROPOSAL_RESPONSE" | jq '.'
PROPOSAL_ID=$(echo "$PROPOSAL_RESPONSE" | jq -r '.id')
echo "Proposal ID: $PROPOSAL_ID"
echo ""

echo "10. List Proposals for Multisig"
curl -s -X GET "$BASE_URL/multisigs/$MULTISIG_ID/proposals" \
  -H "Authorization: Bearer $USER1_TOKEN" | jq '.'
echo ""

echo "11. Get Proposal Details"
curl -s -X GET "$BASE_URL/proposals/$PROPOSAL_ID" \
  -H "Authorization: Bearer $USER1_TOKEN" | jq '.'
echo ""

echo "12. Activate Proposal (User 1)"
curl -s -X POST "$BASE_URL/proposals/$PROPOSAL_ID/activate" \
  -H "Authorization: Bearer $USER1_TOKEN" | jq '.'
echo ""

echo "13. Approve Proposal (User 1)"
APPROVE_RESPONSE=$(curl -s -X POST "$BASE_URL/proposals/$PROPOSAL_ID/approve" \
  -H "Authorization: Bearer $USER1_TOKEN")
echo "$APPROVE_RESPONSE" | jq '.'
echo ""

echo "14. Approve Proposal (User 2)"
curl -s -X POST "$BASE_URL/proposals/$PROPOSAL_ID/approve" \
  -H "Authorization: Bearer $USER2_TOKEN" | jq '.'
echo ""

echo "15. Get Proposal Approvals"
curl -s -X GET "$BASE_URL/proposals/$PROPOSAL_ID/approvals" \
  -H "Authorization: Bearer $USER1_TOKEN" | jq '.'
echo ""

echo "16. Execute Proposal (User 1)"
curl -s -X POST "$BASE_URL/proposals/$PROPOSAL_ID/execute" \
  -H "Authorization: Bearer $USER1_TOKEN" | jq '.'
echo ""

echo "17. Create Another Proposal (User 2)"
PROPOSAL2_RESPONSE=$(curl -s -X POST "$BASE_URL/multisigs/$MULTISIG_ID/proposals" \
  -H "Authorization: Bearer $USER2_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Update Threshold",
    "description": "Change threshold to 3"
  }')
echo "$PROPOSAL2_RESPONSE" | jq '.'
PROPOSAL2_ID=$(echo "$PROPOSAL2_RESPONSE" | jq -r '.id')
echo "Proposal 2 ID: $PROPOSAL2_ID"
echo ""

echo "18. Reject Proposal (User 1)"
curl -s -X POST "$BASE_URL/proposals/$PROPOSAL2_ID/reject" \
  -H "Authorization: Bearer $USER1_TOKEN" | jq '.'
echo ""

echo "=== All Tests Completed ==="

