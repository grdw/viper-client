mod channel;
mod ctpp_channel;
mod helper;
mod stream_wrapper;
pub mod device;
pub mod command;
pub mod command_response;

#[cfg(test)]
mod test_helper;

use serde::Deserialize;
use stream_wrapper::StreamWrapper;
use channel::Channel;
use command::CommandKind;
use command_response::{
    ActivateUserResponse,
    AuthResponse,
    ConfigurationResponse,
    InfoResponse,
    VipResponse
};
use ctpp_channel::CTPPChannel;
use helper::Helper;
use std::{io, fmt, fmt::Display, str};

type JSONResult<T> = Result<T, ViperError>;

pub struct ViperClient {
    stream: StreamWrapper,
    control: [u8; 2]
}

#[derive(Debug)]
pub enum ViperError {
    IOError(io::Error),
    JSONError(serde_json::Error)
}

impl Display for ViperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ViperError::IOError(io_error) =>
                write!(f, "{}", io_error),
            ViperError::JSONError(json_error) =>
                write!(f, "{}", json_error),
        }
    }
}

impl From<io::Error> for ViperError {
    fn from(error: io::Error) -> Self {
        ViperError::IOError(error)
    }
}

impl ViperClient {
    pub fn new(ip: &String, port: &String) -> ViperClient {
        let doorbell = format!("{}:{}", ip, port);

        ViperClient {
            stream: StreamWrapper::new(doorbell),
            control: Helper::control()
        }
    }

    pub fn sign_up(&mut self, email: &String) -> JSONResult<ActivateUserResponse> {
        let fact_channel = self.channel("FACT");
        self.stream.execute(&fact_channel.open())?;
        let activate_user = CommandKind::ActivateUser(String::from(email));
        let act_bytes = self.stream.execute(&fact_channel.com(activate_user))?;
        let json_response = Self::json(&act_bytes);

        self.stream.execute(&fact_channel.close())?;
        json_response
    }

    pub fn remove_all_users(&mut self, email: &String) -> JSONResult<serde_json::Value> {
        let fact_channel = self.channel("FACT");
        self.stream.execute(&fact_channel.open())?;
        let remove_all_users = CommandKind::RemoveAllUsers(String::from(email));
        let rem_bytes = self.stream.execute(&fact_channel.com(remove_all_users))?;
        self.stream.execute(&fact_channel.close())?;

        let json_response = Self::json(&rem_bytes);
        json_response
    }

    pub fn authorize(&mut self, token: String) -> JSONResult<AuthResponse> {
        let uaut = CommandKind::UAUT(token);
        let uaut_channel = self.channel("UAUT");
        self.stream.execute(&uaut_channel.open())?;
        let uaut_bytes = self.stream.execute(&uaut_channel.com(uaut))?;

        let json_response = Self::json(&uaut_bytes);
        self.stream.execute(&uaut_channel.close())?;
        json_response
    }

    pub fn configuration(&mut self, addressbooks: String) -> JSONResult<ConfigurationResponse> {
        let ucfg = CommandKind::UCFG(addressbooks);
        let ucfg_channel = self.channel("UCFG");
        self.stream.execute(&ucfg_channel.open())?;
        let ucfg_bytes = self.stream.execute(&ucfg_channel.com(ucfg))?;

        let json_response = Self::json(&ucfg_bytes);
        self.stream.execute(&ucfg_channel.close())?;
        json_response
    }

    pub fn info(&mut self) -> JSONResult<InfoResponse> {
        let info = CommandKind::INFO;
        let info_channel = self.channel("INFO");
        self.stream.execute(&info_channel.open())?;

        let info_bytes = self.stream.execute(&info_channel.com(info))?;
        let json_response = Self::json(&info_bytes);
        self.stream.execute(&info_channel.close())?;
        json_response
    }

    pub fn face_recognition_params(&mut self) -> JSONResult<serde_json::Value> {
        let frcg = CommandKind::FRCG;
        let frcg_channel = self.channel("FRCG");
        self.stream.execute(&frcg_channel.open())?;

        let frcg_bytes = self.stream.execute(&frcg_channel.com(frcg))?;
        let json_response = Self::json(&frcg_bytes);
        self.stream.execute(&frcg_channel.close())?;
        json_response
    }

    // TODO: This function is not finished
    pub fn open_door(&mut self, vip: &VipResponse) -> Result<(), std::io::Error> {
        let addr = vip.apt_address.to_string();
        let sub = format!("{}{}", addr, vip.apt_subaddress);
        let act = vip.user_parameters.opendoor_address_book[0].apt_address.to_string();

        let mut ctpp_channel = self.ctpp_channel();
        self.stream.execute(&ctpp_channel.open(&sub))?;
        self.stream.write(&ctpp_channel.connect_hs(&sub, &addr))?;

        loop {
            let resp = self.stream.read()?;
            println!("{:02x?}", resp);
            if ctpp_channel.confirm_handshake(&resp) {
                break;
            }
        }

        self.stream.write(&ctpp_channel.ack(0x00, &sub, &addr))?;
        self.stream.write(&ctpp_channel.ack(0x20, &sub, &addr))?;
        self.stream.write(&ctpp_channel.link_actuators(&act, &sub))?;

        let resp = self.stream.read()?;
        if ctpp_channel.confirm(&resp) {
            // ????
        } else {
            // raise an error
        }

        // Close the remaining channels
        self.stream.execute(&ctpp_channel.close())?;
        Ok(())
    }

    fn channel(&mut self, command: &'static str) -> Channel {
        self.tick();

        Channel::new(&self.control, command)
    }

    fn ctpp_channel(&mut self) -> CTPPChannel {
        self.tick();

        CTPPChannel::new(&self.control)
    }

    fn json<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> JSONResult<T> {
        match serde_json::from_slice(bytes) {
            Ok(json) => Ok(json),
            Err(e) => Err(ViperError::JSONError(e))
        }
    }

    pub fn shutdown(&mut self) {
        self.stream.die();
    }

    // Move the control byte 1 ahead
    fn tick(&mut self) {
        self.control[0] += 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use crate::command::Command;
    use crate::test_helper::SimpleTcpListener;

    #[test]
    fn test_tick() {
        let _listener = SimpleTcpListener::new("127.0.0.1:3340");
        let mut client = ViperClient::new(
            &String::from("127.0.0.1"),
            &String::from("3340")
        );

        let c = client.control;
        client.tick();

        assert_eq!(c[0] + 1, client.control[0])
    }

    #[test]
    fn test_authorize() {
        let listener = SimpleTcpListener::new("127.0.0.1:3341");
        let mut client = ViperClient::new(
            &String::from("127.0.0.1"),
            &String::from("3341")
        );

        thread::spawn(move || {
            let mocked_open = [
                0xcd, 0xab, 0x02, 0x00, 0x04, 0x00, 0x00, 0x00,
                0x1a, 0x12, 0x00, 0x00
            ];

            let mocked_json = r#"{
                "message":"access",
                "message-type":"response",
                "message-id":5,
                "response-code":200,
                "response-string":"Access Granted"
            }"#;

            listener.mock_server(
                vec![
                    Command::make(&mocked_open, &[0, 0]),
                    Command::make(&mocked_json.as_bytes(), &[0, 0]),
                    Command::make(&[], &[0, 0]) // Closing the channel
                ]
            )
        });

        let resp = client.authorize(String::from("TESTTOKEN")).unwrap();
        assert_eq!(resp.response.response_string, "Access Granted");
        assert_eq!(resp.response.response_code, 200)
    }
}
