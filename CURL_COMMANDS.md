# API Testing with cURL

Base URL: `http://127.0.0.1:8080`

## Authentication Endpoints

### 1. Register User
```bash
curl -X POST http://127.0.0.1:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "password123"
  }'
```

### 2. Login
```bash
curl -X POST http://127.0.0.1:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "password123"
  }'
```

### 3. Get Current User
```bash
curl -X GET http://127.0.0.1:8080/auth/me \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

## Multisig Endpoints

### 4. Create Multisig
```bash
curl -X POST http://127.0.0.1:8080/multisigs \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Team Multisig",
    "description": "Main team wallet",
    "owners": [1, 2, 3],
    "threshold": 2
  }'
```

### 5. List User's Multisigs
```bash
curl -X GET http://127.0.0.1:8080/multisigs \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

### 6. Get Multisig Details
```bash
curl -X GET http://127.0.0.1:8080/multisigs/1 \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

## Proposal Endpoints

### 7. Create Proposal
```bash
curl -X POST http://127.0.0.1:8080/multisigs/1/proposals \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Transfer 100 SOL",
    "description": "Transfer funds to new wallet",
    "transaction_data": "base64_encoded_transaction_data"
  }'
```

### 8. List Proposals for Multisig
```bash
curl -X GET http://127.0.0.1:8080/multisigs/1/proposals \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

### 9. Get Proposal Details
```bash
curl -X GET http://127.0.0.1:8080/proposals/1 \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

### 10. Activate Proposal
```bash
curl -X POST http://127.0.0.1:8080/proposals/1/activate \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

### 11. Approve Proposal
```bash
curl -X POST http://127.0.0.1:8080/proposals/1/approve \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

### 12. Execute Proposal
```bash
curl -X POST http://127.0.0.1:8080/proposals/1/execute \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

### 13. Reject Proposal
```bash
curl -X POST http://127.0.0.1:8080/proposals/1/reject \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

### 14. Get Proposal Approvals
```bash
curl -X GET http://127.0.0.1:8080/proposals/1/approvals \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

## Complete Test Flow Example

```bash
# 1. Register and get token
TOKEN=$(curl -s -X POST http://127.0.0.1:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "test123"}' \
  | jq -r '.token')

# 2. Get user ID
USER_ID=$(curl -s -X GET http://127.0.0.1:8080/auth/me \
  -H "Authorization: Bearer $TOKEN" \
  | jq -r '.id')

# 3. Create multisig
MULTISIG_ID=$(curl -s -X POST http://127.0.0.1:8080/multisigs \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"Test Multisig\", \"owners\": [$USER_ID], \"threshold\": 1}" \
  | jq -r '.id')

# 4. Create proposal
PROPOSAL_ID=$(curl -s -X POST http://127.0.0.1:8080/multisigs/$MULTISIG_ID/proposals \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title": "Test Proposal"}' \
  | jq -r '.id')

# 5. Activate proposal
curl -X POST http://127.0.0.1:8080/proposals/$PROPOSAL_ID/activate \
  -H "Authorization: Bearer $TOKEN"

# 6. Approve proposal
curl -X POST http://127.0.0.1:8080/proposals/$PROPOSAL_ID/approve \
  -H "Authorization: Bearer $TOKEN"

# 7. Execute proposal
curl -X POST http://127.0.0.1:8080/proposals/$PROPOSAL_ID/execute \
  -H "Authorization: Bearer $TOKEN"
```

## Quick Test Script

Run the automated test script:
```bash
./test_endpoints.sh
```

Make sure you have `jq` installed for JSON parsing:
```bash
brew install jq  # macOS
# or
sudo apt-get install jq  # Linux
```

