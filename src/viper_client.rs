mod command;
mod ctpp;

use command::Command;
use ctpp::CTPP;
use std::fs;
use std::io;
use std::str;
use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;

const TIMEOUT: u64 = 5000;

pub struct ViperClient {
    stream: TcpStream,
    token: String,
    control: [u8; 3]
}

type CommandResult = Result<serde_json::Value, io::Error>;
type ByteResult = Result<Vec<u8>, io::Error>;
type CTPPResult = Result<CTPP, io::Error>;

impl ViperClient {
    pub fn new(ip: &String, port: &String, token: &String) -> ViperClient {
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
            token: token.to_string(),
            control: [117, 95, 0]
        }
    }

    // This command is used to authorize and create a session
    pub fn uaut(&mut self) -> CommandResult {
        self.json_command("UAUT")
    }

    // This command returns the configuration
    pub fn ucfg(&mut self) -> CommandResult {
        self.json_command("UCFG")
    }

    // This command returns the information
    pub fn info(&mut self) -> CommandResult {
        self.json_command("INFO")
    }

    // This command (and this is best guess) return information
    // related to face recognition.
    pub fn frcg(&mut self) -> CommandResult {
        self.json_command("FRCG")
    }

    pub fn ctpp(&mut self, vip: &serde_json::Value) -> CTPPResult {
        self.tick();

        let addr = vip["apt-address"].as_str().unwrap();
        let sub = &vip["apt-subaddress"];
        let apt_address = format!("{}{}", addr, sub);
        let apt_b = format!("\0\0\0{}\0", apt_address);

        let total = [
            &vec![0, 10],
            apt_b.as_bytes()
        ].concat();

        let pre = Command::cmd("CTPP", &total[..],  &self.control);
        let tcp_bytes = [&pre[..], &total].concat();

        match self.execute(&tcp_bytes) {
            Ok(bytes) => Ok(
                CTPP::new(self.control, addr.to_string(), apt_address)
            ),
            Err(e) => Err(e)
        }
    }

    pub fn release_control(&mut self) -> ByteResult {
        let total = Command::release(&self.control);

        self.execute(&total)
    }

    pub fn cspb(&mut self) -> ByteResult {
        self.tick();

        let pre = Command::preflight("CSPB", &self.control);
        self.execute(&pre)
    }

    // Move the control byte 1 ahead
    fn tick(&mut self) {
        self.control[0] += 1
    }

    fn json_command(&mut self, command: &'static str) -> CommandResult {
        self.tick();

        let pre = Command::preflight(command, &self.control);
        let com = Command::make(
            self.command_json(command).as_bytes(),
            &self.control
        );

        self.execute(&pre)?;
        let r = self.execute(&com);

        match r {
            Ok(com_b) => {
                let json_str = str::from_utf8(&com_b).unwrap();
                Ok(serde_json::from_str(json_str).unwrap())
            },
            Err(e) => Err(e)
        }
    }

    fn execute(&mut self, b: &[u8]) -> ByteResult {
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

    fn command_json(&self, command: &'static str) -> String {
        let path = format!("commands/{}.json", command);
        let raw_com = fs::read_to_string(&path).unwrap();

        match command {
            "UCFG" | "INFO" | "FRCG" => raw_com,
            "UAUT" => raw_com.replace("USER-TOKEN", &self.token),
            _ => panic!("Not available {}", command)
        }
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
            &String::from("3333"),
            &String::from("ABCDEF")
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
            &String::from("3334"),
            &String::from("ABCDEF")
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

        let pre = Command::preflight("UCFG", &client.control);
        let r = client.execute(&pre).unwrap();
        assert_eq!(&r[0..8], &[205, 171, 1,  0, 7, 0, 0, 0]);
    }

    #[test]
    fn test_make_uat_command() {
        let listener = TcpListener::bind("127.0.0.1:3335").unwrap();
        let mut client = ViperClient::new(
            &String::from("127.0.0.1"),
            &String::from("3335"),
            &String::from("ABCDEF")
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

        let aut = Command::make(
            client.command_json("UAUT").as_bytes(),
            &client.control
        );
        let r = client.execute(&aut).unwrap();
        assert_eq!(r.len(), 83);
    }

    #[test]
    fn test_ctpp() {
        let listener = TcpListener::bind("127.0.0.1:3336").unwrap();
        let mut client = ViperClient::new(
            &String::from("127.0.0.1"),
            &String::from("3336"),
            &String::from("ABCDEF")
        );

        thread::spawn(move || {
            let (mut socket, _addr) = listener.accept().unwrap();
            let mut head = [0; 8];
            socket.read(&mut head).unwrap();

            let bl = Command::buffer_length(head[2], head[3]);
            let mut buf = vec![0; bl];
            socket.read(&mut buf).unwrap();
            socket.write(&[&head, &buf[..]].concat()).unwrap();
        });

        let data = r#"
            {
                "apt-address":"SB0000011",
                "apt-subaddress": 2
            }
        "#;
        let v: serde_json::Value = serde_json::from_str(data).unwrap();
        _ = client.ctpp(&v);
    }
}
