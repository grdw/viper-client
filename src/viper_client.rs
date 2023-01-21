use std::fs;
use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;

const TIMEOUT: u64 = 5000;
// This is the command prefix I see flying by
// every time
const COMMAND_PREFIX: [u8; 16] = [
    0,   6,   15, 0, 0, 0, 0, 0,
    205, 171, 1,  0, 7, 0, 0, 0
];

pub struct ViperClient {
    stream: TcpStream
}

impl ViperClient {
    pub fn new(ip: &'static str, port: u16) -> ViperClient {
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
            stream: stream
        }
    }

    pub fn uaut(&mut self, token: &String) -> Option<String> {
        let control = [117, 95, 0];
        let pre_aut = Self::make_command("UAUT", &control);
        let r = self.execute(&pre_aut, 20).unwrap();
        println!("{:02x?}", &r);

        let aut = Self::make_uaut_command(&token, &control);
        let r = self.execute(&aut, 124);

        match r {
            Some(aut_b) => {
                let relevant_bytes = &aut_b[8..124];
                let json = String::from_utf8(relevant_bytes.to_vec()).unwrap();
                Some(json)
            },
            None => None
        }
    }

    pub fn ucfg(&mut self) -> Option<String> {
        let control = [118, 95, 0];
        let pre = Self::make_command("UCFG", &control);
        let r = self.execute(&pre, 20).unwrap();
        println!("{:02x?}", &r);

        let com = Self::make_ucfg_command(&control);
        let r = self.execute(&com, 951);

        match r {
            Some(aut_b) => {
                let relevant_bytes = &aut_b[8..951];
                let json = String::from_utf8(relevant_bytes.to_vec()).unwrap();
                Some(json)
            },
            None => None
        }
    }

    fn execute(&mut self, b: &[u8], b_size: usize) -> Option<Vec<u8>> {
        let mut buf = vec![0; b_size];

        return match self.stream.write(b) {
            Ok(_) => {
                match self.stream.read_exact(&mut buf) {
                    Ok(_) => Some(buf),
                    Err(_) => None
                }
            },
            Err(_) => None
        }
    }

    fn make_command(command: &'static str, control: &[u8]) -> Vec<u8> {
        let b_comm = command.as_bytes();

        [&COMMAND_PREFIX, &b_comm[..], &control[..]].concat()
    }

    fn make_uaut_command(token: &String, control: &[u8]) -> Vec<u8> {
        let raw_com = fs::read_to_string("UAUT.json").unwrap();
        let com = raw_com.replace("USER-TOKEN", token);

        Self::make_generic_command(com, control)
    }

    fn make_ucfg_command(control: &[u8]) -> Vec<u8> {
        let com = fs::read_to_string("UCFG.json").unwrap();

        Self::make_generic_command(com, control)
    }

    fn make_generic_command(com: String, control: &[u8]) -> Vec<u8> {
        let b_com = com.as_bytes();
        let second = b_com.len() / 255;

        let length = if second > 0 {
            (b_com.len() % 255) - 8 - second
        } else {
            (b_com.len() % 255) + 8
        };

        let command_prefix = [
            0,
            6,
            length as u8,
            second as u8,
            control[0],
            control[1],
            control[2],
            0
        ];

        [&command_prefix, &b_com[..]].concat()
    }
}

#[test]
fn test_content_length() {
    let mut s = String::from("A");
    s = s.repeat(94);
    let b = ViperClient::make_generic_command(
        s,
        &[1, 2, 0]
    );
    assert_eq!(b[2], 102);
    assert_eq!(b[3], 0);

    let mut s = String::from("A");
    s = s.repeat(367);
    let b = ViperClient::make_generic_command(
        s,
        &[1, 2, 0]
    );
    assert_eq!(b[2], 103);
    assert_eq!(b[3], 1);

    let mut s = String::from("A");
    s = s.repeat(752);
    let b = ViperClient::make_generic_command(
        s,
        &[1, 2, 0]
    );
    assert_eq!(b[2], 232);
    assert_eq!(b[3], 2);

    let mut s = String::from("A");
    s = s.repeat(951);
    let b = ViperClient::make_generic_command(
        s,
        &[1, 2, 0]
    );
    assert_eq!(b[2], 175);
    assert_eq!(b[3], 3);
}
