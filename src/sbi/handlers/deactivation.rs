use crate::context::ue_sms_context::UeSmsContextStore;
use crate::db::Database;
use crate::sbi::models::ProblemDetails;
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

pub async fn deactivate_sms_service(
    State(state): State<Arc<AppState>>,
    Path(supi): Path<String>,
    headers: HeaderMap,
) -> Response {
    let if_match: Option<String> = headers
        .get(axum::http::header::IF_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_matches('"').to_string());

    let context = match state.context_store.get(&supi) {
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

    if let Some(ref etag) = if_match {
        if *etag != context.etag {
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

    if let Err(e) = state.db.delete_ue_context(&supi).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProblemDetails::internal_error(format!(
                "Failed to delete context: {}",
                e
            ))),
        )
            .into_response();
    }

    state.context_store.remove(&supi);

    info!("SMS service deactivated for SUPI: {}", supi);
    StatusCode::NO_CONTENT.into_response()
}
