use crate::context::ue_sms_context::UeSmsContextStore;
use crate::db::Database;
use crate::nf_client::amf::AmfClient;
use crate::nf_client::udm::UdmClient;
use crate::sbi::models::{ProblemDetails, SmsDeliveryReportStatus, SmsRecordDeliveryData};
use crate::sbi::multipart::parse_multipart_sms;
use crate::sms::delivery::SmsDeliveryService;
use crate::sms::types::SmsDeliveryData;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;
use tracing::error;

pub struct AppState {
    pub context_store: UeSmsContextStore,
    pub db: Database,
    pub amf_client: AmfClient,
    pub udm_client: UdmClient,
    pub delivery_service: Arc<SmsDeliveryService>,
}

pub async fn send_downlink_sms(
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
            if !auth_data.mt_sms_allowed {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ProblemDetails::new(
                        403,
                        "MT-SMS is not allowed for this subscriber".to_string(),
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
    let sms_data = SmsDeliveryData {
        sms_record_id: sms_record_id.clone(),
        sms_msg: sms_payload,
    };

    match state.delivery_service.deliver_mt_sms(&supi, sms_data).await {
        Ok(record_id) => {
            let response_data = SmsRecordDeliveryData {
                sms_record_id: record_id,
                delivery_status: SmsDeliveryReportStatus::Pending,
            };
            (StatusCode::OK, Json(response_data)).into_response()
        }
        Err(e) => {
            error!("Failed to deliver MT SMS: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemDetails::internal_error(format!(
                    "Failed to deliver SMS: {}",
                    e
                ))),
            )
                .into_response()
        }
    }
}
