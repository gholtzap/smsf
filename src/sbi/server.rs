use crate::context::ue_sms_context::UeSmsContextStore;
use crate::db::Database;
use crate::nf_client::amf::AmfClient;
use crate::sbi::handlers::{activation, deactivation, send_mt_sms, send_sms, update};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, patch, post, put};
use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub struct AppState {
    pub context_store: UeSmsContextStore,
    pub db: Database,
    pub amf_client: AmfClient,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let activation_state = Arc::new(activation::AppState {
        context_store: state.context_store.clone(),
        db: state.db.clone(),
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
    });

    let send_mt_sms_state = Arc::new(send_mt_sms::AppState {
        context_store: state.context_store.clone(),
        db: state.db.clone(),
        amf_client: state.amf_client.clone(),
    });

    Router::new()
        .route("/health", axum::routing::get(health_check))
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
        .layer(TraceLayer::new_for_http())
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
