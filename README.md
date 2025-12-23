# Feature Flag Service

> A production-ready feature flag management service built with Rust, Axum, and PostgreSQL

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

**[📖 Complete API Documentation →](./API.md)** | **[🧪 Postman Collection →](./postman_collection.json)**

## What Is This?

A complete, self-hosted feature flag service that allows you to control feature rollouts, perform A/B testing, and target specific users with feature flags. This service provides both a management API for developers and an SDK endpoint for client applications to evaluate flags in real-time.

**Key Features:**

- **Feature Flag Management**: Create, update, and toggle feature flags across multiple projects
- **Sophisticated Targeting Rules**: Target users by ID, email, or email domain
- **Percentage-Based Rollouts**: Gradually roll out features with consistent hashing
- **Multi-Project Support**: Manage flags for multiple applications from a single service
- **SDK Integration**: Public SDK endpoint with API key authentication
- **Analytics Ready**: Tracks all flag evaluations for dashboards and reporting
- **Secure by Default**: JWT authentication, Argon2 password hashing, user-scoped data access
- **Type-Safe**: Rust with compile-time verified SQL queries

## Architecture Overview

This service provides two main interfaces:

1. **Management API** (`/api/*`): For developers to create and manage feature flags, targeting rules, and projects
2. **SDK API** (`/sdk/*`): For client applications to evaluate flags for end users

```
┌─────────────────────────────────────────────────────────┐
│                  Feature Flag Service                   │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Management API          SDK API                        │
│  (JWT Auth)              (SDK Key Auth)                 │
│  ├── Projects            ├── Evaluate Flags             │
│  ├── Flags               └── (Public Endpoint)          │
│  ├── Rules                                              │
│  └── Users                                              │
│                                                         │
│  ┌───────────────────────────────────────────┐          │
│  │      Evaluation Engine                    │          │
│  │  • Rule Matching (priority-based)         │          │
│  │  • Percentage Rollout (consistent hash)   │          │
│  │  • Evaluation Logging                     │          │
│  └───────────────────────────────────────────┘          │
│                                                         │
│  ┌───────────────────────────────────────────┐          │
│  │         PostgreSQL Database               │          │
│  │  • Users & Projects                       │          │
│  │  • Feature Flags                          │          │
│  │  • Targeting Rules                        │          │
│  │  • Evaluation History                     │          │
│  └───────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- **Rust** 1.75+ ([Install Rust](https://www.rust-lang.org/tools/install))
- **Docker** ([Install Docker](https://docs.docker.com/get-docker/))
- **PostgreSQL 16+** (provided via Docker Compose)

### Get Started in 5 Steps

```bash
# 1. Clone the repository
git clone git@github.com:Webrowse/feature-flag-service.git
cd feature-flag-service

# 2. Setup environment variables
cp .env.example .env
# IMPORTANT: Edit .env and change JWT_SECRET to a secure random string!
# Generate secure secret with: openssl rand -base64 32

# 3. Start PostgreSQL
docker-compose up -d

# 4. Run database migrations
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run

# 5. Run the service
cargo run

# Server running at http://127.0.0.1:3000
```

### Quick Test

```bash
# 1. Register a user
curl -X POST http://127.0.0.1:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"developer@example.com","password":"secure123"}'

# 2. Login and get JWT token
TOKEN=$(curl -X POST http://127.0.0.1:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"developer@example.com","password":"secure123"}' \
  | jq -r '.token')

# 3. Create a project
PROJECT=$(curl -X POST http://127.0.0.1:3000/api/projects \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"My App","description":"Production app"}' \
  | jq -r '.id')

echo "Project created: $PROJECT"

# 4. Create a feature flag
curl -X POST http://127.0.0.1:3000/api/projects/$PROJECT/flags \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name":"Dark Mode",
    "key":"dark_mode",
    "description":"Enable dark mode UI",
    "enabled":true,
    "rollout_percentage":50
  }'
```

### Using Postman

For a better testing experience, import the provided Postman collection:

1. **Import Collection**: Import [postman_collection.json](./postman_collection.json) into Postman
2. **Auto-save Variables**: The collection automatically saves JWT tokens, project IDs, flag IDs, and SDK keys
3. **Quick Start**:
   - Run "Register User" → "Login" (saves JWT automatically)
   - Run "Create Project" (saves project_id and sdk_key)
   - Run "Create Flag" (saves flag_id)
   - All subsequent requests will use these saved variables

See [API.md](./API.md) for complete API documentation.

## Core Concepts

### 1. Projects

Projects represent your applications or services. Each project has:
- A unique **SDK key** for client authentication
- Multiple feature flags
- User ownership (only you can access your projects)

```bash
# Create a project
POST /api/projects
{
  "name": "Mobile App",
  "description": "iOS and Android app"
}
```

### 2. Feature Flags

Feature flags control whether a feature is enabled for users. Each flag has:
- **name**: Human-readable name (e.g., "Dark Mode")
- **key**: Unique identifier (e.g., `dark_mode`) - lowercase alphanumeric, `_`, `-` only
- **enabled**: Global on/off switch
- **rollout_percentage**: 0-100% gradual rollout using consistent hashing

```bash
# Create a feature flag
POST /api/projects/{project_id}/flags
{
  "name": "New Checkout Flow",
  "key": "new_checkout",
  "description": "Redesigned checkout experience",
  "enabled": true,
  "rollout_percentage": 25
}
```

### 3. Targeting Rules

Rules allow you to target specific users before applying percentage rollouts. Rules are evaluated in **priority order** (higher priority first).

**Rule Types:**

- **`user_id`**: Match specific user identifiers
  ```json
  {"rule_type": "user_id", "rule_value": "user_12345", "priority": 100}
  ```

- **`user_email`**: Match specific email addresses
  ```json
  {"rule_type": "user_email", "rule_value": "beta@example.com", "priority": 90}
  ```

- **`email_domain`**: Match email domains (must start with `@`)
  ```json
  {"rule_type": "email_domain", "rule_value": "@company.com", "priority": 80}
  ```

```bash
# Create a targeting rule
POST /api/projects/{project_id}/flags/{flag_id}/rules
{
  "rule_type": "email_domain",
  "rule_value": "@yourcompany.com",
  "priority": 100,
  "enabled": true
}
```

### 4. Flag Evaluation

The evaluation algorithm works as follows:

1. **Check if flag is enabled**: If `enabled = false`, return `false` immediately
2. **Evaluate targeting rules**: Check rules in priority order (highest first)
   - If a rule matches, return `true`
   - Only evaluate enabled rules
3. **Apply percentage rollout**: Use consistent hashing on user identifier
   - Hash the combination of flag key + user identifier
   - Return `true` if hash falls within rollout percentage
4. **Return result with reason**: Include explanation (e.g., "rule_match", "rollout", "disabled")

**Example Evaluation:**

```bash
# Evaluate all flags for a user
POST /sdk/v1/evaluate
Headers: X-SDK-Key: sdk_your_key_here
Body:
{
  "user_id": "user_12345",
  "user_email": "alice@example.com",
  "custom_attributes": {}
}

# Response:
{
  "dark_mode": {
    "enabled": true,
    "reason": "rollout"
  },
  "new_checkout": {
    "enabled": true,
    "reason": "rule_match"
  },
  "premium_features": {
    "enabled": false,
    "reason": "disabled"
  }
}
```

## Project Structure

```
feature-flag-service/
├── migrations/                    # Database schema migrations
│   ├── 20251130132949_create_users.sql
│   ├── 20251130132950_create_tasks.sql
│   └── 20251130132951_feature_flag.sql
│
├── src/
│   ├── main.rs                    # Application entry point
│   ├── config.rs                  # Environment configuration
│   ├── state.rs                   # Shared AppState (DB pool)
│   │
│   ├── evaluation/                # Flag evaluation engine
│   │   └── mod.rs                 # Core evaluation logic + tests
│   │
│   └── routes/                    # API route handlers
│       ├── mod.rs                 # Route registration
│       ├── health.rs              # Health check
│       ├── auth.rs                # Registration & login
│       ├── middleware_auth.rs     # JWT middleware
│       ├── sdk_auth.rs            # SDK key middleware
│       │
│       ├── projects/              # Project management
│       │   ├── mod.rs             # Models & validation
│       │   └── routes.rs          # CRUD handlers
│       │
│       ├── flags/                 # Feature flag management
│       │   ├── mod.rs             # Models & validation
│       │   └── routes.rs          # CRUD + toggle handlers
│       │
│       ├── rules/                 # Targeting rules
│       │   ├── mod.rs             # Models & validation
│       │   └── routes.rs          # CRUD handlers
│       │
│       ├── sdk/                   # SDK endpoints
│       │   ├── mod.rs             # Response models
│       │   └── routes.rs          # Flag evaluation
│       │
│       └── tasks/                 # Legacy task management
│           └── ...
│
├── test_*.sh                      # Integration test scripts
├── API.md                         # Complete API documentation
├── CONTRIBUTING.md                # Contribution guidelines
├── Cargo.toml                     # Rust dependencies
├── docker-compose.yml             # PostgreSQL setup
└── .env                           # Configuration (gitignored)
```

## API Overview

### Authentication Endpoints (Public)

| Method | Endpoint          | Description        |
|--------|-------------------|--------------------|
| POST   | `/auth/register`  | Register new user  |
| POST   | `/auth/login`     | Login and get JWT  |

### Management API (JWT Required)

**Projects:**
| Method | Endpoint                              | Description              |
|--------|---------------------------------------|--------------------------|
| POST   | `/api/projects`                       | Create project           |
| GET    | `/api/projects`                       | List your projects       |
| GET    | `/api/projects/{id}`                  | Get project details      |
| PUT    | `/api/projects/{id}`                  | Update project           |
| DELETE | `/api/projects/{id}`                  | Delete project           |
| POST   | `/api/projects/{id}/regenerate-key`   | Regenerate SDK key       |

**Feature Flags:**
| Method | Endpoint                                      | Description        |
|--------|-----------------------------------------------|--------------------|
| POST   | `/api/projects/{pid}/flags`                   | Create flag        |
| GET    | `/api/projects/{pid}/flags`                   | List flags         |
| GET    | `/api/projects/{pid}/flags/{fid}`             | Get flag           |
| PUT    | `/api/projects/{pid}/flags/{fid}`             | Update flag        |
| DELETE | `/api/projects/{pid}/flags/{fid}`             | Delete flag        |
| POST   | `/api/projects/{pid}/flags/{fid}/toggle`      | Toggle enabled     |

**Targeting Rules:**
| Method | Endpoint                                         | Description     |
|--------|--------------------------------------------------|-----------------|
| POST   | `/api/projects/{pid}/flags/{fid}/rules`          | Create rule     |
| GET    | `/api/projects/{pid}/flags/{fid}/rules`          | List rules      |
| GET    | `/api/projects/{pid}/flags/{fid}/rules/{rid}`    | Get rule        |
| PUT    | `/api/projects/{pid}/flags/{fid}/rules/{rid}`    | Update rule     |
| DELETE | `/api/projects/{pid}/flags/{fid}/rules/{rid}`    | Delete rule     |

### SDK API (SDK Key Required)

| Method | Endpoint             | Description                    |
|--------|----------------------|--------------------------------|
| POST   | `/sdk/v1/evaluate`   | Evaluate all flags for user    |

**Headers:** `X-SDK-Key: sdk_your_key_here`

See [API.md](./API.md) for detailed documentation with examples.

## Database Schema

### Core Tables

**users** - User authentication
- `id` (UUID, PK)
- `email` (TEXT, unique)
- `password_hash` (TEXT)
- `created_at` (TIMESTAMP)

**projects** - Feature flag projects
- `id` (UUID, PK)
- `name` (TEXT)
- `description` (TEXT, nullable)
- `sdk_key` (TEXT, globally unique, indexed)
- `created_by` (UUID, FK → users)
- `created_at`, `updated_at` (TIMESTAMPTZ)

**feature_flags** - Feature flags
- `id` (UUID, PK)
- `project_id` (UUID, FK → projects, CASCADE)
- `name` (TEXT)
- `key` (TEXT, unique per project)
- `description` (TEXT, nullable)
- `enabled` (BOOLEAN, default FALSE)
- `rollout_percentage` (INT, 0-100, default 0)
- `created_at`, `updated_at` (TIMESTAMPTZ)

**flag_rules** - Targeting rules
- `id` (UUID, PK)
- `flag_id` (UUID, FK → feature_flags, CASCADE)
- `rule_type` (TEXT: user_id, user_email, email_domain)
- `rule_value` (TEXT)
- `enabled` (BOOLEAN, default TRUE)
- `priority` (INT, default 0)
- `created_at` (TIMESTAMPTZ)

**flag_evaluations** - Evaluation history (analytics)
- `id` (BIGSERIAL, PK)
- `flag_id` (UUID, FK → feature_flags, CASCADE)
- `user_identifier` (TEXT)
- `result` (BOOLEAN)
- `evaluated_at` (TIMESTAMPTZ)

### Indexes for Performance

- `idx_projects_created_by` - Fast user project lookup
- `idx_flags_project` - Fast flag lookup per project
- `idx_flags_project_key` - Unique key constraint per project
- `idx_rules_flag` - Fast rule lookup per flag
- `idx_rules_flag_priority` - Rule ordering for evaluation
- `idx_evaluations_flag_time` - Analytics queries
- `idx_project_sdk_key` - SDK key authentication

## Tech Stack

| Component          | Technology               | Purpose                           |
|--------------------|--------------------------|-----------------------------------|
| Language           | Rust 1.75+               | Performance & safety              |
| Web Framework      | Axum 0.8.7               | Async, type-safe HTTP             |
| Database           | PostgreSQL 16            | Persistent storage                |
| DB Driver          | SQLx 0.8.6               | Compile-time verified queries     |
| Runtime            | Tokio                    | Async executor                    |
| Authentication     | JWT (jsonwebtoken 9.0)   | Token-based auth                  |
| Password Hashing   | Argon2 0.5.3             | Memory-hard hashing               |
| Serialization      | serde 1.0 + serde_json   | JSON API                          |
| CORS               | tower-http               | Cross-origin requests             |

## Security Features

- **JWT Authentication**: 24-hour token validity with secure signing
- **Argon2 Password Hashing**: Memory-hard algorithm with per-password salts
- **SQL Injection Protection**: Compile-time verified queries via SQLx
- **User Scoping**: Users can only access their own projects and flags
- **SDK Key Authentication**: Secure API keys for public SDK endpoints
- **No Plaintext Secrets**: All sensitive data properly hashed/encrypted

## Development

### Running Tests

```bash
# Use Postman collection for interactive testing
# Import postman_collection.json for organized CRUD operations
```

### Code Quality

```bash
# Lint code
cargo clippy

# Format code
cargo fmt

# Check for issues
cargo check
```

### Database Migrations

```bash
# Create new migration
# Example names: create_users, add_projects_table, add_email_index
sqlx migrate add create_feature_flags

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

## Production Deployment

### Environment Variables

**Critical Configuration:**

```env
# Server
PORT=3000

# Database
DATABASE_URL=postgresql://user:password@host:5432/dbname

# Security
JWT_SECRET=your_super_secure_random_secret_at_least_32_characters_long

# Optional
RUST_LOG=info
```

### Build for Production

```bash
# Build optimized release binary
cargo build --release

# Binary location
./target/release/feature-flag-service

# Run
./target/release/feature-flag-service
```

### Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/feature-flag-service /usr/local/bin/
COPY migrations /migrations

ENV PORT=3000
EXPOSE 3000

CMD ["feature-flag-service"]
```

## Use Cases

### Gradual Feature Rollout

Roll out a new feature to 10% of users, then gradually increase:

```bash
# Start with 10%
curl -X POST http://localhost:3000/api/projects/$PROJECT/flags \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"New UI","key":"new_ui","enabled":true,"rollout_percentage":10}'

# Increase to 50%
curl -X PUT http://localhost:3000/api/projects/$PROJECT/flags/$FLAG \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"rollout_percentage":50}'

# Full rollout
curl -X PUT http://localhost:3000/api/projects/$PROJECT/flags/$FLAG \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"rollout_percentage":100}'
```

### Internal Beta Testing

Enable features for your company's email domain:

```bash
# Create rule for company domain
curl -X POST http://localhost:3000/api/projects/$PROJECT/flags/$FLAG/rules \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "rule_type":"email_domain",
    "rule_value":"@yourcompany.com",
    "priority":100,
    "enabled":true
  }'
```

### VIP User Access

Give specific users early access:

```bash
# Target specific user
curl -X POST http://localhost:3000/api/projects/$PROJECT/flags/$FLAG/rules \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "rule_type":"user_email",
    "rule_value":"vip@example.com",
    "priority":100,
    "enabled":true
  }'
```

### Kill Switch

Quickly disable a problematic feature:

```bash
# Disable flag immediately
curl -X POST http://localhost:3000/api/projects/$PROJECT/flags/$FLAG/toggle \
  -H "Authorization: Bearer $TOKEN"
```

## Client Integration

### Example: JavaScript/TypeScript SDK

```typescript
class FeatureFlagClient {
  constructor(private sdkKey: string, private baseUrl: string) {}

  async evaluateFlags(userId: string, userEmail?: string) {
    const response = await fetch(`${this.baseUrl}/sdk/v1/evaluate`, {
      method: 'POST',
      headers: {
        'X-SDK-Key': this.sdkKey,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_id: userId,
        user_email: userEmail,
        custom_attributes: {},
      }),
    });

    return await response.json();
  }

  async isEnabled(flagKey: string, userId: string, userEmail?: string) {
    const flags = await this.evaluateFlags(userId, userEmail);
    return flags[flagKey]?.enabled ?? false;
  }
}

// Usage
const client = new FeatureFlagClient(
  'sdk_your_key_here',
  'https://flags.yourdomain.com'
);

if (await client.isEnabled('dark_mode', 'user_123', 'user@example.com')) {
  // Show dark mode
}
```

## Performance Considerations

- **Consistent Hashing**: Ensures same user always gets same rollout decision
- **Database Indexes**: Optimized for fast flag evaluation queries
- **Connection Pooling**: SQLx connection pool for concurrent requests
- **Compile-Time Queries**: Zero runtime SQL parsing overhead
- **Async I/O**: Non-blocking request handling with Tokio

**Typical Performance:**
- Flag evaluation: < 10ms (including DB query)
- Rule matching: O(n) where n = number of rules per flag
- Scales to millions of evaluations/day on modest hardware

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

MIT License - see LICENSE file for details

## Support

- **Issues**: Report bugs at [GitHub Issues](https://github.com/yourusername/feature-flag-service/issues)
- **Documentation**: See [API.md](./API.md) for complete API reference
- **Tests**: Run integration tests with `./test_*.sh` scripts

## Acknowledgments

Built with these excellent Rust crates:

- [Axum](https://github.com/tokio-rs/axum) - Ergonomic web framework
- [SQLx](https://github.com/launchbadge/sqlx) - Type-safe SQL toolkit
- [Tokio](https://tokio.rs/) - Async runtime
- [Argon2](https://github.com/RustCrypto/password-hashes) - Secure password hashing
- [jsonwebtoken](https://github.com/Keats/jsonwebtoken) - JWT implementation
- [serde](https://serde.rs/) - Serialization framework

---

**Built with Rust** | Production-ready feature flag management
