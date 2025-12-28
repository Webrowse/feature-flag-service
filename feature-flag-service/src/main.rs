mod config;
mod routes;
mod state;
mod evaluation;

use sqlx::PgPool;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let config = config::Config::from_env();

    let db = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Error connecting DB");

    let state = state::AppState { db: db.clone() };

    let app = routes::routes().with_state(state)
        .layer(axum::Extension(db))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(config.addr()).await.unwrap();

    println!("server is chilling at http://{}", config.addr());

    axum::serve(listener, app).await.unwrap();
}
