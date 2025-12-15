pub mod routes;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// MODELS

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sdk_key: String,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sdk_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// HELPER FUNCTIONS

/// Generate a secure SDK key
/// Format: "sdk_" + 32 random alphanumeric characters
pub fn generate_sdk_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const KEY_LENGTH: usize = 32;

    let mut rng = rand::thread_rng();
    let key: String = (0..KEY_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("sdk_{}", key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sdk_key() {
        let key1 = generate_sdk_key();
        let key2 = generate_sdk_key();

        assert!(key1.starts_with("sdk_"));
        assert_eq!(key1.len(), 36); // "sdk_" (4) + 32 chars
        assert_ne!(key1, key2); // Should be random
    }
}
