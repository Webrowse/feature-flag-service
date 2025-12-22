pub mod routes;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export UserContext from evaluation module
pub use crate::evaluation::UserContext;

#[derive(Debug, Serialize)]
pub struct EvaluateResponse {
    pub flags: HashMap<String, FlagState>,
}

#[derive(Debug, Serialize)]
pub struct FlagState {
    pub enabled: bool,
    pub reason: String,
}