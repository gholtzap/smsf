use crate::context::ue_sms_context::{UeSmsContext, UeSmsContextStore};
use crate::db::Database;
use crate::nf_client::udm::UdmClient;
use crate::sbi::models::{ProblemDetails, UeSmsContextData};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct AppState {
    pub context_store: UeSmsContextStore,
    pub db: Database,
    pub udm_client: UdmClient,
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

    match state.udm_client.get_sms_authorization(&supi).await {
        Ok(auth_data) => {
            if !auth_data.sms_subscribed {
                warn!("SMS activation rejected for SUPI {}: not subscribed", supi);
                return (
                    StatusCode::FORBIDDEN,
                    Json(ProblemDetails::new(
                        403,
                        "SMS service is not subscribed for this user".to_string(),
                    )),
                )
                    .into_response();
            }

            if !auth_data.mo_sms_allowed && !auth_data.mt_sms_allowed {
                warn!("SMS activation rejected for SUPI {}: MO and MT SMS both barred", supi);
                return (
                    StatusCode::FORBIDDEN,
                    Json(ProblemDetails::new(
                        403,
                        "SMS service is barred for this user".to_string(),
                    )),
                )
                    .into_response();
            }

            info!(
                "SMS authorization validated for SUPI {}: MSISDN={:?}, MO={}, MT={}",
                supi, auth_data.msisdn, auth_data.mo_sms_allowed, auth_data.mt_sms_allowed
            );
        }
        Err(e) => {
            error!("Failed to get SMS authorization from UDM: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemDetails::internal_error(
                    "Failed to validate SMS subscription".to_string(),
                )),
            )
                .into_response();
        }
    }

    let update_data = context_data.clone();
    if let Some(updated_ctx) = state.context_store.update(&supi, |ctx| {
        ctx.update_from_data(update_data);
    }) {
        if let Err(e) = state.db.update_ue_context(&updated_ctx).await {
            error!("Failed to update context in DB: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemDetails::internal_error(
                    "Failed to update context".to_string(),
                )),
            )
                .into_response();
        }

        info!("SMS service updated for SUPI: {}", supi);
        return (
            StatusCode::OK,
            [(axum::http::header::ETAG, updated_ctx.etag.clone())],
            Json(updated_ctx.to_data()),
        )
            .into_response();
    }

    let context = UeSmsContext::from_data(context_data);

    if let Err(e) = state.db.save_ue_context(&context).await {
        error!("Failed to save context: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProblemDetails::internal_error(
                "Failed to save context".to_string(),
            )),
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
