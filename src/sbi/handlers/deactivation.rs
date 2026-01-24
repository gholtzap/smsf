use crate::context::ue_sms_context::UeSmsContextStore;
use crate::db::Database;
use crate::sbi::models::ProblemDetails;
use axum::extract::{Path, State};
use axum::http::StatusCode;
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
) -> Response {
    if !state.context_store.contains(&supi) {
        return (
            StatusCode::NOT_FOUND,
            Json(ProblemDetails::not_found(
                "UE SMS context not found".to_string(),
            )),
        )
            .into_response();
    }

    state.context_store.remove(&supi);

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

    info!("SMS service deactivated for SUPI: {}", supi);
    StatusCode::NO_CONTENT.into_response()
}
