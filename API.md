# Feature Flag Service - API Reference

## Authentication
All `/api/*` endpoints require JWT authentication via the `Authorization: Bearer {token}` header.

## Endpoints

### Authentication

#### Register
```
POST /auth/register
Body: { "email": "user@example.com", "password": "password123" }
Response: { "id": "uuid", "email": "user@example.com" }
```

#### Login
```
POST /auth/login
Body: { "email": "user@example.com", "password": "password123" }
Response: { "token": "jwt_token_here" }
```

---

### Projects

#### Create Project
```
POST /api/projects
Body: {
  "name": "My App",
  "description": "Optional description"
}
Response: {
  "id": "uuid",
  "name": "My App",
  "description": "Optional description",
  "sdk_key": "sdk_xxxxx...",
  "created_at": "2024-12-14T10:00:00Z",
  "updated_at": "2024-12-14T10:00:00Z"
}
```

#### List Projects
```
GET /api/projects
Response: [ {...project}, {...project} ]
```

#### Get Project
```
GET /api/projects/{project_id}
Response: {...project}
```

#### Update Project
```
PUT /api/projects/{project_id}
Body: {
  "name": "Updated Name",
  "description": "Updated description"
}
Response: {...project}
```

#### Delete Project
```
DELETE /api/projects/{project_id}
Response: 204 No Content
Note: Cascades to delete all flags
```

#### Regenerate SDK Key
```
POST /api/projects/{project_id}/regenerate-key
Response: {...project with new sdk_key}
```

---

### Feature Flags

#### Create Flag
```
POST /api/projects/{project_id}/flags
Body: {
  "name": "New Checkout",
  "key": "new_checkout",              // lowercase, alphanumeric, _, -
  "description": "Optional",
  "enabled": true,                    // optional, default: false
  "rollout_percentage": 50           // optional, 0-100, default: 0
}
Response: {
  "id": "uuid",
  "project_id": "uuid",
  "name": "New Checkout",
  "key": "new_checkout",
  "description": "Optional",
  "enabled": true,
  "rollout_percentage": 50,
  "created_at": "2024-12-14T10:00:00Z",
  "updated_at": "2024-12-14T10:00:00Z"
}
```

**Validation Rules:**
- `key` must start with a letter
- `key` can only contain lowercase letters, numbers, `_`, and `-`
- `key` must be unique within the project
- `rollout_percentage` must be 0-100

#### List Flags
```
GET /api/projects/{project_id}/flags
Response: [ {...flag}, {...flag} ]
```

#### Get Flag
```
GET /api/projects/{project_id}/flags/{flag_id}
Response: {...flag}
```

#### Update Flag
```
PUT /api/projects/{project_id}/flags/{flag_id}
Body: {
  "name": "Updated Name",
  "description": "Updated description",
  "enabled": false,
  "rollout_percentage": 75
}
Note: All fields are optional, only provided fields are updated
Response: {...flag}
```

#### Toggle Flag
```
POST /api/projects/{project_id}/flags/{flag_id}/toggle
Response: {...flag with flipped enabled state}
```

#### Delete Flag
```
DELETE /api/projects/{project_id}/flags/{flag_id}
Response: 204 No Content
```

---

## Error Responses

All error responses follow this format:
```
Status: 4xx or 5xx
Body: "Error message string"
```

Common status codes:
- `400 Bad Request` - Invalid input (validation failed)
- `401 Unauthorized` - Missing or invalid JWT token
- `404 Not Found` - Resource doesn't exist
- `409 Conflict` - Duplicate key or other constraint violation
- `500 Internal Server Error` - Server-side error

---

## Testing

Run the test scripts to verify everything works:

```bash
# Test projects
chmod +x test_project.sh
./test_project.sh

# Test feature flags
chmod +x test_flags.sh
./test_flags.sh
```

---

## Next Steps: SDK Evaluation Endpoint

Coming soon: Public endpoint for client SDKs to evaluate flags based on user context.