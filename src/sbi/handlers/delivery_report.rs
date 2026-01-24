use crate::db::Database;
use crate::sbi::models::{ProblemDetails, SmsDeliveryReportData};
use crate::sms::status_report::StatusReportService;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;
use tracing::{error, info};

pub struct AppState {
    pub db: Database,
    pub status_report_service: Arc<StatusReportService>,
}

pub async fn receive_delivery_report(
    State(state): State<Arc<AppState>>,
    Path(supi): Path<String>,
    Json(report): Json<SmsDeliveryReportData>,
) -> Response {
    match state.db.get_sms_record(&report.sms_record_id).await {
        Ok(Some(_)) => {},
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ProblemDetails::not_found(format!(
                    "SMS record not found: {}",
                    report.sms_record_id
                ))),
            )
                .into_response();
        }
        Err(e) => {
            error!("Failed to get SMS record: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemDetails::internal_error(format!(
                    "Failed to retrieve SMS record: {}",
                    e
                ))),
            )
                .into_response();
        }
    }

    let delivery_status = report.delivery_status.clone();

    if let Err(e) = state
        .db
        .update_sms_status(&report.sms_record_id, delivery_status.clone())
        .await
    {
        error!("Failed to update SMS delivery status: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProblemDetails::internal_error(format!(
                "Failed to update delivery status: {}",
                e
            ))),
        )
            .into_response();
    }

    if let Err(e) = state
        .status_report_service
        .handle_delivery_status_change(&report.sms_record_id, delivery_status.clone())
        .await
    {
        error!("Failed to send status report: {}", e);
    }

    info!(
        "Delivery report received for SUPI: {}, SMS record: {}, status: {:?}",
        supi, report.sms_record_id, delivery_status
    );

    StatusCode::NO_CONTENT.into_response()
}
