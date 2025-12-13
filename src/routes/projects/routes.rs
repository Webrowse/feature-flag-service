// src/routes/projects/routes.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::routes::middleware_auth::JwtUser;
use crate::state::AppState;
use super::{CreateProjectRequest, UpdateProjectRequest, Project, ProjectResponse, generate_sdk_key};


// HANDLERS

/// Create a new project
pub async fn create(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,  // ← Tuple struct destructuring
    Json(payload): Json<CreateProjectRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Generate a secure SDK key (this is what client apps will use)
    let sdk_key = generate_sdk_key();

    let project = sqlx::query_as::<_, Project>(
        r#"
        INSERT INTO projects (name, description, sdk_key, created_by)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&sdk_key)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to create project: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    let response = ProjectResponse {
        id: project.id,
        name: project.name,
        description: project.description,
        sdk_key: project.sdk_key,
        created_at: project.created_at,
        updated_at: project.updated_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// List all projects for the authenticated user
pub async fn list(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let projects = sqlx::query_as::<_, Project>(
        r#"
        SELECT * FROM projects
        WHERE created_by = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch projects: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch projects".to_string())
    })?;

    let response: Vec<ProjectResponse> = projects
        .into_iter()
        .map(|p| ProjectResponse {
            id: p.id,
            name: p.name,
            description: p.description,
            sdk_key: p.sdk_key,
            created_at: p.created_at,
            updated_at: p.updated_at,
        })
        .collect();

    Ok(Json(response))
}

/// Get a single project by ID
pub async fn get(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let project = sqlx::query_as::<_, Project>(
        r#"
        SELECT * FROM projects
        WHERE id = $1 AND created_by = $2
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch project: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch project".to_string())
    })?;

    match project {
        Some(p) => {
            let response = ProjectResponse {
                id: p.id,
                name: p.name,
                description: p.description,
                sdk_key: p.sdk_key,
                created_at: p.created_at,
                updated_at: p.updated_at,
            };
            Ok(Json(response))
        }
        None => Err((StatusCode::NOT_FOUND, "Project not found".to_string())),
    }
}

/// Update a project
pub async fn update(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<UpdateProjectRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // First check if project exists and belongs to user
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1 AND created_by = $2)"
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to check project: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    if !exists {
        return Err((StatusCode::NOT_FOUND, "Project not found".to_string()));
    }

    // Build dynamic update query based on what fields are provided
    let mut query = String::from("UPDATE projects SET updated_at = NOW()");
    let mut bind_count = 1;

    if payload.name.is_some() {
        query.push_str(&format!(", name = ${}", bind_count));
        bind_count += 1;
    }
    if payload.description.is_some() {
        query.push_str(&format!(", description = ${}", bind_count));
        bind_count += 1;
    }

    query.push_str(&format!(" WHERE id = ${} AND created_by = ${} RETURNING *", bind_count, bind_count + 1));

    let mut query_builder = sqlx::query_as::<_, Project>(&query);

    if let Some(name) = payload.name {
        query_builder = query_builder.bind(name);
    }
    if let Some(description) = payload.description {
        query_builder = query_builder.bind(description);
    }

    let project = query_builder
        .bind(project_id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Failed to update project: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update project".to_string())
        })?;

    let response = ProjectResponse {
        id: project.id,
        name: project.name,
        description: project.description,
        sdk_key: project.sdk_key,
        created_at: project.created_at,
        updated_at: project.updated_at,
    };

    Ok(Json(response))
}

/// Delete a project (this will cascade delete all flags)
pub async fn delete(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let result = sqlx::query(
        r#"
        DELETE FROM projects
        WHERE id = $1 AND created_by = $2
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to delete project: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete project".to_string())
    })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Project not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Regenerate SDK key for a project (useful if key is compromised)
pub async fn regenerate_key(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let new_sdk_key = generate_sdk_key();

    let project = sqlx::query_as::<_, Project>(
        r#"
        UPDATE projects
        SET sdk_key = $1, updated_at = NOW()
        WHERE id = $2 AND created_by = $3
        RETURNING *
        "#,
    )
    .bind(&new_sdk_key)
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to regenerate SDK key: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to regenerate SDK key".to_string())
    })?;

    match project {
        Some(p) => {
            let response = ProjectResponse {
                id: p.id,
                name: p.name,
                description: p.description,
                sdk_key: p.sdk_key,
                created_at: p.created_at,
                updated_at: p.updated_at,
            };
            Ok(Json(response))
        }
        None => Err((StatusCode::NOT_FOUND, "Project not found".to_string())),
    }
}