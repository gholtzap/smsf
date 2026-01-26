use crate::config::OAuth2Config;
use crate::context::ue_sms_context::UeSmsContextStore;
use crate::db::Database;
use crate::nf_client::amf::AmfClient;
use crate::nf_client::udm::UdmClient;
use crate::sbi::handlers::{activation, deactivation, delivery_report, send_mt_sms, send_sms, update};
use crate::sbi::middleware::oauth2::oauth2_auth;
use crate::sms::delivery::SmsDeliveryService;
use crate::sms::status_report::StatusReportService;
use axum::http::StatusCode;
use axum::middleware as axum_middleware;
use axum::response::IntoResponse;
use axum::routing::{delete, patch, post, put};
use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub struct AppState {
    pub context_store: UeSmsContextStore,
    pub db: Database,
    pub amf_client: AmfClient,
    pub udm_client: UdmClient,
    pub delivery_service: Arc<SmsDeliveryService>,
    pub status_report_service: Arc<StatusReportService>,
    pub oauth2_config: OAuth2Config,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let activation_state = Arc::new(activation::AppState {
        context_store: state.context_store.clone(),
        db: state.db.clone(),
        udm_client: state.udm_client.clone(),
    });

    let deactivation_state = Arc::new(deactivation::AppState {
        context_store: state.context_store.clone(),
        db: state.db.clone(),
    });

    let update_state = Arc::new(update::AppState {
        context_store: state.context_store.clone(),
        db: state.db.clone(),
    });

    let send_sms_state = Arc::new(send_sms::AppState {
        context_store: state.context_store.clone(),
        db: state.db.clone(),
        udm_client: state.udm_client.clone(),
    });

    let send_mt_sms_state = Arc::new(send_mt_sms::AppState {
        context_store: state.context_store.clone(),
        db: state.db.clone(),
        amf_client: state.amf_client.clone(),
        udm_client: state.udm_client.clone(),
        delivery_service: state.delivery_service.clone(),
    });

    let delivery_report_state = Arc::new(delivery_report::AppState {
        db: state.db.clone(),
        status_report_service: state.status_report_service.clone(),
    });

    let protected = Router::new()
        .route(
            "/nsmsf-sms/v1/ue-contexts/:supi",
            put(activation::activate_sms_service).with_state(activation_state),
        )
        .route(
            "/nsmsf-sms/v1/ue-contexts/:supi",
            patch(update::update_sms_context).with_state(update_state),
        )
        .route(
            "/nsmsf-sms/v1/ue-contexts/:supi",
            delete(deactivation::deactivate_sms_service).with_state(deactivation_state),
        )
        .route(
            "/nsmsf-sms/v1/ue-contexts/:supi/sendsms",
            post(send_sms::send_uplink_sms).with_state(send_sms_state),
        )
        .route(
            "/nsmsf-sms/v1/ue-contexts/:supi/send-mt-sms",
            post(send_mt_sms::send_downlink_sms).with_state(send_mt_sms_state),
        )
        .route(
            "/nsmsf-sms/v1/ue-contexts/:supi/delivery-report",
            post(delivery_report::receive_delivery_report).with_state(delivery_report_state),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            oauth2_auth,
        ));

    Router::new()
        .route("/health", axum::routing::get(health_check))
        .merge(protected)
        .layer(TraceLayer::new_for_http())
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
