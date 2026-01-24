use super::types::{SmsDeliveryData, SmsDeliveryStatus, SmsRecord};
use crate::context::ue_sms_context::UeSmsContextStore;
use crate::db::Database;
use crate::nf_client::amf::AmfClient;
use anyhow::Result;
use chrono::Utc;
use tracing::{error, info};

pub struct SmsDeliveryService {
    context_store: UeSmsContextStore,
    db: Database,
    amf_client: AmfClient,
}

impl SmsDeliveryService {
    pub fn new(context_store: UeSmsContextStore, db: Database, amf_client: AmfClient) -> Self {
        Self {
            context_store,
            db,
            amf_client,
        }
    }

    pub async fn deliver_mt_sms(&self, supi: &str, sms_data: SmsDeliveryData) -> Result<String> {
        let context = self
            .context_store
            .get(supi)
            .ok_or_else(|| anyhow::anyhow!("UE SMS context not found"))?;

        let sms_record_id = sms_data.sms_record_id.clone();
        let sms_record = SmsRecord {
            sms_record_id: sms_record_id.clone(),
            sms_payload: sms_data.sms_msg.clone(),
            delivery_status: SmsDeliveryStatus::Pending,
            gpsi: context.gpsi.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.db.save_sms_record(&sms_record).await?;

        match self.amf_client.send_n1n2_message(supi, &context.amf_id, sms_data.sms_msg).await {
            Ok(_) => {
                info!("MT SMS delivered to AMF for SUPI: {}", supi);
                self.update_delivery_status(&sms_record_id, SmsDeliveryStatus::Accepted).await?;
                Ok(sms_record_id)
            }
            Err(e) => {
                error!("Failed to deliver MT SMS to AMF: {}", e);
                self.update_delivery_status(&sms_record_id, SmsDeliveryStatus::Failed).await?;
                Err(e)
            }
        }
    }

    async fn update_delivery_status(&self, sms_record_id: &str, status: SmsDeliveryStatus) -> Result<()> {
        self.db.update_sms_status(sms_record_id, status).await
    }
}
