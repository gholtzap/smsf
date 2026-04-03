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
    let content_type_valid = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("application/json-patch+json"))
        .unwrap_or(false);

    if !content_type_valid {
        return (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Json(ProblemDetails::new(
                415,
                "Content-Type must be application/json-patch+json".to_string(),
            )),
        )
            .into_response();
    }

    let if_match: Option<String> = headers
        .get(axum::http::header::IF_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_matches('"').to_string());

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

    let result = state.context_store.try_update(&supi, |ctx| {
        if let Some(ref etag) = if_match {
            if *etag != ctx.etag {
                return Err((
                    StatusCode::PRECONDITION_FAILED,
                    ProblemDetails::new(
                        412,
                        "ETag mismatch - context has been modified".to_string(),
                    ),
                ));
            }
        }

        let mut context_json = serde_json::to_value(&ctx.to_data()).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ProblemDetails::internal_error(format!("Failed to serialize context: {}", e)),
            )
        })?;

        json_patch::patch(&mut context_json, &patch).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                ProblemDetails::bad_request(format!("Failed to apply patch: {}", e)),
            )
        })?;

        let updated_data: UeSmsContextData =
            serde_json::from_value(context_json).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    ProblemDetails::bad_request(format!(
                        "Invalid context data after patch: {}",
                        e
                    )),
                )
            })?;

        if updated_data.supi != ctx.supi {
            return Err((
                StatusCode::BAD_REQUEST,
                ProblemDetails::bad_request("Cannot modify SUPI".to_string()),
            ));
        }

        ctx.update_from_data(updated_data);
        Ok(())
    });

    let updated_context = match result {
        Some(Ok(ctx)) => ctx,
        Some(Err((status, problem))) => {
            return (status, Json(problem)).into_response();
        }
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
