use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateTask {
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub done: Option<bool>,
}