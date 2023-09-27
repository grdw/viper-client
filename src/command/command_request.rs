use super::base::Base;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Authorize {
    #[serde(flatten)]
    base: Base,
    pub user_token: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    #[serde(flatten)]
    base: Base,
    pub addressbooks: String
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoveAllUsers {
    #[serde(flatten)]
    base: Base,
    requester: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ActivateUser {
    #[serde(flatten)]
    base: Base,
    email: String,
    description: String,
}

pub struct CommandRequest {}

impl CommandRequest {
    pub fn to_json<T: Serialize>(cmd: T) -> String {
        return serde_json::to_string(&cmd).unwrap()
    }

    pub fn authorize(user_token: String) -> Authorize {
        return Authorize {
            base: Base::request("access"),
            user_token
        }
    }

    pub fn configuration(addressbooks: String) -> Configuration {
        return Configuration {
            base: Base::request("get-configuration"),
            addressbooks
        }
    }

    pub fn remove_all_users(requester: String) -> RemoveAllUsers {
        return RemoveAllUsers {
            base: Base::request("remove-all-users"),
            requester,
        }
    }

    pub fn activate_user(email: String) -> ActivateUser {
        return ActivateUser {
            base: Base::request("activate-user"),
            description: String::from("viper-client"),
            email,
        }
    }

    pub fn default(message: &'static str) -> Base {
        return Base::request(message)
    }
}
