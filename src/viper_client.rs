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
                println!("{:?}", aut_b.len());
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
                match self.stream.read(&mut buf) {
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
        let command_prefix = [
            0, 6, 109, 0, control[0], control[1], control[2], 0
        ];
        let raw_com = fs::read_to_string("UAUT.json").unwrap();
        let com = raw_com.replace("USER-TOKEN", token);
        let b_com = com.as_bytes();

        [&command_prefix, &b_com[..]].concat()
    }

    fn make_ucfg_command(control: &[u8]) -> Vec<u8> {
        let command_prefix = [
            0, 6, 94, 0, control[0], control[1], control[2], 0
        ];
        let com = fs::read_to_string("UCFG.json").unwrap();
        let b_com = com.as_bytes();

        [&command_prefix, &b_com[..]].concat()
    }
}
