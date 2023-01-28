mod channel;
mod ctpp_channel;
mod helper;
pub mod command;

use channel::Channel;
use command::{Command, CommandKind};
use ctpp_channel::CTPPChannel;
use helper::Helper;
use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;
use std::{io, str};

const TIMEOUT: u64 = 5000;

pub struct ViperClient {
    pub stream: TcpStream,
    control: [u8; 2]
}

type JSONResult = Result<serde_json::Value, serde_json::Error>;
type ByteResult = Result<Vec<u8>, io::Error>;

impl ViperClient {
    pub fn new(ip: &String, port: &String) -> ViperClient {
        let doorbell = format!("{}:{}", ip, port);
        let stream = TcpStream::connect(doorbell)
            .expect("Doorbell unavailable");

        stream
            .set_read_timeout(Some(Duration::from_millis(TIMEOUT)))
            .unwrap();

        stream
            .set_write_timeout(Some(Duration::from_millis(TIMEOUT)))
            .unwrap();

        ViperClient {
            stream: stream,
            control: Helper::control()
        }
    }

    pub fn channel(&mut self, command: &'static str) -> Channel {
        self.tick();

        Channel::new(&self.control, command)
    }

    pub fn ctpp_channel(&mut self, apt: String, sub: String) -> CTPPChannel {
        self.tick();

        CTPPChannel::new(&self.control, apt, sub)
    }

    pub fn json(bytes: &[u8]) -> JSONResult {
        let json_str =  str::from_utf8(&bytes).unwrap();

        serde_json::from_str(json_str)
    }

    pub fn execute(&mut self, b: &[u8]) -> ByteResult {
        return match self.stream.write(b) {
            Ok(_) => {
                let mut head = [0; 8];
                self.stream.read(&mut head).unwrap();
                let buffer_size = Command::buffer_length(
                    head[2],
                    head[3]
                );

                let mut buf = vec![0; buffer_size];
                self.stream.read(&mut buf).unwrap();
                Ok(buf)
            },
            Err(e) => Err(e)
        }
    }

    // Move the control byte 1 ahead
    fn tick(&mut self) {
        self.control[0] += 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;
    use std::thread;
    use std::net::TcpListener;

    #[test]
    fn test_execute() {
        let listener = TcpListener::bind("127.0.0.1:3333").unwrap();
        let mut client = ViperClient::new(
            &String::from("127.0.0.1"),
            &String::from("3333")
        );

        // This is the doorbell server essentially
        thread::spawn(move || {
            let length = 2;
            let (mut socket, _addr) = listener.accept().unwrap();
            let mut buf = [0; 1];
            socket.read(&mut buf).unwrap();
            socket.write(&[
                0, 0, length, 0, 0, 0, 0, 0,
                65, 65
            ]).unwrap();
        });

        let response = client.execute(&[0]).unwrap();
        assert_eq!(str::from_utf8(&response).unwrap(), "AA");
    }

    #[test]
    fn test_make_command() {
        let listener = TcpListener::bind("127.0.0.1:3334").unwrap();
        let mut client = ViperClient::new(
            &String::from("127.0.0.1"),
            &String::from("3334")
        );

        // This is the doorbell server essentially
        thread::spawn(move || {
            let (mut socket, _addr) = listener.accept().unwrap();
            let mut head = [0; 8];
            socket.read(&mut head).unwrap();

            let bl = Command::buffer_length(head[2], head[3]);
            let mut buf = vec![0; bl];
            socket.read(&mut buf).unwrap();
            socket.write(&[&head, &buf[..]].concat()).unwrap();
        });

        let command = "UCFG".to_string();
        let pre = Command::channel(&command, &client.control, None);
        let r = client.execute(&pre).unwrap();
        assert_eq!(&r[0..8], &[205, 171, 1,  0, 7, 0, 0, 0]);
    }

    #[test]
    fn test_make_uat_command() {
        let listener = TcpListener::bind("127.0.0.1:3335").unwrap();
        let mut client = ViperClient::new(
            &String::from("127.0.0.1"),
            &String::from("3335")
        );

        // This is the doorbell server essentially
        thread::spawn(move || {
            let (mut socket, _addr) = listener.accept().unwrap();
            let mut head = [0; 8];
            socket.read(&mut head).unwrap();

            let bl = Command::buffer_length(head[2], head[3]);
            let mut buf = vec![0; bl];
            socket.read(&mut buf).unwrap();
            socket.write(&[&head, &buf[..]].concat()).unwrap();
        });

        let aut = Command::for_kind(CommandKind::UAUT("ABCDEFG".to_string()), &client.control);
        let r = client.execute(&aut).unwrap();
        assert_eq!(r.len(), 83);
    }
}
