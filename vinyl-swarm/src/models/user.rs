use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize)]
pub struct UserModel {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_first_name: String,
    pub user_email: String,
    pub user_password: String,
    pub created_at: NaiveDateTime,
}
