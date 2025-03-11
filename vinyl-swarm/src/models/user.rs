use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;


/// for pagination in a front end UI
#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct PutUserRecord {
    pub record_id: Uuid
}


// due to security concerns
#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponseSchema {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_first_name: String,
    pub user_last_name: String,
    pub user_email: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserSchema {
    pub user_name: String,
    pub user_first_name: String,
    pub user_last_name: String,
    pub user_email: String,
    pub user_password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserSchema {
    pub user_name: Option<String>,
    pub user_first_name: Option<String>,
    pub user_last_name: Option<String>,
    pub user_email: Option<String>,
    pub user_password: Option<String>,
}

// helps for converting to the appropriate exposed 
// API GET Endpoints leveraging the From trait 
impl From<UserModel> for UserResponseSchema {
    fn from(user: UserModel) -> Self {
        UserResponseSchema {
            user_id: user.user_id,
            user_name: user.user_name,
            user_first_name: user.user_first_name,
            user_last_name: user.user_last_name,
            user_email: user.user_email,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserModel {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_first_name: String,
    pub user_last_name: String,
    pub user_email: String,
    pub user_password: String,
    pub created_at: Option<DateTime<Utc>>,
}