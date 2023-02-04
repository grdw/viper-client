mod channel;
mod ctpp_channel;
mod helper;
mod stream_wrapper;
pub mod device;
pub mod command;

use stream_wrapper::StreamWrapper;
use channel::Channel;
use command::CommandKind;
use ctpp_channel::CTPPChannel;
use helper::Helper;
use std::{io, fmt, fmt::Display, str};

type JSONResult = Result<serde_json::Value, ViperError>;

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

    pub fn authorize(&mut self, token: String) -> JSONResult {
        let uaut = CommandKind::UAUT(token);
        let uaut_channel = self.channel("UAUT");
        self.stream.execute(&uaut_channel.open())?;
        let uaut_bytes = self.stream.execute(&uaut_channel.com(uaut))?;

        let json_response = Self::json(&uaut_bytes);
        self.stream.execute(&uaut_channel.close())?;
        json_response
    }

    pub fn configuration(&mut self, addressbooks: String) -> JSONResult {
        let ucfg = CommandKind::UCFG(addressbooks);
        let ucfg_channel = self.channel("UCFG");
        self.stream.execute(&ucfg_channel.open())?;
        let ucfg_bytes = self.stream.execute(&ucfg_channel.com(ucfg))?;

        let json_response = Self::json(&ucfg_bytes);
        self.stream.execute(&ucfg_channel.close())?;
        json_response
    }

    pub fn info(&mut self) -> JSONResult {
        let info = CommandKind::INFO;
        let info_channel = self.channel("INFO");
        self.stream.execute(&info_channel.open())?;

        let info_bytes = self.stream.execute(&info_channel.com(info))?;
        let json_response = Self::json(&info_bytes);
        self.stream.execute(&info_channel.close())?;

        json_response
    }

    pub fn face_recognition_params(&mut self) -> JSONResult {
        let frcg = CommandKind::FRCG;
        let frcg_channel = self.channel("FRCG");
        self.stream.execute(&frcg_channel.open())?;

        let frcg_bytes = self.stream.execute(&frcg_channel.com(frcg))?;
        let json_response = Self::json(&frcg_bytes);
        self.stream.execute(&frcg_channel.close())?;
        json_response
    }

    // TODO: This function is not finished
    pub fn open_door(&mut self, vip: &serde_json::Value) -> Result<(), std::io::Error> {
        let addr = vip["apt-address"].to_string();
        let sub = format!("{}{}", addr, vip["apt-subaddress"]);

        let act = vip["user-parameters"]
                     ["opendoor-address-book"]
                     [0]
                     ["apt-address"].to_string();

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
        if ctpp_channel.confirm(resp) {
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

    fn json(bytes: &[u8]) -> JSONResult {
        let json_str =  str::from_utf8(&bytes).unwrap();

        match serde_json::from_str(json_str) {
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
    use std::net::TcpListener;

    #[test]
    fn test_tick() {
        let _listener = TcpListener::bind("127.0.0.1:3340").unwrap();
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
        // TODO: Find a way to write proper TCPListener tests
        assert_eq!(true, true)
    }
}
