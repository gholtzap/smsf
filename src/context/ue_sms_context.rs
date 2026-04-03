use crate::sbi::models::{Guami, UeSmsContextData, UserLocation};
use crate::sms::types::AccessType;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UeSmsContext {
    pub supi: String,
    pub amf_id: String,
    pub access_type: AccessType,
    pub guami: Option<Guami>,
    pub ue_location: Option<UserLocation>,
    pub gpsi: Option<String>,
    pub ue_time_zone: Option<String>,
    pub etag: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UeSmsContext {
    pub fn from_data(data: UeSmsContextData) -> Self {
        let now = Utc::now();
        let etag = uuid::Uuid::new_v4().to_string();
        Self {
            supi: data.supi,
            amf_id: data.amf_id,
            access_type: data.access_type,
            guami: data.guami,
            ue_location: data.ue_location,
            gpsi: data.gpsi,
            ue_time_zone: data.ue_time_zone,
            etag,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_from_data(&mut self, data: UeSmsContextData) {
        self.amf_id = data.amf_id;
        self.access_type = data.access_type;
        self.guami = data.guami;
        self.ue_location = data.ue_location;
        self.gpsi = data.gpsi;
        self.ue_time_zone = data.ue_time_zone;
        self.etag = uuid::Uuid::new_v4().to_string();
        self.updated_at = Utc::now();
    }

    pub fn to_data(&self) -> UeSmsContextData {
        UeSmsContextData {
            supi: self.supi.clone(),
            amf_id: self.amf_id.clone(),
            access_type: self.access_type.clone(),
            guami: self.guami.clone(),
            ue_location: self.ue_location.clone(),
            gpsi: self.gpsi.clone(),
            ue_time_zone: self.ue_time_zone.clone(),
        }
    }
}

#[derive(Clone)]
pub struct UeSmsContextStore {
    store: Arc<DashMap<String, UeSmsContext>>,
}

impl UeSmsContextStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(DashMap::new()),
        }
    }

    pub fn insert(&self, supi: String, context: UeSmsContext) {
        self.store.insert(supi, context);
    }

    pub fn get(&self, supi: &str) -> Option<UeSmsContext> {
        self.store.get(supi).map(|entry| entry.value().clone())
    }

    pub fn remove(&self, supi: &str) -> Option<UeSmsContext> {
        self.store.remove(supi).map(|(_, context)| context)
    }

    pub fn update<F>(&self, supi: &str, f: F) -> Option<UeSmsContext>
    where
        F: FnOnce(&mut UeSmsContext),
    {
        self.store.get_mut(supi).map(|mut entry| {
            f(entry.value_mut());
            entry.value().clone()
        })
    }

    pub fn try_update<F, E>(&self, supi: &str, f: F) -> Option<Result<UeSmsContext, E>>
    where
        F: FnOnce(&mut UeSmsContext) -> Result<(), E>,
    {
        self.store.get_mut(supi).map(|mut entry| {
            match f(entry.value_mut()) {
                Ok(()) => Ok(entry.value().clone()),
                Err(e) => Err(e),
            }
        })
    }

    pub fn contains(&self, supi: &str) -> bool {
        self.store.contains_key(supi)
    }

    pub fn load_contexts(&self, contexts: Vec<UeSmsContext>) {
        for context in contexts {
            let supi = context.supi.clone();
            self.store.insert(supi, context);
        }
    }

    pub fn count(&self) -> usize {
        self.store.len()
    }
}
