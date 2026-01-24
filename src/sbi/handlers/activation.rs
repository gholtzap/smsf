use crate::context::ue_sms_context::{UeSmsContext, UeSmsContextStore};
use crate::db::Database;
use crate::sbi::models::{ProblemDetails, UeSmsContextData};
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

pub async fn activate_sms_service(
    State(state): State<Arc<AppState>>,
    Path(supi): Path<String>,
    Json(context_data): Json<UeSmsContextData>,
) -> Response {
    if context_data.supi != supi {
        return (
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails::bad_request(
                "SUPI in path does not match SUPI in body".to_string(),
            )),
        )
            .into_response();
    }

    if state.context_store.contains(&supi) {
        return (
            StatusCode::CONFLICT,
            Json(ProblemDetails::conflict(
                "UE SMS context already exists".to_string(),
            )),
        )
            .into_response();
    }

    let context = UeSmsContext::from_data(context_data);

    if let Err(e) = state.db.save_ue_context(&context).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProblemDetails::internal_error(format!(
                "Failed to save context: {}",
                e
            ))),
        )
            .into_response();
    }

    let etag = context.etag.clone();
    state.context_store.insert(supi.clone(), context.clone());

    info!("SMS service activated for SUPI: {}", supi);

    (
        StatusCode::CREATED,
        [(axum::http::header::ETAG, etag)],
        Json(context.to_data()),
    )
        .into_response()
}
