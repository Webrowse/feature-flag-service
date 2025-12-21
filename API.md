# Feature Flag Service - API Reference

## Overview

This is a Rust-based feature flag management service built with Axum and PostgreSQL. It provides a complete REST API for managing feature flags with sophisticated targeting rules.

**Tech Stack:**
- Axum 0.8.7 (Web Framework)
- PostgreSQL 16 (Database)
- SQLx 0.8.6 (Database Driver)
- JWT Authentication
- Argon2 Password Hashing

**Base URL:** `http://localhost:3000`

## Authentication

All `/api/*` endpoints require JWT authentication via the `Authorization: Bearer {token}` header.

## Endpoints

### Health Check

#### Check Service Health
```
GET /health
Response: "OK"
```

### Authentication (Public)

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

### Current User

#### Get Current User
```
GET /api/me
Response: { "user_id": "uuid" }
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

### Flag Rules (Targeting)

Target specific users or groups with advanced flag rules. Rules are evaluated for each flag to determine if it should be enabled for a specific user.

#### Create Rule
```
POST /api/projects/{project_id}/flags/{flag_id}/rules
Body: {
  "rule_type": "user_email",           // user_id, user_email, or email_domain
  "rule_value": "admin@example.com",   // The value to match
  "enabled": true,                     // optional, default: true
  "priority": 10                       // optional, default: 0, higher = evaluated first
}
Response: {
  "id": "uuid",
  "flag_id": "uuid",
  "rule_type": "user_email",
  "rule_value": "admin@example.com",
  "enabled": true,
  "priority": 10,
  "created_at": "2024-12-14T10:00:00Z"
}
```

**Rule Types:**
- `user_id` - Match specific user identifier
- `user_email` - Match specific email address (must contain @)
- `email_domain` - Match email domain (must start with @, e.g., "@company.com")

**Validation Rules:**
- `rule_value` cannot be empty
- Email domains must start with @
- User emails must contain @
- `priority` determines evaluation order (higher values evaluated first)

#### List Rules
```
GET /api/projects/{project_id}/flags/{flag_id}/rules
Response: [ {...rule}, {...rule} ]
Note: Rules are returned ordered by priority (highest first)
```

#### Get Rule
```
GET /api/projects/{project_id}/flags/{flag_id}/rules/{rule_id}
Response: {...rule}
```

#### Update Rule
```
PUT /api/projects/{project_id}/flags/{flag_id}/rules/{rule_id}
Body: {
  "rule_type": "email_domain",
  "rule_value": "@newcompany.com",
  "enabled": false,
  "priority": 20
}
Note: All fields are optional, only provided fields are updated
Response: {...rule}
```

#### Delete Rule
```
DELETE /api/projects/{project_id}/flags/{flag_id}/rules/{rule_id}
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

## Database Schema

**Tables:**
- `users` - User authentication
- `projects` - Feature flag projects/applications
- `feature_flags` - Feature flags with rollout percentages
- `flag_rules` - Targeting rules for flags
- `flag_evaluations` - Analytics/history (table exists, API pending)
- `tasks` - Legacy task management (from template)

**Key Constraints:**
- Project SDK keys are globally unique
- Flag keys must be unique within a project
- All data is user-scoped (users can only access their own projects)

---

## Security Features

- **JWT Authentication** - 24-hour token validity
- **Argon2 Password Hashing** - Memory-hard algorithm
- **User Ownership Verification** - Users can only access their own projects
- **SQL Injection Protection** - Compile-time verified queries via SQLx
- **CORS Support** - Configurable via Tower middleware

---

## Configuration

**Environment Variables (.env):**
```
PORT=3000
DATABASE_URL=postgres://admin:admin@localhost:5432/axum_starter
JWT_SECRET=thisIsASecretShhhhh
```

**Database:** PostgreSQL 16 via Docker Compose

**Start Service:**
```bash
docker-compose up -d    # Start PostgreSQL
cargo run               # Start API server
```

---

## Next Steps

**Coming Soon:**
- SDK Evaluation Endpoint - Public endpoint for client SDKs to evaluate flags based on user context
- Analytics Dashboard - Utilize `flag_evaluations` table for usage metrics
- Webhooks - Notify external services when flags change
- Audit Logs - Track who changed what and when