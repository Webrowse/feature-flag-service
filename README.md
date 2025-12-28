# Feature Flag Service

A self-hosted feature flag service built with Rust (Axum) and React.

## Quick Start

### Prerequisites

- Docker (for PostgreSQL)
- Rust 1.70+
- Node.js 18+

### 1. Start the Database

```bash
docker run -d \
  --name feature-flag-db \
  -e POSTGRES_USER=admin \
  -e POSTGRES_PASSWORD=admin \
  -e POSTGRES_DB=axum_starter \
  -p 5432:5432 \
  postgres:16
```

### 2. Run the API Server

```bash
cd feature-flag-service

# Create .env file
echo 'DATABASE_URL=postgres://admin:admin@localhost:5432/axum_starter' > .env
echo 'JWT_SECRET=your-secret-key-min-32-chars-long!' >> .env

# Run migrations
cargo install sqlx-cli
sqlx migrate run

# Start the server
cargo run
```

The API runs at `http://localhost:3000`

### 3. Run the Admin UI

```bash
cd admin-ui
npm install
VITE_API_URL=http://localhost:3000 npm run dev
```

The Admin UI runs at `http://localhost:5173`

---

## Core Concepts

### Hierarchy

```
Project
└── Environment (production, staging, etc.)
    └── Feature Flag
        └── Targeting Rules
```

- **Project**: Container for your application. Has a unique SDK key.
- **Environment**: Deployment target (production, staging). Auto-created when you create a project.
- **Feature Flag**: A toggleable feature with rollout controls.
- **Rules**: Target specific users by ID, email, or email domain.

---

## Admin UI Usage

### 1. Register/Login

Navigate to `http://localhost:5173` and create an account.

### 2. Create a Project

Click **+ New Project**, enter a name. Two environments (`production` and `staging`) are created automatically.

### 3. Manage Flags

1. Click **Environments** on your project
2. Select an environment (e.g., `production`)
3. Click **+ New Flag**
4. Configure the flag:
   - **Toggle**: Enable/disable globally
   - **Rollout %**: Percentage of users who see the feature
   - **Rules**: Target specific users

---

## SDK Integration

### Authentication

All SDK requests require the `X-SDK-Key` header. Find your SDK key in the Admin UI on the project card.

### Evaluate Flags

**Endpoint**: `POST /sdk/v1/evaluate`

**Request**:
```bash
curl -X POST http://localhost:3000/sdk/v1/evaluate \
  -H "Content-Type: application/json" \
  -H "X-SDK-Key: sdk_your_key_here" \
  -d '{
    "environment": "production",
    "context": {
      "user_id": "user-123",
      "user_email": "user@example.com",
      "custom_attributes": {}
    }
  }'
```

**Response**:
```json
{
  "flags": {
    "dark_mode": {
      "enabled": true,
      "reason": "Matched user_id rule: user-123"
    },
    "new_checkout": {
      "enabled": false,
      "reason": "User not in 25% rollout"
    }
  }
}
```

### JavaScript/TypeScript SDK Example

```typescript
const SDK_KEY = 'sdk_your_key_here';
const API_URL = 'http://localhost:3000';

interface UserContext {
  user_id?: string;
  user_email?: string;
  custom_attributes?: Record<string, string>;
}

interface FlagState {
  enabled: boolean;
  reason: string;
}

async function evaluateFlags(
  environment: string,
  context: UserContext
): Promise<Record<string, FlagState>> {
  const response = await fetch(`${API_URL}/sdk/v1/evaluate`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-SDK-Key': SDK_KEY,
    },
    body: JSON.stringify({ environment, context }),
  });

  if (!response.ok) {
    throw new Error(`Failed to evaluate flags: ${response.status}`);
  }

  const data = await response.json();
  return data.flags;
}

// Usage
const flags = await evaluateFlags('production', {
  user_id: 'user-123',
  user_email: 'user@company.com',
});

if (flags.dark_mode?.enabled) {
  enableDarkMode();
}
```

### React Hook Example

```typescript
import { useState, useEffect } from 'react';

const SDK_KEY = 'sdk_your_key_here';
const API_URL = 'http://localhost:3000';

export function useFeatureFlags(userId: string, userEmail?: string) {
  const [flags, setFlags] = useState<Record<string, boolean>>({});
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch(`${API_URL}/sdk/v1/evaluate`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-SDK-Key': SDK_KEY,
      },
      body: JSON.stringify({
        environment: 'production',
        context: { user_id: userId, user_email: userEmail },
      }),
    })
      .then((res) => res.json())
      .then((data) => {
        const simplified: Record<string, boolean> = {};
        for (const [key, value] of Object.entries(data.flags)) {
          simplified[key] = (value as { enabled: boolean }).enabled;
        }
        setFlags(simplified);
      })
      .finally(() => setLoading(false));
  }, [userId, userEmail]);

  return { flags, loading, isEnabled: (key: string) => flags[key] ?? false };
}

// Usage in component
function App() {
  const { isEnabled, loading } = useFeatureFlags('user-123');

  if (loading) return <div>Loading...</div>;

  return (
    <div>
      {isEnabled('new_feature') && <NewFeature />}
    </div>
  );
}
```

---

## API Reference

### Authentication Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/auth/register` | Register a new user |
| POST | `/auth/login` | Login and get JWT token |

### Project Endpoints (requires JWT)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/projects` | List all projects |
| POST | `/api/projects` | Create a project |
| GET | `/api/projects/:id` | Get a project |
| PUT | `/api/projects/:id` | Update a project |
| DELETE | `/api/projects/:id` | Delete a project |
| POST | `/api/projects/:id/regenerate-key` | Regenerate SDK key |

### Environment Endpoints (requires JWT)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/projects/:projectId/environments` | List environments |
| POST | `/api/projects/:projectId/environments` | Create environment |
| GET | `/api/projects/:projectId/environments/:id` | Get environment |
| PUT | `/api/projects/:projectId/environments/:id` | Update environment |
| DELETE | `/api/projects/:projectId/environments/:id` | Delete environment |

### Flag Endpoints (requires JWT)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/projects/:p/environments/:e/flags` | List flags |
| POST | `/api/projects/:p/environments/:e/flags` | Create flag |
| GET | `/api/projects/:p/environments/:e/flags/:id` | Get flag |
| PUT | `/api/projects/:p/environments/:e/flags/:id` | Update flag |
| DELETE | `/api/projects/:p/environments/:e/flags/:id` | Delete flag |
| POST | `/api/projects/:p/environments/:e/flags/:id/toggle` | Toggle flag |

### Rule Endpoints (requires JWT)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `.../flags/:flagId/rules` | List rules |
| POST | `.../flags/:flagId/rules` | Create rule |
| GET | `.../flags/:flagId/rules/:id` | Get rule |
| PUT | `.../flags/:flagId/rules/:id` | Update rule |
| DELETE | `.../flags/:flagId/rules/:id` | Delete rule |

### SDK Endpoint (requires X-SDK-Key)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/sdk/v1/evaluate` | Evaluate all flags for a user |

---

## Flag Evaluation Logic

Flags are evaluated in this order:

1. **Global toggle**: If flag is disabled, return `false`
2. **Targeting rules** (by priority, highest first):
   - `user_id`: Exact match on user ID
   - `user_email`: Exact match on email
   - `email_domain`: Match email domain (e.g., `@company.com`)
3. **Percentage rollout**: Consistent hashing ensures same user always gets same result
4. **Default**: If enabled with no rules/rollout, return `true`

---

## Targeting Rules

### Rule Types

| Type | Description | Example Value |
|------|-------------|---------------|
| `user_id` | Match specific user ID | `user-123` |
| `user_email` | Match specific email | `alice@example.com` |
| `email_domain` | Match email domain | `@company.com` |

### Priority

Rules with higher priority are evaluated first. If a rule matches, evaluation stops and the flag is enabled for that user.

---

## Percentage Rollout

The rollout percentage uses consistent hashing:

- Same user + same flag = same result (deterministic)
- 25% rollout means ~25% of users see the feature
- Based on hash of `flag_key:user_identifier`

Set rollout to:
- `0`: Only targeting rules apply
- `100`: Everyone (who passes global toggle)
- `1-99`: Gradual rollout

---

## Best Practices

### 1. Use Environments

- `production`: Real users
- `staging`: QA testing
- Create additional environments as needed (e.g., `development`)

### 2. Gradual Rollouts

Start with a small percentage and increase:
```
Day 1: 5%
Day 2: 25%
Day 3: 50%
Day 4: 100%
```

### 3. Targeting for Testing

Add rules to enable features for your team first:
```
Rule: email_domain = @yourcompany.com
```

### 4. Clean Up Old Flags

Delete flags that are 100% rolled out and no longer needed.

---

## Environment Variables

### API Server

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://user:pass@localhost:5432/db` |
| `JWT_SECRET` | Secret for JWT signing (min 32 chars) | `your-super-secret-key-here-32chars!` |
| `PORT` | Server port (optional, default 3000) | `3000` |

### Admin UI

| Variable | Description | Example |
|----------|-------------|---------|
| `VITE_API_URL` | API server URL | `http://localhost:3000` |
