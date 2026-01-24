use crate::context::ue_sms_context::UeSmsContext;
use crate::sms::types::{SmsDeliveryStatus, SmsRecord};
use anyhow::Result;
use mongodb::bson::doc;
use mongodb::{Client, Collection};
use tracing::info;

#[derive(Clone)]
pub struct Database {
    ue_contexts: Collection<UeSmsContext>,
    sms_records: Collection<SmsRecord>,
}

impl Database {
    pub async fn new(uri: &str) -> Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database("smsf");

        let ue_contexts = db.collection::<UeSmsContext>("ue_sms_contexts");
        let sms_records = db.collection::<SmsRecord>("sms_records");

        info!("Connected to MongoDB");
        Ok(Self {
            ue_contexts,
            sms_records,
        })
    }

    pub async fn save_ue_context(&self, context: &UeSmsContext) -> Result<()> {
        self.ue_contexts
            .insert_one(context)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to save UE context: {}", e))?;
        Ok(())
    }

    pub async fn update_ue_context(&self, context: &UeSmsContext) -> Result<()> {
        self.ue_contexts
            .replace_one(doc! { "supi": &context.supi }, context)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update UE context: {}", e))?;
        Ok(())
    }

    pub async fn delete_ue_context(&self, supi: &str) -> Result<()> {
        self.ue_contexts
            .delete_one(doc! { "supi": supi })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete UE context: {}", e))?;
        Ok(())
    }

    pub async fn get_ue_context(&self, supi: &str) -> Result<Option<UeSmsContext>> {
        self.ue_contexts
            .find_one(doc! { "supi": supi })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get UE context: {}", e))
    }

    pub async fn save_sms_record(&self, record: &SmsRecord) -> Result<()> {
        self.sms_records
            .insert_one(record)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to save SMS record: {}", e))?;
        Ok(())
    }

    pub async fn update_sms_status(&self, sms_record_id: &str, status: SmsDeliveryStatus) -> Result<()> {
        self.sms_records
            .update_one(
                doc! { "sms_record_id": sms_record_id },
                doc! { "$set": {
                    "delivery_status": mongodb::bson::to_bson(&status)?,
                    "updated_at": mongodb::bson::DateTime::now()
                }},
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update SMS status: {}", e))?;
        Ok(())
    }

    pub async fn get_sms_record(&self, sms_record_id: &str) -> Result<Option<SmsRecord>> {
        self.sms_records
            .find_one(doc! { "sms_record_id": sms_record_id })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get SMS record: {}", e))
    }

    pub async fn get_pending_retries(&self) -> Result<Vec<SmsRecord>> {
        use mongodb::bson::DateTime as BsonDateTime;
        use futures_util::TryStreamExt;

        let now = BsonDateTime::now();
        let cursor = self.sms_records
            .find(doc! {
                "$or": [
                    { "delivery_status": "PENDING" },
                    { "delivery_status": "FAILED" }
                ],
                "expires_at": { "$gt": now },
                "$or": [
                    { "next_retry_at": { "$lte": now } },
                    { "next_retry_at": null }
                ]
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query pending retries: {}", e))?;

        cursor.try_collect::<Vec<SmsRecord>>().await
            .map_err(|e| anyhow::anyhow!("Failed to collect pending retries: {}", e))
    }

    pub async fn increment_retry_count(&self, sms_record_id: &str, next_retry_at: Option<chrono::DateTime<chrono::Utc>>) -> Result<()> {
        let next_retry_bson = next_retry_at.map(|dt| {
            mongodb::bson::DateTime::from_millis(dt.timestamp_millis())
        });

        self.sms_records
            .update_one(
                doc! { "sms_record_id": sms_record_id },
                doc! {
                    "$inc": { "retry_count": 1 },
                    "$set": {
                        "next_retry_at": next_retry_bson,
                        "updated_at": mongodb::bson::DateTime::now()
                    }
                },
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to increment retry count: {}", e))?;
        Ok(())
    }

    pub async fn mark_expired(&self, sms_record_id: &str) -> Result<()> {
        self.sms_records
            .update_one(
                doc! { "sms_record_id": sms_record_id },
                doc! { "$set": {
                    "delivery_status": "FAILED",
                    "updated_at": mongodb::bson::DateTime::now()
                }},
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to mark as expired: {}", e))?;
        Ok(())
    }
}
