use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// for pagination in a front end UI
#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRecordStoreSchema {
    pub store_name: String,
    pub store_address: String,
    pub store_city: String,
    pub store_state: String,
    pub store_zip: String,
    pub phone_number: Option<String>,
    pub website: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PutRecordStoreSchema {
    pub record_store_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PatchRecordStoreSchema {
    pub record_store_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateRecordStoreSchema {
    pub store_name: Option<String>,
    pub store_address: Option<String>,
    pub store_city: Option<String>,
    pub store_state: Option<String>,
    pub store_zip: Option<String>,
    pub phone_number: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordStoreModel {
    pub record_store_id: Uuid,
    pub store_name: String,
    pub store_address: String,
    pub store_city: String,
    pub store_state: String,
    pub store_zip: String,
    pub phone_number: Option<String>,
    pub website: Option<String>,
}
