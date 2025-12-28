use axum::{
    extract::{FromRequestParts, Request},
    http::request::Parts,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;
use uuid::Uuid;

/// Extractor for SDK authentication, returns the project_id
pub struct SdkProject(pub Uuid);

impl<S> FromRequestParts<S> for SdkProject
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Uuid>()
            .copied()
            .map(SdkProject)
            .ok_or((StatusCode::UNAUTHORIZED, "missing project"))
    }
}

/// Middleware to validate SDK key and inject project_id
pub async fn require_sdk_key(
    mut req: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Get SDK key from X-SDK-Key header
    let sdk_key = req
        .headers()
        .get("x-sdk-key")
        .and_then(|v| v.to_str().ok());

    let sdk_key = match sdk_key {
        Some(key) => key,
        None => {
            return Err((StatusCode::UNAUTHORIZED, "Missing X-SDK-Key header"));
        }
    };

    // Get database pool from extensions
    let pool = req
        .extensions()
        .get::<PgPool>()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Database pool not found"))?;

    // Verify SDK key and get project_id
    let project = sqlx::query!(
        r#"
        SELECT id FROM projects WHERE sdk_key = $1
        "#,
        sdk_key
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        eprintln!("Database error validating SDK key: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
    })?;

    match project {
        Some(p) => {
            // Insert project_id into request extensions
            req.extensions_mut().insert(p.id);
            Ok(next.run(req).await)
        }
        None => Err((StatusCode::UNAUTHORIZED, "Invalid SDK key")),
    }
}