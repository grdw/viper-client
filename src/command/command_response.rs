use super::base::Base;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AuthorizeResponse {
    #[serde(flatten)]
    base: Base,
    pub response_code: u8,
    pub response_string: String
}

pub struct CommandResponse {}

impl CommandResponse {
    pub fn authorize(
        response_code: u8, response_string: &'static str) -> AuthorizeResponse {

        return AuthorizeResponse {
            base: Base::response("access"),
            response_string: String::from(response_string),
            response_code
        }
    }
}
