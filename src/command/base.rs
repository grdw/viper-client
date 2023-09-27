use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Base {
    pub message: String,
    pub message_type: String,
    pub message_id: u8
}

