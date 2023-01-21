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
    stream: TcpStream,
    token: String
}

impl ViperClient {
    pub fn new(ip: &'static str, port: u16, token: &String) -> ViperClient {
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
            token: token.to_string()
        }
    }

    pub fn uaut(&mut self) -> Option<String> {
        let control = [117, 95, 0];
        let pre_aut = Self::make_command("UAUT", &control);
        let r = self.execute(&pre_aut).unwrap();
        println!("{:02x?}", &r);

        let aut = Self::make_uaut_command(&self.token, &control);
        let r = self.execute(&aut);

        match r {
            Some(aut_b) => {
                let relevant_bytes = &aut_b;
                let json = String::from_utf8(relevant_bytes.to_vec()).unwrap();
                Some(json)
            },
            None => None
        }
    }

    pub fn ucfg(&mut self) -> Option<String> {
        let control = [118, 95, 0];
        let pre = Self::make_command("UCFG", &control);
        let r = self.execute(&pre).unwrap();
        println!("{:02x?}", &r);

        let com = Self::make_ucfg_command(&control);
        let r = self.execute(&com);

        match r {
            Some(aut_b) => {
                let relevant_bytes = &aut_b;
                let json = String::from_utf8(relevant_bytes.to_vec()).unwrap();
                Some(json)
            },
            None => None
        }
    }

    fn execute(&mut self, b: &[u8]) -> Option<Vec<u8>> {
        let mut head = [0; 8];

        return match self.stream.write(b) {
            Ok(_) => {
                self.stream.read_exact(&mut head).unwrap();
                let buffer_size = Self::buffer_length(
                    head[2],
                    head[3]
                );

                let mut buf = vec![0; buffer_size];
                println!("{}", buffer_size);
                self.stream.read_exact(&mut buf).unwrap();
                Some(buf)
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

    fn buffer_length(b2: u8, b3: u8) -> usize {
        let b2 = b2 as usize;
        let b3 = b3 as usize;

        (b3 * 255) + b2 + 8 + b3
    }
}

mod tests {
    use super::*;
    use std::str;
    use std::thread;
    use std::net::TcpListener;

    #[test]
    fn test_content_length() {
        let control = [1, 2, 0];
        let list = vec![
            (94, 102, 0),
            (367, 103, 1),
            (752, 232, 2),
            (951, 175, 3)
        ];

        for (byte_length, b2, b3) in list {
            let mut s = String::from("A");
            s = s.repeat(byte_length);
            let b = ViperClient::make_generic_command(s, &control);
            assert_eq!(b[2], b2);
            assert_eq!(b[3], b3);
        }
    }

    #[test]
    fn test_buffer_length() {
        assert_eq!(ViperClient::buffer_length(94, 0), 102);
        assert_eq!(ViperClient::buffer_length(109, 0), 117);
        assert_eq!(ViperClient::buffer_length(103, 1), 367);
        assert_eq!(ViperClient::buffer_length(232, 2), 752);
        assert_eq!(ViperClient::buffer_length(175, 3), 951);
    }

    #[test]
    fn test_execute() {
        let listener = TcpListener::bind("127.0.0.1:3333").unwrap();
        let mut client = ViperClient::new(
            "127.0.0.1",
            3333,
            &String::from("ABCDEF")
        );

        thread::spawn(move || {
            let length = 2;
            let (mut socket, _addr) = listener.accept().unwrap();
            let mut buf = [0; 1];
            socket.read(&mut buf).unwrap();
            socket.write(&[
                0, 0, length, 0, 0, 0, 0, 0,
                65, 65, 65, 65, 65, 65, 65, 65, 65, 65
            ]).unwrap();
        });

        let response = client.execute(&[0]).unwrap();
        assert_eq!(
            str::from_utf8(&response).unwrap(),
            "AAAAAAAAAA"
        );
    }
}
