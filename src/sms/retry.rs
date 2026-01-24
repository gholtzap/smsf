use super::delivery::SmsDeliveryService;
use super::types::SmsDeliveryStatus;
use crate::config::RetryConfig;
use crate::db::Database;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tokio::time;
use tracing::{debug, error, info, warn};

pub struct SmsRetryService {
    db: Database,
    delivery_service: Arc<SmsDeliveryService>,
    config: RetryConfig,
}

impl SmsRetryService {
    pub fn new(db: Database, delivery_service: Arc<SmsDeliveryService>, config: RetryConfig) -> Self {
        Self {
            db,
            delivery_service,
            config,
        }
    }

    pub async fn start(self: Arc<Self>) {
        info!("Starting SMS retry service with interval: {}s", self.config.retry_interval_secs);

        let mut interval = time::interval(time::Duration::from_secs(self.config.retry_interval_secs));

        loop {
            interval.tick().await;

            if let Err(e) = self.process_retries().await {
                error!("Error processing retries: {}", e);
            }
        }
    }

    async fn process_retries(&self) -> anyhow::Result<()> {
        let pending_messages = self.db.get_pending_retries().await?;

        if !pending_messages.is_empty() {
            debug!("Processing {} pending SMS messages for retry", pending_messages.len());
        }

        for sms_record in pending_messages {
            if Utc::now() > sms_record.expires_at {
                warn!("SMS {} has expired, marking as failed", sms_record.sms_record_id);
                self.db.mark_expired(&sms_record.sms_record_id).await?;
                continue;
            }

            if sms_record.retry_count >= self.config.max_attempts {
                warn!(
                    "SMS {} has reached max retry attempts ({}), marking as failed",
                    sms_record.sms_record_id, self.config.max_attempts
                );
                self.db.update_sms_status(&sms_record.sms_record_id, SmsDeliveryStatus::Failed).await?;
                continue;
            }

            match self.delivery_service.attempt_delivery(&sms_record).await {
                Ok(_) => {
                    info!(
                        "SMS {} successfully delivered on retry attempt {}",
                        sms_record.sms_record_id, sms_record.retry_count + 1
                    );
                    self.db.update_sms_status(&sms_record.sms_record_id, SmsDeliveryStatus::Accepted).await?;
                }
                Err(e) => {
                    let backoff = self.calculate_backoff(sms_record.retry_count);
                    let next_retry_at = Utc::now() + Duration::seconds(backoff as i64);

                    warn!(
                        "SMS {} delivery failed (attempt {}), will retry in {}s: {}",
                        sms_record.sms_record_id,
                        sms_record.retry_count + 1,
                        backoff,
                        e
                    );

                    self.db.increment_retry_count(&sms_record.sms_record_id, Some(next_retry_at)).await?;
                    self.db.update_sms_status(&sms_record.sms_record_id, SmsDeliveryStatus::Pending).await?;
                }
            }
        }

        Ok(())
    }

    fn calculate_backoff(&self, retry_count: u32) -> u64 {
        let backoff = self.config.initial_backoff_secs * 2u64.pow(retry_count);
        backoff.min(self.config.max_backoff_secs)
    }
}
