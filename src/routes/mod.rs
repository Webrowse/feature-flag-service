use axum::{
    extract::Request,
    middleware,
    routing::{get, post},
    Router,
};

mod auth;
mod health;
mod middleware_auth;
mod projects;
mod flags;
mod rules;
mod sdk_auth;
mod sdk;
pub mod environments; 

pub use auth::register;
pub use health::health;

use crate::routes::auth::login;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    let projects_router = Router::new()
        .route(
            "/",
            post(projects::routes::create).get(projects::routes::list),
        )
        .route(
            "/{id}",
            get(projects::routes::get)
                .put(projects::routes::update)
                .delete(projects::routes::delete),
        )
        .route(
            "/{id}/regenerate-key",
            post(projects::routes::regenerate_key),
        );

    // Rules router - handles /rules and /rules/{rule_id}
    let rules_router = Router::new()
        .route("/", post(rules::routes::create).get(rules::routes::list))
        .route(
            "/{rule_id}",
            get(rules::routes::get)
                .put(rules::routes::update)
                .delete(rules::routes::delete),
        );

    // Flags router - handles flags AND nests rules under /{flag_id}/rules
    let flags_router = Router::new()
        .route("/", post(flags::routes::create).get(flags::routes::list))
        .route(
            "/{flag_id}",
            get(flags::routes::get)
                .put(flags::routes::update)
                .delete(flags::routes::delete),
        )
        .route("/{flag_id}/toggle", post(flags::routes::toggle))
        .nest("/{flag_id}/rules", rules_router);

    // Environments router - handles /environments and /environments/{environment_id}
    let environments_router = Router::new()
        .route(
            "/",
            post(environments::routes::create).get(environments::routes::list),
        )
        .route(
            "/{environment_id}",
            get(environments::routes::get)
                .put(environments::routes::update)
                .delete(environments::routes::delete),
        );  

    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .nest(
            "/api",
            Router::new()
                .route("/me", get(me_handler))
                .nest("/projects", projects_router)
                .nest("/projects/{project_id}/environments", environments_router)
                .nest("/projects/{project_id}/environments/{environment_id}/flags", flags_router)
                .layer(middleware::from_fn(middleware_auth::require_auth)),
        )
        .nest(
            "/sdk/v1",
            Router::new()
                .route("/evaluate", post(sdk::routes::evaluate))
                .layer(middleware::from_fn(sdk_auth::require_sdk_key)),
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