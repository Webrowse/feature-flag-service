pub mod routes;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// MODELS

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FeatureFlag {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub key: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub rollout_percentage: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFlagRequest {
    pub name: String,
    pub key: String,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub rollout_percentage: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFlagRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub rollout_percentage: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct FlagResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub key: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub rollout_percentage: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// HELPER FUNCTIONS

// Validating the flag key
pub fn validate_flag_key(key: &str) -> Result<(), String> {
    if key.is_empty() {                                         // Checks if flag key is empty
        return Err("Flag key cannot be empty".to_string());
    }

    if key.len()>64 {
        return Err("Flag key is too long (Max: 64 characters)".to_string());      // Max size of flag
    }

    if !key.chars().next().unwrap().is_ascii_alphabetic() {
        return Err("Flag must start with an alphabet".to_string());
    }

    if !key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-') {
        return Err("flag can only be \n - lowercase letters\n - numbers\n - underscores\n - and hypens.".to_string());
    }

    Ok(())
}

// Checks if percentage number is between the number 0 to 100
pub fn validate_rollout_percentage(percentage: i32) -> Result<(), String> {
    if !(0..100).contains(&percentage) {
        return Err("Roolout percentage must be between 0 to 100".to_string());
    }

    Ok(())
}