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

        let sms_record = SmsRecord {
            sms_record_id: sms_record_id.clone(),
            sms_payload: sms_data.sms_msg.clone(),
            delivery_status: SmsDeliveryStatus::Pending,
            gpsi: context.gpsi.clone(),
            supi: supi.to_string(),
            amf_id: context.amf_id.clone(),
            retry_count: 0,
            next_retry_at: None,
            expires_at,
            created_at: now,
            updated_at: now,
        };

        self.db.save_sms_record(&sms_record).await?;

        match self.attempt_delivery(&sms_record).await {
            Ok(_) => {
                info!("MT SMS delivered to AMF for SUPI: {}", supi);
                self.update_delivery_status(&sms_record_id, SmsDeliveryStatus::Accepted).await?;
                Ok(sms_record_id)
            }
            Err(e) => {
                warn!("Failed to deliver MT SMS to AMF, will retry: {}", e);
                Ok(sms_record_id)
            }
        }
    }

    pub async fn attempt_delivery(&self, sms_record: &SmsRecord) -> Result<()> {
        self.amf_client
            .send_n1n2_message(&sms_record.supi, &sms_record.amf_id, sms_record.sms_payload.clone())
            .await
    }

    async fn update_delivery_status(&self, sms_record_id: &str, status: SmsDeliveryStatus) -> Result<()> {
        self.db.update_sms_status(sms_record_id, status).await
    }
}
