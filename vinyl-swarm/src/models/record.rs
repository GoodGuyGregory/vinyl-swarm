use chrono::{NaiveDate, NaiveTime};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use sqlx::types::BigDecimal;

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRecordSchema {
    pub artist: String,
    pub title: String,
    pub released: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<BigDecimal>,
    pub label: String,
    pub duration_length: NaiveTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateRecordSchema {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub released: Option<NaiveDate>,
    pub genre: Option<Vec<String>>,
    pub format: Option<String>,
    pub price: Option<BigDecimal>,
    pub label: Option<String>,
    pub duration_length: Option<NaiveTime>,
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