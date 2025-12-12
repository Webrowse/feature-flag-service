use axum::{
    http::StatusCode,
    middleware::Next,
    response::{Response, IntoResponse},
    extract::{ Request, FromRequestParts},
    http::request::Parts,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::env;
use uuid::Uuid;
use serde::Deserialize;

pub struct JwtUser(pub Uuid);


impl<S> FromRequestParts<S> for JwtUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Uuid>()
            .copied()
            .map(JwtUser)
            .ok_or((StatusCode::UNAUTHORIZED, "missing user"))
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

pub async fn require_auth(mut req: Request, next: Next) -> Result<Response, impl IntoResponse> {
    let auth_header = req.headers().get("authorization").and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return Err((StatusCode::UNAUTHORIZED, "missing token"));
        }
    };

    let secret = env::var("JWT_SECRET").expect("JWT is not found");

    let token_data =  match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("JWT decode error: {}", e);
            return Err((StatusCode::UNAUTHORIZED, "invalid token"));
        }
    };

    match Uuid::parse_str(&token_data.claims.sub) {
        Ok(user_id) => {
            req.extensions_mut().insert(user_id);
            Ok(next.run(req).await)
        }
        Err(_) => {
            Err((StatusCode::UNAUTHORIZED, "invalid subject"))
        }
    }
}

