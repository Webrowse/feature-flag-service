# Axum API Template

> A production-ready REST API template built with Axum, PostgreSQL, and JWT authentication

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

## What Is This?

A complete, working REST API starter template that includes everything you need to build a secure, scalable backend:

- ✅ JWT-based authentication
- ✅ User registration & login
- ✅ Task management (full CRUD)
- ✅ PostgreSQL with migrations
- ✅ Password hashing with Argon2
- ✅ Type-safe database queries
- ✅ Middleware-based auth
- ✅ Docker Compose setup

**Built over 6 days** as a learning journey (see development log below).

## Quick Start

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.75+
- [Docker](https://docs.docker.com/get-docker/) (easiest way)
- OR PostgreSQL 16+ installed locally

### Get Started in 3 Steps
```bash
# 1. Use this template (click "Use this template" on GitHub)
# OR clone it:
git clone https://github.com/yourusername/axum-api-template.git
cd axum-api-template

# 2. Setup environment
cp .env.example .env
# Edit .env and change JWT_SECRET to something secure!

# 3. Start database and run migrations
docker-compose up -d
cargo install sqlx-cli
sqlx migrate run

# 4. Run the server
cargo run

# Server running at http://127.0.0.1:3000
```

## API Endpoints

### Public Endpoints

#### Register User
```bash
curl -X POST http://127.0.0.1:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'
```

#### Login
```bash
curl -X POST http://127.0.0.1:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'

# Returns: {"token":"eyJ0eXAiOiJKV1QiLCJhbGc..."}
```

### Protected Endpoints (Require JWT Token)

#### Create Task
```bash
curl -X POST http://127.0.0.1:3000/api/task \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"title":"Buy groceries"}'
```

#### List Tasks
```bash
curl http://127.0.0.1:3000/api/task \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### Update Task
```bash
curl -X PUT http://127.0.0.1:3000/api/task/TASK_ID \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"done":true}'
```

#### Delete Task
```bash
curl -X DELETE http://127.0.0.1:3000/api/task/TASK_ID \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

##  Project Structure

```md
axum-api-template/
├── migrations/           # Database migrations
│   ├── *_create_users.sql
│   └── *_create_tasks.sql
├── src/
│   ├── config.rs        # Configuration management
│   ├── state.rs         # Shared app state
│   ├── routes/
│   │   ├── mod.rs       # Route definitions
│   │   ├── auth.rs      # Registration & login
│   │   ├── health.rs    # Health check endpoint
│   │   ├── middleware_auth.rs  # JWT validation
│   │   └── tasks/       # Task management module
│   │       ├── dto.rs   # Request/response types
│   │       ├── model.rs # Database models
│   │       ├── queries.rs # Database operations
│   │       └── routes.rs  # Task endpoints
│   └── main.rs          # Application entry point
├── .env                 # Environment template
├── docker-compose.yml   # PostgreSQL setup
└── Cargo.toml
```

## Key Features Explained

### JWT Authentication
- Tokens valid for 24 hours
- Middleware automatically validates tokens
- Custom `JwtUser` extractor for clean handlers

### Database Design
- UUID primary keys for distributed systems
- Foreign key constraints with CASCADE delete
- Timestamptz for timezone awareness
- SQLX compile-time query verification

### Security
- Argon2 password hashing (memory-hard algorithm)
- Random salt per password
- No plaintext passwords stored
- User-scoped data access

### Code Organization
- Modular route structure
- Separation of DTOs, models, and queries
- Custom extractors for reduced boilerplate
- Type-safe throughout

## Development Journey

This project was built incrementally over 6 days as a learning exercise:

- **Day 1**: Basic Axum server with text and JSON routes
- **Day 2**: PostgreSQL connection, SQLX setup, first migration
- **Day 3**: User registration with Argon2 password hashing
- **Day 4**: JWT login flow and authentication middleware
- **Day 5**: Task CRUD operations with database persistence
- **Day 6**: Code refactoring, custom extractors, better patterns

Each day built upon the previous, demonstrating incremental development.

## Customization Guide

### Adding New Protected Routes
```rust
// In src/routes/mod.rs
pub fn routes() -> Router<AppState> {
    Router::new()
        // ... existing routes
        .nest(
            "/api",
            Router::new()
                .route("/your-route", get(your_handler))
                // Automatically protected by JWT middleware
                .layer(middleware::from_fn(middleware_auth::require_auth)),
        )
}
```

### Adding Database Tables
```bash
# Create migration
sqlx migrate add create_your_table

# Edit the migration file in migrations/
# Run migration
sqlx migrate run
```

### Adding New Features

1. Create module in `src/routes/your_feature/`
2. Add `dto.rs`, `model.rs`, `queries.rs`, `routes.rs`
3. Register routes in `src/routes/mod.rs`
4. Follow the existing task module pattern

## Production Deployment

### Environment Variables

 **Critical**: Change these before deploying!
```env
JWT_SECRET=generate_a_strong_random_secret_at_least_32_chars
DATABASE_URL=postgresql://user:password@prod-host:5432/db
PORT=3000
```

### Build for Production
```bash
cargo build --release
./target/release/axum-api-template
```

### Docker Deployment
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates
COPY --from=builder /app/target/release/axum-api-template /usr/local/bin/
CMD ["axum-api-template"]
```

## Testing
```bash
# Run tests
cargo test

# Check code
cargo clippy

# Format code
cargo fmt
```

## Contributing

This is a learning project and template. Feel free to:
- Fork it and make it your own
- Submit PRs for improvements
- Open issues for bugs or suggestions
- Use it in your projects

## License

MIT License - see LICENSE file

## Acknowledgments

Built with these amazing Rust crates:
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [SQLX](https://github.com/launchbadge/sqlx) - SQL toolkit
- [Tokio](https://tokio.rs/) - Async runtime
- [Argon2](https://github.com/RustCrypto/password-hashes) - Password hashing
- [jsonwebtoken](https://github.com/Keats/jsonwebtoken) - JWT handling

## What's Next?

Consider adding:
- [ ] Email verification
- [ ] Password reset flow
- [ ] Rate limiting
- [ ] API documentation (Swagger/OpenAPI)
- [ ] Logging with tracing
- [ ] Tests (unit & integration)
- [ ] CI/CD pipeline
- [ ] Refresh tokens
- [ ] Role-based access control

---

**Star this repo** if it helped you! ⭐

Built by Adarsh | [X (Twitter)](https://x.com/Adarsh_web3)