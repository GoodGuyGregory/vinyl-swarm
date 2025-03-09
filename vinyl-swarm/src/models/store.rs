use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordStore {
    pub record_store_id: Uuid,
    pub store_name: String,
    pub store_address: String,
    pub store_city: String,
    pub store_state: String,
    pub store_zip: String,
    pub phone_number: Option<String>,
    pub website: Option<String>,
}