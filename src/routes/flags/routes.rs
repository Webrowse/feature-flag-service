use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use uuid::Uuid;

use crate::routes::{flags::validate_flag_key, middleware_auth::JwtUser};
use crate::state::AppState;
use super::{
    CreateFlagRequest, UpdateFlagRequest, FeatureFlag, FlagResponse,
    validate_rollout_percentage
};

// Handles

/// Create a new feature flag within a project

pub async fn create(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateFlagRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {

    // validate flag key
    validate_flag_key(&payload.key).map_err(|e|(StatusCode::BAD_REQUEST, e))?;
    // Checking if the rollout percentage is provided
    if let Some(percentage) = payload.rollout_percentage {
        validate_rollout_percentage(percentage).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    }


    // checking if project exists and owned by the user
    let project_exist = sqlx::query_scalar::<_, bool>(
       "SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1 AND created_by = $2)"
    )   .bind(project_id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await.map_err(|e| {
            eprintln!("Failed to check project: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
        })?;

        if !project_exist {
            return Err((StatusCode::NOT_FOUND, "Project not found".to_string()));
        }

        //create the flag
        let flag = match sqlx::query_as::<_, FeatureFlag> (
            r#"
            INSERT INTO feature_flags (project_id, name, key, description, enabled, rollout_percentage)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(project_id)
        .bind(&payload.name)
        .bind(&payload.key)
        .bind(&payload.description)
        .bind(payload.enabled.unwrap_or(false))
        .bind(payload.rollout_percentage.unwrap_or(0))
        .fetch_one(&state.db)
        .await
        {
            Ok(flag) => flag,
            Err(e) => {
                if let Some(db_error) = e.as_database_error() {
                    if db_error.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                        return Err((StatusCode::CONFLICT, "Flag key already exists".to_string()));
                    }
                }
                return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)));
            }
        };

    let response = FlagResponse {
        id: flag.id,
        project_id: flag.project_id,
        name: flag.name,
        key: flag.key,
        description: flag.description,
        enabled: flag.enabled,
        rollout_percentage: flag.rollout_percentage,
        created_at: flag.created_at,
        updated_at: flag.updated_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {

    let project_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM project WHERE id = $1 AND created_by = $2)"
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await.map_err(|e| {
        eprintln!("Failed to check project: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    if !project_exists {
        return Err((StatusCode::NOT_FOUND, "Project not found".to_string()));
    }

    let flags = sqlx::query_as::<_, FeatureFlag>(
        r#"
        SELECT * FROM feature_flags
        WHERE project_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(project_id)
    .fetch_all(&state.db)
    .await.map_err(|e| {
        eprintln!("Fialed to fetch flags: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch flags".to_string())
    })?;

    let response: Vec<FlagResponse> = flags.into_iter()
        .map(|f| FlagResponse {
            id: f.id,
            project_id: f.project_id,
            name: f.name,
            key: f.key,
            description: f.description,
            enabled: f.enabled,
            rollout_percentage: f.rollout_percentage,
            created_at: f.created_at,
            updated_at: f.updated_at,
        })
        .collect();

    Ok(Json(response))
}

/// Get a single flag by ID
pub async fn get(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
    Path((project_id, flag_ig)): Path<(Uuid, Uuid)>,

) -> Result<impl IntoResponse, (StatusCode, String)> {
    // fetch flag and verify ownership of project
    let flag = sqlx::query_as::<_, FeatureFlag> (
        r#"
        SELECT f.* FROM feature_flags f
        JOIN projects p ON f.project_id = p.id
        WHERE f.id =$1 AND f.project_id = $2 AND p.created_by = $3
        "#,
    )
    .bind(flag_ig)
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch flag: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch flag".to_string())
    })?;

    match flag {
        Some(f) => {
            let response = FlagResponse {
                id: f.id,
                project_id: f.project_id,
                name: f.name,
                key: f.key,
                description: f.description,
                enabled: f.enabled,
                rollout_percentage: f.rollout_percentage,
                created_at: f. created_at,
                updated_at: f. updated_at,
            };
            Ok(Json(response))
        }
        None => Err((StatusCode::NOT_FOUND, "Flag not found".to_string())),
    }
}

pub async fn update(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
    Path((project_id, flag_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateFlagRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    //validate rollout percentage
    if let Some(percentage) = payload.rollout_percentage {
        validate_rollout_percentage(percentage)
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    }

    // Check if flag exists and user owns the project
    let exists = sqlx::query_scalar::<_,bool> (
        r#"
        SELECT EXISTS(
        SELECT 1 FROM feature_flags f
        JOIN projects p ON f.project_id = p.id
        WHERE f.id = $1 AND f.project_id = $2 AND p.created_by =$3
    )
        "#
    )
    .bind(flag_id)
    .bind(project_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("FAILED to check flag: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    if !exists {
        return Err((StatusCode::NOT_FOUND, "Flag not found".to_string()));
    }

    // dynamic update library
    let mut query = String::from("UPDATE feature_flag SET updated_at = NOW()");
    let mut bind_count = 1;

    if payload.name.is_some() {
        query.push_str(&format!(", name = ${}", bind_count));
        bind_count += 1;
    }

    if payload.description.is_some() {
        query.push_str(&format!(", description = ${}", bind_count));
        bind_count += 1;
    }

    if payload.enabled.is_some() {
        query.push_str(&format!(", enabled = ${}", bind_count));
        bind_count += 1;
    }

    if payload.rollout_percentage.is_some() {
        query.push_str(&format!(", rollout_percentage = ${}", bind_count));
        bind_count += 1;
    }

    query.push_str(&format!(" WHERE id = ${} RETURNING *", bind_count));

    let mut query_builder = sqlx::query_as::<_, FeatureFlag>(&query);

    if let Some(name) = payload.name {
        query_builder = query_builder.bind(name);
    }

    if let Some(description) = payload.description {
        query_builder = query_builder.bind(description);
    }

    if let Some(enabled) = payload.enabled {
        query_builder = query_builder.bind(enabled);
    }

    if let Some(rollout_percentage) = payload.rollout_percentage {
        query_builder = query_builder.bind(rollout_percentage);
    }

    let flag = query_builder
        .bind(flag_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Failed to update flag: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update flag".to_string())
        })?;

    let response = FlagResponse {
        id: flag.id,
        project_id: flag.project_id,
        name: flag.name,
        key: flag.key,
        description: flag.description,
        enabled: flag.enabled,
        rollout_percentage: flag.rollout_percentage,
        created_at: flag.created_at,
        updated_at: flag.updated_at,
    };
    
    Ok(Json(response))
}

// Delete a feature flag
pub async fn delete (
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
    Path((project_id, flag_id)): Path<(Uuid, Uuid)>,

) -> Result<impl IntoResponse, (StatusCode, String)> {
    let result = sqlx::query(
        r#"
        DELETE FROM feature_flags
        WHERE id = $1 AND project_id = $2
        AND EXISTS(SELECT 1 FROM projects WHERE id = $2 AND created_by = $3)
        "#,
    )
    .bind(flag_id)
    .bind(project_id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e|{
        eprintln!("Failed to delete flag: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete flag".to_string())
    })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Flag not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// Toggle a flag's enabled state 
pub async fn toggle(
    State(state): State<AppState>,
    JwtUser(user_id): JwtUser,
    Path((project_id, flag_id)): Path<(Uuid, Uuid)>,

) -> Result<impl IntoResponse, (StatusCode, String)> {
    let flag = sqlx::query_as::<_,FeatureFlag>(
        r#"
        UPDATE feature_flags f
        SET enabled = NOT enabled, updated_at = NOW()
        FROM projects p
        WHERE f.id = $1 AND f.project_id = $2 AND p.id = $2 AND p.created_by = $3
        RETURNING f.*
        "#,
    )
    .bind(flag_id)
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e|{
        eprintln!("Failed to toggle flag: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to toggle flag".to_string())
    })?;

    match flag {
        Some(f) => {
            let response = FlagResponse {
                id: f.id,
                project_id: f.project_id,
                name: f.name,
                key: f.key,
                description: f.description,
                enabled: f.enabled,
                rollout_percentage:f.rollout_percentage,
                created_at: f.created_at,
                updated_at: f.updated_at,

            };
            Ok(Json(response))
        },
        None => Err((StatusCode::NOT_FOUND, "Flag not found".to_string())),
    }
}