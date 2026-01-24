use crate::context::ue_sms_context::UeSmsContextStore;
use crate::db::Database;
use crate::nf_client::udm::UdmClient;
use crate::sbi::models::{ProblemDetails, SmsRecordData};
use crate::sbi::multipart::parse_multipart_sms;
use crate::sms::types::{SmsDeliveryStatus, SmsRecord};
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use base64::Engine;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tracing::{error, info};

pub struct AppState {
    pub context_store: UeSmsContextStore,
    pub db: Database,
    pub udm_client: UdmClient,
}

pub async fn send_uplink_sms(
    State(state): State<Arc<AppState>>,
    Path(supi): Path<String>,
    headers: HeaderMap,
    body: Bytes,
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

    match state.udm_client.get_sms_authorization(&supi).await {
        Ok(auth_data) => {
            if !auth_data.mo_sms_allowed {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ProblemDetails::new(
                        403,
                        "MO-SMS is not allowed for this subscriber".to_string(),
                    )),
                )
                    .into_response();
            }
        }
        Err(e) => {
            error!("Failed to get SMS authorization from UDM: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemDetails::internal_error(format!(
                    "Failed to check SMS authorization: {}",
                    e
                ))),
            )
                .into_response();
        }
    }

    let content_type = match headers.get(axum::http::header::CONTENT_TYPE) {
        Some(ct) => ct.to_str().unwrap_or(""),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ProblemDetails::bad_request(
                    "Missing Content-Type header".to_string(),
                )),
            )
                .into_response();
        }
    };

    if !content_type.starts_with("multipart/related") {
        return (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Json(ProblemDetails::new(
                415,
                "Content-Type must be multipart/related".to_string(),
            )),
        )
            .into_response();
    }

    let boundary = content_type
        .split("boundary=")
        .nth(1)
        .and_then(|b| b.split(';').next())
        .unwrap_or("");

    if boundary.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ProblemDetails::bad_request(
                "Missing boundary in Content-Type".to_string(),
            )),
        )
            .into_response();
    }

    let (_json_data, sms_payload) = match parse_multipart_sms(boundary, body).await {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to parse multipart SMS: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(ProblemDetails::bad_request(format!(
                    "Failed to parse multipart: {}",
                    e
                ))),
            )
                .into_response();
        }
    };

    let sms_record_id = uuid::Uuid::new_v4().to_string();

    let context = state.context_store.get(&supi).unwrap();
    let now = Utc::now();
    let sms_record = SmsRecord {
        sms_record_id: sms_record_id.clone(),
        sms_payload: sms_payload.clone(),
        delivery_status: SmsDeliveryStatus::Accepted,
        gpsi: context.gpsi.clone(),
        supi: supi.clone(),
        amf_id: context.amf_id.clone(),
        retry_count: 0,
        next_retry_at: None,
        expires_at: now + Duration::days(1),
        created_at: now,
        updated_at: now,
    };

    if let Err(e) = state.db.save_sms_record(&sms_record).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProblemDetails::internal_error(format!(
                "Failed to save SMS record: {}",
                e
            ))),
        )
            .into_response();
    }

    info!("Uplink SMS received from SUPI: {}", supi);

    let response_data = SmsRecordData {
        sms_record_id: sms_record_id.clone(),
        sms_payload: base64::engine::general_purpose::STANDARD.encode(&sms_payload),
        gpsi: context.gpsi.clone(),
    };

    (StatusCode::OK, Json(response_data)).into_response()
}
