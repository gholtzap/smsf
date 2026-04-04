use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use crate::sbi::server::AppState;
use crate::sbi::models::ProblemDetails;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iss: Option<String>,
    pub aud: Option<serde_json::Value>,
    pub exp: i64,
    pub iat: Option<i64>,
    pub nbf: Option<i64>,
    pub scope: Option<String>,
    #[serde(rename = "nfInstanceId")]
    pub nf_instance_id: Option<String>,
    #[serde(rename = "nfType")]
    pub nf_type: Option<String>,
}

pub async fn oauth2_auth(
    State(state): State<std::sync::Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, axum::Json<ProblemDetails>)> {
    if !state.oauth2_config.enabled {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Err((
                StatusCode::UNAUTHORIZED,
                axum::Json(ProblemDetails {
                    problem_type: Some("https://example.com/unauthorized".to_string()),
                    title: Some("Unauthorized".to_string()),
                    status: 401,
                    detail: Some("Missing or invalid Authorization header".to_string()),
                    instance: None,
                }),
            ));
        }
    };

    let decoding_key = DecodingKey::from_secret(state.oauth2_config.secret_key.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.algorithms = vec![Algorithm::HS256];

    if !state.oauth2_config.issuer.is_empty() {
        validation.set_issuer(&[&state.oauth2_config.issuer]);
    }

    if !state.oauth2_config.audience.is_empty() {
        validation.set_audience(&state.oauth2_config.audience);
    } else {
        validation.validate_aud = false;
    }

    let token_data = decode::<TokenClaims>(token, &decoding_key, &validation).map_err(|e| {
        tracing::warn!("Token validation failed: {}", e);
        (
            StatusCode::UNAUTHORIZED,
            axum::Json(ProblemDetails {
                problem_type: Some("https://example.com/unauthorized".to_string()),
                title: Some("Unauthorized".to_string()),
                status: 401,
                detail: Some("Token validation failed".to_string()),
                instance: None,
            }),
        )
    })?;

    if let Some(required_scope) = &state.oauth2_config.required_scope {
        let has_scope = token_data.claims.scope
            .as_ref()
            .map(|s| s.split_whitespace().any(|scope| scope == required_scope))
            .unwrap_or(false);

        if !has_scope {
            return Err((
                StatusCode::FORBIDDEN,
                axum::Json(ProblemDetails {
                    problem_type: Some("https://example.com/forbidden".to_string()),
                    title: Some("Forbidden".to_string()),
                    status: 403,
                    detail: Some("Insufficient scope".to_string()),
                    instance: None,
                }),
            ));
        }
    }

    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}
