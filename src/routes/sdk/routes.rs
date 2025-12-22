use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::collections::HashMap;

use crate::evaluation::{evaluate_flag, FlagData, RuleData, UserContext};
use crate::routes::sdk_auth::SdkProject;
use crate::state::AppState;
use super::{EvaluateResponse, FlagState};

/// Evaluate all flags for a project based on user context
pub async fn evaluate(
    State(state): State<AppState>,
    SdkProject(project_id): SdkProject,
    Json(context): Json<UserContext>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Fetch all enabled flags for the project
    let flags = sqlx::query!(
        r#"
        SELECT id, key, enabled, rollout_percentage
        FROM feature_flags
        WHERE project_id = $1
        "#,
        project_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch flags: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch flags".to_string())
    })?;

    let mut result_flags = HashMap::new();

    // Evaluate each flag
    for flag in flags {
        // Fetch rules for this flag
        let rules = sqlx::query!(
            r#"
            SELECT rule_type, rule_value, enabled, priority
            FROM flag_rules
            WHERE flag_id = $1 AND enabled = true
            ORDER BY priority DESC
            "#,
            flag.id
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Failed to fetch rules: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch rules".to_string())
        })?;

        // Convert to evaluation types
        let flag_data = FlagData {
            key: flag.key.clone(),
            enabled: flag.enabled,
            rollout_percentage: flag.rollout_percentage,
        };

        let rule_data: Vec<RuleData> = rules
            .into_iter()
            .map(|r| RuleData {
                rule_type: r.rule_type,
                rule_value: r.rule_value,
                enabled: r.enabled,
                priority: r.priority,
            })
            .collect();

        // Evaluate the flag
        let evaluation = evaluate_flag(&flag_data, &rule_data, &context);

        // Store result
        result_flags.insert(
            flag.key,
            FlagState {
                enabled: evaluation.enabled,
                reason: evaluation.reason,
            },
        );

        // Optional: Log the evaluation (async in production)
        let user_identifier = context.user_id.as_ref()
            .or(context.user_email.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("anonymous");

        let _ = sqlx::query!(
            r#"
            INSERT INTO flag_evaluations (flag_id, user_identifier, result)
            VALUES ($1, $2, $3)
            "#,
            flag.id,
            user_identifier,
            evaluation.enabled
        )
        .execute(&state.db)
        .await;
        // Ignore logging errors - don't fail the request
    }

    Ok(Json(EvaluateResponse { flags: result_flags }))
}