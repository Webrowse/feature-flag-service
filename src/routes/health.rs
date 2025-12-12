use axum::{ Json, http::StatusCode };
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthData {
    status: u16,
}

pub async fn health() -> Json<HealthData> {
    let health_data = HealthData { status: StatusCode::OK.as_u16() };
    Json(health_data)
}