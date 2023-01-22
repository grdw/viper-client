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
    token: String,
    control: [u8; 3]
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
            token: token.to_string(),
            control: [117, 95, 0]
        }
    }

    pub fn uaut(&mut self) -> Option<String> {
        self.execute_command("UAUT")
    }

    pub fn ucfg(&mut self) -> Option<String> {
        self.execute_command("UCFG")
    }

    // Move the control byte 1 ahead
    fn tick(&mut self) {
        self.control[0] += 1
    }

    fn execute_command(&mut self, command: &'static str) -> Option<String> {
        self.tick();

        let pre = Command::preflight(command, &self.control);
        let com = Command::make(
            self.command_json(command),
            &self.control
        );

        self.execute(&pre).unwrap();
        let r = self.execute(&com);

        if let Some(aut_b) = r {
            let relevant_bytes = aut_b.to_vec();
            let json = String::from_utf8(relevant_bytes).unwrap();
            Some(json)
        } else {
            None
        }
    }

    fn execute(&mut self, b: &[u8]) -> Option<Vec<u8>> {
        return match self.stream.write(b) {
            Ok(_) => {
                let mut head = [0; 8];
                self.stream.read(&mut head).unwrap();
                let buffer_size = Self::buffer_length(
                    head[2],
                    head[3]
                );

                let mut buf = vec![0; buffer_size];
                self.stream.read(&mut buf).unwrap();
                Some(buf)
            },
            Err(_) => None
        }
    }

    fn command_json(&self, command: &'static str) -> String {
        match command {
            "UAUT" => {
                let raw_com = fs::read_to_string("UAUT.json").unwrap();
                raw_com.replace("USER-TOKEN", &self.token)
            },
            "UCFG" => fs::read_to_string("UCFG.json").unwrap(),
            _ => {
                panic!("Not available {}", command)
            }
        }
    }

    fn buffer_length(b2: u8, b3: u8) -> usize {
        let b2 = b2 as usize;
        let b3 = b3 as usize;

        (b3 * 255) + b2 + b3
    }
}

struct Command { }

impl Command {
    fn preflight(command: &'static str, control: &[u8]) -> Vec<u8> {
        let b_comm = command.as_bytes();

        [&COMMAND_PREFIX, &b_comm[..], &control[..]].concat()
    }

    fn make(com: String, control: &[u8]) -> Vec<u8> {
        let b_com = com.as_bytes();
        let second = b_com.len() / 255;
        let length = (b_com.len() % 255) - second;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;
    use std::thread;
    use std::net::TcpListener;

    #[test]
    fn test_content_length() {
        let control = [1, 2, 0];
        let list = vec![
            (94, 94, 0),
            (117, 117, 0),
            (367, 111, 1),
            (752, 240, 2),
            (951, 183, 3)
        ];

        for (byte_length, b2, b3) in list {
            let mut s = String::from("A");
            s = s.repeat(byte_length);
            let b = Command::make(s, &control);
            assert_eq!(b[2], b2);
            assert_eq!(b[3], b3);
        }
    }

    #[test]
    fn test_buffer_length() {
        assert_eq!(ViperClient::buffer_length(94, 0), 94);
        assert_eq!(ViperClient::buffer_length(109, 0), 109);
        assert_eq!(ViperClient::buffer_length(103, 1), 359);
        assert_eq!(ViperClient::buffer_length(232, 2), 744);
        assert_eq!(ViperClient::buffer_length(175, 3), 943);
    }

    #[test]
    fn test_execute() {
        let listener = TcpListener::bind("127.0.0.1:3333").unwrap();
        let mut client = ViperClient::new(
            "127.0.0.1",
            3333,
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
            "127.0.0.1",
            3334,
            &String::from("ABCDEF")
        );

        // This is the doorbell server essentially
        thread::spawn(move || {
            let (mut socket, _addr) = listener.accept().unwrap();
            let mut head = [0; 8];
            socket.read(&mut head).unwrap();

            let bl = ViperClient::buffer_length(head[2], head[3]);
            let mut buf = vec![0; bl];
            socket.read(&mut buf).unwrap();
            socket.write(&[&head, &buf[..]].concat()).unwrap();
        });

        let pre = Command::preflight("UCFG", &client.control);
        let r = client.execute(&pre).unwrap();
        assert_eq!(&r[0..8], &COMMAND_PREFIX[8..]);
    }

    #[test]
    fn test_make_uat_command() {
        let listener = TcpListener::bind("127.0.0.1:3335").unwrap();
        let mut client = ViperClient::new(
            "127.0.0.1",
            3335,
            &String::from("ABCDEF")
        );

        // This is the doorbell server essentially
        thread::spawn(move || {
            let (mut socket, _addr) = listener.accept().unwrap();
            let mut head = [0; 8];
            socket.read(&mut head).unwrap();

            let bl = ViperClient::buffer_length(head[2], head[3]);
            let mut buf = vec![0; bl];
            socket.read(&mut buf).unwrap();
            socket.write(&[&head, &buf[..]].concat()).unwrap();
        });

        let aut = Command::make(
            client.command_json("UAUT"),
            &client.control
        );
        let r = client.execute(&aut).unwrap();
        assert_eq!(r.len(), 83);
    }
}
