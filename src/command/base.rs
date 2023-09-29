use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Base {
    pub message: String,
    pub message_type: String,
    pub message_id: u8
}

impl Base {
    pub fn response(message: &'static str) -> Base {
        return Self::new(message, "response")
    }

    pub fn request(message: &'static str) -> Base {
        return Self::new(message, "request")
    }

    fn new(message: &'static str, t: &'static str) -> Base {
        return Base {
            message: String::from(message),
            message_type: String::from(t),
            message_id: 1
        }
    }
}
