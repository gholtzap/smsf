use crate::context::ue_sms_context::UeSmsContextStore;
use crate::db::Database;
use crate::sbi::models::{ProblemDetails, UeSmsContextData};
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;
use tracing::info;

pub struct AppState {
    pub context_store: UeSmsContextStore,
    pub db: Database,
}

pub async fn update_sms_context(
    State(state): State<Arc<AppState>>,
    Path(supi): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let if_match = headers
        .get(axum::http::header::IF_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_matches('"'));

    let current_context = match state.context_store.get(&supi) {
        Some(ctx) => ctx,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ProblemDetails::not_found(
                    "UE SMS context not found".to_string(),
                )),
            )
                .into_response();
        }
    };

    if let Some(etag) = if_match {
        if etag != current_context.etag {
            return (
                StatusCode::PRECONDITION_FAILED,
                Json(ProblemDetails::new(
                    412,
                    "ETag mismatch - context has been modified".to_string(),
                )),
            )
                .into_response();
        }
    }

    let patch: json_patch::Patch = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ProblemDetails::bad_request(format!(
                    "Invalid JSON Patch: {}",
                    e
                ))),
            )
                .into_response();
        }
    };

    let mut context_json = match serde_json::to_value(&current_context.to_data()) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemDetails::internal_error(format!(
                    "Failed to serialize context: {}",
                    e
                ))),
            )
                .into_response();
        }
    };

    if let Err(e) = json_patch::patch(&mut context_json, &patch) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails::bad_request(format!(
                "Failed to apply patch: {}",
                e
            ))),
        )
            .into_response();
    }

    let updated_data: UeSmsContextData = match serde_json::from_value(context_json) {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ProblemDetails::bad_request(format!(
                    "Invalid context data after patch: {}",
                    e
                ))),
            )
                .into_response();
        }
    };

    if updated_data.supi != supi {
        return (
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails::bad_request(
                "Cannot modify SUPI".to_string(),
            )),
        )
            .into_response();
    }

    let updated_context = match state.context_store.update(&supi, |ctx| {
        ctx.update_from_data(updated_data);
    }) {
        Some(ctx) => ctx,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ProblemDetails::not_found(
                    "UE SMS context not found".to_string(),
                )),
            )
                .into_response();
        }
    };

    if let Err(e) = state.db.update_ue_context(&updated_context).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProblemDetails::internal_error(format!(
                "Failed to update context: {}",
                e
            ))),
        )
            .into_response();
    }

    info!("SMS context updated for SUPI: {}", supi);

    (
        StatusCode::OK,
        [(axum::http::header::ETAG, updated_context.etag.clone())],
        Json(updated_context.to_data()),
    )
        .into_response()
}
