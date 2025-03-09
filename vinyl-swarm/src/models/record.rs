use chrono::{NaiveDate, NaiveTime};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordModel {
    pub record_id: Uuid,
    pub artist: String,
    pub title: String,
    pub released: NaiveDate,
    pub genre: Vec<String>,
    pub format: String,
    pub price: f64,
    pub label: String,
    pub duration_length: NaiveTime,
}