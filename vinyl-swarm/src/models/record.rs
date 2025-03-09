use chrono::{NaiveDate, NaiveTime};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use sqlx::types::BigDecimal;

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordModel {
    pub record_id: Uuid,
    pub artist: String,
    pub title: String,
    pub released: NaiveDate,
    pub genre: Option<Vec<String>>,
    pub format: Option<String>,
    pub price: Option<BigDecimal>,
    pub label: String,
    pub duration_length: NaiveTime,
}