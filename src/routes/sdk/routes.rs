use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::evaluation::{evaluate_flag, FlagData, RuleData};
use crate::routes::sdk_auth::SdkProject;
use crate::state::AppState;
use super::{EvaluateRequest, EvaluateResponse, FlagState};

// Database row types for batch queries
#[derive(Debug, sqlx::FromRow)]
struct EnvironmentRow {
    id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
struct FlagRow {
    id: Uuid,
    key: String,
    enabled: bool,
    rollout_percentage: i32,
}

#[derive(Debug, sqlx::FromRow)]
struct RuleRow {
    flag_id: Uuid,
    rule_type: String,
    rule_value: String,
    enabled: bool,
    priority: i32,
}

/// Evaluate all flags for a project/environment based on user context
/// Uses optimized batch loading of rules to minimize database round trips
pub async fn evaluate(
    State(state): State<AppState>,
    SdkProject(project_id): SdkProject,
    Json(request): Json<EvaluateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let context = request.context;
    let environment_key = request.environment;

    // Step 1: Verify environment exists and get environment_id
    let environment: Option<EnvironmentRow> = sqlx::query_as(
        r#"
        SELECT id FROM environments
        WHERE project_id = $1 AND key = $2
        "#,
    )
    .bind(project_id)
    .bind(&environment_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch environment: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch environment".to_string())
    })?;

    let environment_id = match environment {
        Some(env) => env.id,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                format!("Environment '{}' not found", environment_key),
            ));
        }
    };

    // Step 2: Fetch all flags for this environment in one query
    let flags: Vec<FlagRow> = sqlx::query_as(
        r#"
        SELECT id, key, enabled, rollout_percentage
        FROM feature_flags
        WHERE environment_id = $1
        "#,
    )
    .bind(environment_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch flags: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch flags".to_string())
    })?;

    if flags.is_empty() {
        return Ok(Json(EvaluateResponse {
            flags: HashMap::new(),
        }));
    }

    // Step 3: Collect all flag IDs for batch rule loading
    let flag_ids: Vec<Uuid> = flags.iter().map(|f| f.id).collect();

    // Step 4: Preload ALL rules for ALL flags in ONE query (key optimization!)
    let rules: Vec<RuleRow> = sqlx::query_as(
        r#"
        SELECT flag_id, rule_type, rule_value, enabled, priority
        FROM flag_rules
        WHERE flag_id = ANY($1)
        ORDER BY priority DESC
        "#,
    )
    .bind(&flag_ids)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch rules: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch rules".to_string())
    })?;

    // Step 5: Build a HashMap<flag_id, Vec<RuleData>> for fast lookup
    let mut rules_by_flag: HashMap<Uuid, Vec<RuleData>> = HashMap::new();
    for rule in rules {
        let rule_data = RuleData {
            rule_type: rule.rule_type,
            rule_value: rule.rule_value,
            enabled: rule.enabled,
            priority: rule.priority,
        };
        rules_by_flag
            .entry(rule.flag_id)
            .or_insert_with(Vec::new)
            .push(rule_data);
    }

    // Step 6: Evaluate each flag using the preloaded rules
    let mut result_flags = HashMap::new();
    let mut evaluation_records = Vec::new();

    let user_identifier = context
        .user_id
        .as_ref()
        .or(context.user_email.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("anonymous");

    for flag in &flags {
        // Get rules for this flag from our preloaded HashMap (O(1) lookup)
        let flag_rules = rules_by_flag.get(&flag.id).map(|v| v.as_slice()).unwrap_or(&[]);

        // Convert to evaluation types
        let flag_data = FlagData {
            key: flag.key.clone(),
            enabled: flag.enabled,
            rollout_percentage: flag.rollout_percentage,
        };

        // Evaluate the flag
        let evaluation = evaluate_flag(&flag_data, flag_rules, &context);

        // Store result
        result_flags.insert(
            flag.key.clone(),
            FlagState {
                enabled: evaluation.enabled,
                reason: evaluation.reason,
            },
        );

        // Collect evaluation record for batch insert
        evaluation_records.push((flag.id, user_identifier.to_string(), evaluation.enabled));
    }

    // Step 7: Batch insert evaluation logs (async, don't block response)
    // Using a single INSERT with multiple values for efficiency
    if !evaluation_records.is_empty() {
        let flag_ids: Vec<Uuid> = evaluation_records.iter().map(|(id, _, _)| *id).collect();
        let user_ids: Vec<String> = evaluation_records.iter().map(|(_, u, _)| u.clone()).collect();
        let results: Vec<bool> = evaluation_records.iter().map(|(_, _, r)| *r).collect();

        let _ = sqlx::query(
            r#"
            INSERT INTO flag_evaluations (flag_id, user_identifier, result)
            SELECT * FROM UNNEST($1::uuid[], $2::text[], $3::bool[])
            "#,
        )
        .bind(&flag_ids)
        .bind(&user_ids)
        .bind(&results)
        .execute(&state.db)
        .await;
    }

    Ok(Json(EvaluateResponse { flags: result_flags }))
}
