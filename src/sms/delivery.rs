use super::types::{SmsDeliveryData, SmsDeliveryStatus, SmsRecord};
use crate::context::ue_sms_context::UeSmsContextStore;
use crate::db::Database;
use crate::nf_client::amf::AmfClient;
use anyhow::Result;
use chrono::{Duration, Utc};
use tracing::{info, warn};

pub struct SmsDeliveryService {
    context_store: UeSmsContextStore,
    db: Database,
    amf_client: AmfClient,
    default_validity_secs: u64,
}

impl SmsDeliveryService {
    pub fn new(context_store: UeSmsContextStore, db: Database, amf_client: AmfClient, default_validity_secs: u64) -> Self {
        Self {
            context_store,
            db,
            amf_client,
            default_validity_secs,
        }
    }

    pub async fn deliver_mt_sms(&self, supi: &str, sms_data: SmsDeliveryData) -> Result<String> {
        let context = self
            .context_store
            .get(supi)
            .ok_or_else(|| anyhow::anyhow!("UE SMS context not found"))?;

        let sms_record_id = sms_data.sms_record_id.clone();
        let now = Utc::now();
        let expires_at = now + Duration::seconds(self.default_validity_secs as i64);

        let mut amf_uri = context.amf_id.clone();

        if amf_uri.is_empty() {
            info!("No AMF ID in context, attempting AMF discovery for SUPI: {}", supi);
            match self.amf_client.discover_amf(context.guami.as_ref(), context.ue_location.as_ref()).await {
                Ok(discovered_uri) => {
                    info!("Discovered AMF via NRF: {}", discovered_uri);
                    amf_uri = discovered_uri;
                }
                Err(e) => {
                    warn!("Failed to discover AMF via NRF: {}", e);
                    return Err(anyhow::anyhow!("No AMF available for delivery"));
                }
            }
        }

        let sms_record = SmsRecord {
            sms_record_id: sms_record_id.clone(),
            sms_payload: sms_data.sms_msg.clone(),
            delivery_status: SmsDeliveryStatus::Pending,
            gpsi: context.gpsi.clone(),
            supi: supi.to_string(),
            amf_id: amf_uri.clone(),
            retry_count: 0,
            next_retry_at: None,
            expires_at,
            created_at: now,
            updated_at: now,
            status_report_requested: false,
            originator_address: None,
            destination_address: context.gpsi.clone(),
            message_reference: None,
            is_mobile_originated: false,
            failure_reason: None,
            is_international: None,
            route_type: None,
        };

        self.db.save_sms_record(&sms_record).await?;

        match self.attempt_delivery(&sms_record).await {
            Ok(_) => {
                info!("MT SMS delivered to AMF for SUPI: {}", supi);
                self.update_delivery_status(&sms_record_id, SmsDeliveryStatus::AcceptedByNetwork).await?;
                Ok(sms_record_id)
            }
            Err(e) => {
                warn!("Failed to deliver MT SMS to AMF, will retry: {}", e);
                let status = self.classify_delivery_error(&e);
                self.update_delivery_status_with_reason(&sms_record_id, status, Some(e.to_string())).await?;
                Ok(sms_record_id)
            }
        }
    }

    pub async fn attempt_delivery(&self, sms_record: &SmsRecord) -> Result<()> {
        match self.amf_client.check_ue_reachability(&sms_record.supi, &sms_record.amf_id).await {
            Ok(true) => {
                info!("UE is reachable, proceeding with SMS delivery");
            }
            Ok(false) => {
                warn!("UE is not reachable (CM-IDLE state), delivery will be queued");
                return Err(anyhow::anyhow!("UE not reachable"));
            }
            Err(e) => {
                warn!("Failed to check UE reachability: {}, continuing with delivery attempt", e);
            }
        }

        self.amf_client
            .send_n1n2_message(&sms_record.supi, &sms_record.amf_id, sms_record.sms_payload.clone())
            .await
    }

    fn classify_delivery_error(&self, error: &anyhow::Error) -> SmsDeliveryStatus {
        let error_str = error.to_string().to_lowercase();

        if error_str.contains("not reachable") || error_str.contains("cm-idle") {
            SmsDeliveryStatus::UeNotReachable
        } else if error_str.contains("memory") || error_str.contains("capacity") {
            SmsDeliveryStatus::MemoryCapacityExceeded
        } else if error_str.contains("network") || error_str.contains("connection") || error_str.contains("timeout") {
            SmsDeliveryStatus::NetworkFailure
        } else {
            SmsDeliveryStatus::Failed
        }
    }

    async fn update_delivery_status(&self, sms_record_id: &str, status: SmsDeliveryStatus) -> Result<()> {
        self.db.update_sms_status(sms_record_id, status).await
    }

    async fn update_delivery_status_with_reason(&self, sms_record_id: &str, status: SmsDeliveryStatus, reason: Option<String>) -> Result<()> {
        self.db.update_sms_status_with_reason(sms_record_id, status, reason).await
    }
}
