use serde::{Serialize, Deserialize };
use uuid::Uuid;
use chrono::DateTime;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub done: bool,
    pub created_at: DateTime<Utc>,
}