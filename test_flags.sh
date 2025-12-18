#!/bin/bash
# Test script for Feature Flags API

BASE_URL="http://127.0.0.1:3000"

echo "=== Feature Flag Service - Flags API Tests ==="
echo ""

# Step 1: Register and login
echo "1. Registering new user..."
REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "flags_test@example.com",
    "password": "password123"
  }')
echo "Response: $REGISTER_RESPONSE"
echo ""

echo "2. Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "flags_test@example.com",
    "password": "password123"
  }')
echo "Response: $LOGIN_RESPONSE"

TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"token":"[^"]*' | cut -d'"' -f4)
echo "Token: $TOKEN"
echo ""

# Step 2: Create a project first (flags need a project)
echo "3. Creating a project for flags..."
CREATE_PROJECT=$(curl -s -X POST "$BASE_URL/api/projects" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Mobile App",
    "description": "Test project for flags"
  }')
echo "Response: $CREATE_PROJECT"

PROJECT_ID=$(echo $CREATE_PROJECT | grep -o '"id":"[^"]*' | cut -d'"' -f4)
echo "Project ID: $PROJECT_ID"
echo ""

# Step 3: Create feature flags
echo "4. Creating first feature flag (new_checkout)..."
CREATE_FLAG1=$(curl -s -X POST "$BASE_URL/api/projects/$PROJECT_ID/flags" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "New Checkout Flow",
    "key": "new_checkout",
    "description": "Redesigned checkout experience",
    "enabled": true,
    "rollout_percentage": 50
  }')
echo "Response: $CREATE_FLAG1"

FLAG1_ID=$(echo $CREATE_FLAG1 | grep -o '"id":"[^"]*' | cut -d'"' -f4)
echo "Flag 1 ID: $FLAG1_ID"
echo ""

echo "5. Creating second feature flag (dark_mode)..."
CREATE_FLAG2=$(curl -s -X POST "$BASE_URL/api/projects/$PROJECT_ID/flags" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Dark Mode",
    "key": "dark_mode",
    "description": "Dark theme for the app",
    "enabled": false,
    "rollout_percentage": 0
  }')
echo "Response: $CREATE_FLAG2"

FLAG2_ID=$(echo $CREATE_FLAG2 | grep -o '"id":"[^"]*' | cut -d'"' -f4)
echo "Flag 2 ID: $FLAG2_ID"
echo ""

echo "6. Creating third feature flag (beta_features)..."
CREATE_FLAG3=$(curl -s -X POST "$BASE_URL/api/projects/$PROJECT_ID/flags" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Beta Features",
    "key": "beta_features",
    "description": "Access to beta features"
  }')
echo "Response: $CREATE_FLAG3"
echo ""

# Step 4: Test duplicate key (should fail)
echo "7. Testing duplicate key (should fail with 409)..."
curl -s -w "\nHTTP Status: %{http_code}" -X POST "$BASE_URL/api/projects/$PROJECT_ID/flags" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Duplicate Checkout",
    "key": "new_checkout",
    "description": "This should fail"
  }'
echo ""
echo ""

# Step 5: Test invalid key format (should fail)
echo "8. Testing invalid key format (should fail with 400)..."
curl -s -w "\nHTTP Status: %{http_code}" -X POST "$BASE_URL/api/projects/$PROJECT_ID/flags" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Invalid Key",
    "key": "Invalid-Key-With-CAPS",
    "description": "Should fail validation"
  }'
echo ""
echo ""

# Step 6: List all flags
echo "9. Listing all flags for project..."
curl -s -X GET "$BASE_URL/api/projects/$PROJECT_ID/flags" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# Step 7: Get single flag
echo "10. Getting single flag details..."
curl -s -X GET "$BASE_URL/api/projects/$PROJECT_ID/flags/$FLAG1_ID" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# Step 8: Update flag
echo "11. Updating flag (changing rollout percentage)..."
curl -s -X PUT "$BASE_URL/api/projects/$PROJECT_ID/flags/$FLAG1_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "New Checkout Flow (Updated)",
    "description": "Updated checkout with improved UX",
    "rollout_percentage": 75
  }'
echo ""
echo ""

# Step 9: Toggle flag
echo "12. Toggling dark_mode flag (currently disabled, should become enabled)..."
TOGGLE_RESPONSE=$(curl -s -X POST "$BASE_URL/api/projects/$PROJECT_ID/flags/$FLAG2_ID/toggle" \
  -H "Authorization: Bearer $TOKEN")
echo "Response: $TOGGLE_RESPONSE"
echo ""

echo "13. Toggling dark_mode again (should become disabled)..."
curl -s -X POST "$BASE_URL/api/projects/$PROJECT_ID/flags/$FLAG2_ID/toggle" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# Step 10: Test invalid rollout percentage
echo "14. Testing invalid rollout percentage (should fail with 400)..."
curl -s -w "\nHTTP Status: %{http_code}" -X PUT "$BASE_URL/api/projects/$PROJECT_ID/flags/$FLAG1_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "rollout_percentage": 150
  }'
echo ""
echo ""

# Step 11: List flags again to see changes
echo "15. Listing flags after updates..."
curl -s -X GET "$BASE_URL/api/projects/$PROJECT_ID/flags" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# Step 12: Delete a flag
echo "16. Deleting beta_features flag..."
DELETE_RESPONSE=$(curl -s -w "\nHTTP Status: %{http_code}" -X DELETE "$BASE_URL/api/projects/$PROJECT_ID/flags/$FLAG2_ID" \
  -H "Authorization: Bearer $TOKEN")
echo "$DELETE_RESPONSE"
echo ""

# Step 13: Try to access deleted flag (should 404)
echo "17. Trying to access deleted flag (should get 404)..."
curl -s -w "\nHTTP Status: %{http_code}" -X GET "$BASE_URL/api/projects/$PROJECT_ID/flags/$FLAG2_ID" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# Step 14: Final list
echo "18. Final flag list (should show 2 flags)..."
curl -s -X GET "$BASE_URL/api/projects/$PROJECT_ID/flags" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

# Step 15: Test accessing flags from wrong project (should 404)
echo "19. Testing cross-project access (should fail)..."
# Create another project
CREATE_PROJECT2=$(curl -s -X POST "$BASE_URL/api/projects" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Another Project"
  }')
PROJECT_ID2=$(echo $CREATE_PROJECT2 | grep -o '"id":"[^"]*' | cut -d'"' -f4)

curl -s -w "\nHTTP Status: %{http_code}" -X GET "$BASE_URL/api/projects/$PROJECT_ID2/flags/$FLAG1_ID" \
  -H "Authorization: Bearer $TOKEN"
echo ""
echo ""

echo "=== All tests completed! ==="
echo ""
echo "Summary:"
echo "- Created 3 feature flags"
echo "- Tested duplicate key validation"
echo "- Tested invalid key format validation"
echo "- Updated flag properties"
echo "- Toggled flag enabled state"
echo "- Tested invalid rollout percentage"
echo "- Deleted 1 flag"
echo "- Tested cross-project access control"
echo "- Final count: 2 flags remaining"