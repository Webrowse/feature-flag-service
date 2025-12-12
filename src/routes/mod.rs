use axum::{
    Router,
    extract::Request,
    middleware,
    routing::{get, post, put},
};

mod auth;
mod health;
mod middleware_auth;
mod tasks;


pub use auth::register;
pub use health::health;

use crate::routes::auth::login;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {

    let task_router = Router::new()
    .route("/", post(tasks::routes::create).get(tasks::routes::list))
    .route("/{id}", put(tasks::routes::update).delete(tasks::routes::delete));

    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .nest(
            "/api",
            Router::new()
                .route("/me", get(me_handler))
                .nest("/task", task_router)
                .layer(middleware::from_fn(middleware_auth::require_auth)),
        )
}

async fn root() -> &'static str {
    "Welcome to the API written in Rust"
}

async fn me_handler(req: Request<axum::body::Body>) -> impl axum::response::IntoResponse {
    let user_id = req.extensions().get::<uuid::Uuid>().cloned();
    match user_id {
        Some(u) => (axum::http::StatusCode::OK, format!("user_id: {}", u)),
        None => (axum::http::StatusCode::UNAUTHORIZED, "no user".into()),
    }
}
