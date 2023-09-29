use std::collections::HashMap;
use serde::Deserialize;
use serde_json::Value;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ActivateUserResponse {
    pub user_token: String,

     #[serde(flatten)]
    pub response: BaseResponse
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ViperClientResponse {
    pub description: String
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AptConfigResponse {
    pub description: String,
    pub call_divert_busy_en: bool,
    pub call_divert_address: String,
    pub virtual_key_enabled: bool
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct VipResponse {
    pub enabled: bool,
    pub apt_address: String,
    pub apt_subaddress: u16,
    pub logical_subaddress: u16,
    pub apt_config: AptConfigResponse
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigurationResponse {
    pub viper_server: ViperServerResponse,
    pub viper_client: ViperClientResponse,
    pub vip: VipResponse,

    #[serde(flatten)]
    pub response: BaseResponse
}
