use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct BaseResponse {
    pub message: String,
    pub message_type: String,
    pub message_id: u8,
    pub response_code: u8,
    pub response_string: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AuthResponse {
    #[serde(flatten)]
    pub response: BaseResponse
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct InfoResponse {
    pub model: String,
    pub version: String,
    pub serial_code: String,
    pub capabilities: Vec<String>,

    #[serde(flatten)]
    pub channel_details: HashMap<String, Value>,

    #[serde(flatten)]
    pub response: BaseResponse
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ActivateUserResponse {
    pub user_token: String,

     #[serde(flatten)]
    pub response: BaseResponse
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ViperServerResponse {
    pub local_address: String,
    pub local_tcp_port: u16,
    pub local_udp_port: u16,
    pub remote_address: String,
    pub remote_tcp_port: u16,
    pub remote_udp_port: u16
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ViperClientResponse {
    pub description: String
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AptConfigResponse {
    pub description: String,
    pub call_divert_busy_en: bool,
    pub call_divert_address: String,
    pub virtual_key_enabled: bool
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Switchboard {
    pub id: String,
    pub name: String,
    pub apt_address: String,
    pub emergency_calls: bool
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Entrance {
    pub id: String,
    pub name: String,
    pub apt_address: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Actuator {
    pub id: String,
    pub name: String,
    pub apt_address: String,
    pub module_index: u8,
    pub output_index: u8
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Opendoor {
    pub id: String,
    pub name: String,
    pub apt_address: String,
    pub output_index: u8,
    pub secure_mode: bool
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct OpendoorAction {
    pub id: String,
    pub action: String,
    pub apt_address: String,
    pub output_index: u8
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct UserParametersResponse {
    pub forced: bool,
    pub apt_address_book: Vec<HashMap<String, Value>>,
    pub camera_address_book: Vec<HashMap<String, Value>>,
    pub rtsp_camera_address_book: Vec<HashMap<String, Value>>,
    pub switchboard_address_book: Vec<Switchboard>,
    pub entrance_address_book: Vec<Entrance>,
    pub actuator_address_book: Vec<Actuator>,
    pub opendoor_address_book: Vec<Opendoor>,
    pub opendoor_actions: Vec<OpendoorAction>,
    pub additional_actuator: Vec<Actuator>,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct VipResponse {
    pub enabled: bool,
    pub apt_address: String,
    pub apt_subaddress: u16,
    pub logical_subaddress: u16,
    pub apt_config: AptConfigResponse,
    pub user_parameters: UserParametersResponse
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigurationResponse {
    pub viper_server: ViperServerResponse,
    pub viper_client: ViperClientResponse,
    pub vip: VipResponse,

    #[serde(flatten)]
    pub response: BaseResponse
}
