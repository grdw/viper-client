use std::collections::HashMap;
use serde::Deserialize;
use serde_json::Value;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct BaseResponse {
    message: String,
    message_type: String,
    message_id: u8,
    response_code: u8,
    response_string: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct InfoResponse {
    model: String,
    version: String,
    serial_code: String,
    capabilities: Vec<String>,

    #[serde(flatten)]
    channel_details: HashMap<String, Value>,

    #[serde(flatten)]
    base: BaseResponse
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ActivateUserResponse {
    user_token: String,

     #[serde(flatten)]
    base: BaseResponse
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ViperServerResponse {
    local_address: String,
    local_tcp_port: u16,
    local_udp_port: u16,
    remote_address: String,
    remote_tcp_port: u16,
    remote_udp_port: u16
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ViperClientResponse {
    description: String
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AptConfigResponse {
    description: String,
    call_divert_busy_en: bool,
    call_divert_address: String,
    virtual_key_enabled: bool
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct VipResponse {
    enabled: bool,
    apt_address: String,
    apt_subaddress: u16,
    logical_subaddress: u16,
    apt_config: AptConfigResponse
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigurationResponse {
    #[serde(flatten)]
    base: BaseResponse,
    viper_server: ViperServerResponse,
    viper_client: ViperClientResponse,
    vip: VipResponse,
}
