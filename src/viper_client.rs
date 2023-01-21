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
        let mut stream = TcpStream::connect(doorbell)
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
        let r = self.execute(&pre_aut).unwrap();
        println!("{:02x?}", &r[0..18]);

        let aut = Self::make_uaut_command(&token, &control);
        let r = self.execute(&aut);

        match r {
            Some(aut_b) => {
                let relevant_bytes = &aut_b[8..109+15];
                let json = String::from_utf8(relevant_bytes.to_vec()).unwrap();
                Some(json)
            },
            None => None
        }
    }

    fn execute(&mut self, bytes: &[u8]) -> Option<[u8; 256]> {
        let mut buf = [0; 256];
        return match self.stream.write(bytes) {
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
}
