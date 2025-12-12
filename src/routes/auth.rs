use axum::{ extract::{ Json, State }, http::StatusCode, response::IntoResponse, };
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use uuid::Uuid;
use argon2::{Argon2, PasswordHasher, PasswordVerifier };
use rand::rngs::OsRng;
use argon2::password_hash::{SaltString, PasswordHash};
use jsonwebtoken::{EncodingKey, Header, encode };
use std::env;
use chrono::Utc;
use chrono::Duration; 

#[derive(Deserialize)]
pub struct RegistrationRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub id: Uuid,
    pub email: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegistrationRequest>
) -> impl IntoResponse {

    if payload.email.trim().is_empty() || payload.password.len() < 8 {
        return (StatusCode::BAD_REQUEST, "invalid payload").into_response();
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon = Argon2::default();

    let password_hash = argon.hash_password(payload.password.as_bytes(), &salt).unwrap().to_string();
    let user_id = Uuid::new_v4();

    let res = sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash)
        VALUES ($1,$2,$3)
        "#,
        user_id, payload.email, password_hash
    )
    .execute(&state.db)
    .await;

    match res {
        Ok(_) => (StatusCode::CREATED, Json(RegisterResponse { id: user_id, email: payload.email})).into_response(),
        Err(e) => {
            eprintln!("DB insert error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "could not create user").into_response()
        }
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let row = sqlx::query!(
        r#"
        SELECT id, password_hash FROM users WHERE email = $1
        "#,
        payload.email
    )
    .fetch_optional(&state.db)
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::UNAUTHORIZED, "Invalid Credential").into_response(),
        Err(e) => {
            eprintln!("DB Error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
        }
    };

    let parsed_hash = PasswordHash::new(&row.password_hash).unwrap();
    let argon = Argon2::default();
    let verify = argon.verify_password(payload.password.as_bytes(), &parsed_hash).is_ok();

    if !verify {
        return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
    }

    // create JWT
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET not found");
    let now = Utc::now();
    let exp = now + Duration::hours(24);
    let claims = Claims {
        sub: row.id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| {
            eprintln!("jwt encode error: {}",e);
            (StatusCode::INTERNAL_SERVER_ERROR, "token error")
        });
    
    match token {
        Ok(t) => (StatusCode::OK, Json(LoginResponse {token: t })).into_response(),
        Err(err_resp) => err_resp.into_response(),
    }
}