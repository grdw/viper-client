use std::fs;
use std::io;
use std::str;
use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;

const TIMEOUT: u64 = 5000;
// This is the command prefix I see flying by
// every time
const COMMAND_HEADER: [u8; 8] = [205, 171, 1,  0, 7, 0, 0, 0];
// This is another header that's pretty consistent,
// not sure what it's for tbh:
const UNKNOWN_HEADER: [u8; 8] = [239, 1,   3,  0, 2, 0, 0, 0];

pub struct ViperClient {
    stream: TcpStream,
    token: String,
    control: [u8; 3]
}

type CommandResult = Result<serde_json::Value, io::Error>;
type ByteResult = Result<Vec<u8>, io::Error>;

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

    // This command does something.. but I couldn't possibly tell
    // what it does. I'm assuming this is related to the camera in some
    // shape or way, but I'm not sure yet... it returns a threshold
    // of sorts. I'm assuming it opens something for 90 seconds ...
    // ... but what?
    //
    // Notes:
    // - Each consecutive call it borks for some reason and returns
    // what I think is a fault response
    // - All calls featuring apt-addresses and what not are all
    // using the same control bits (So it doesn't tick further).
    // (Perhaps write a separate struct here because it's getting a bit
    // weird)
    //
    // Debug notes:
    // This opens a channel of sorts ... but a TPP channel? Hur hur
    // Toilet paper channel.
    //
    // Also how does this one close? I guess automatically as soon
    // as the doorbell disconnects from the internet.
    // This one either responds with nothing. Just an ACK with
    // the control bytes.
    //
    // All subsequent CTPP requests use the same control bytes,
    // they will move over the same TcpStream, but they all of
    // a sudden switch protocol midway through the result.
    // I'm not sure how any of this works, but I'll have to analyze
    // how and what and why.
    pub fn ctpp(&mut self, vip: &serde_json::Value) -> ByteResult {
        self.tick();

        let apt_address = format!("{}{}",
                                  vip["apt-address"].as_str().unwrap(),
                                  vip["apt-subaddress"]);

        let apt_b = apt_address.as_bytes();

        let total = [
            &vec![0, 10, 0, 0, 0],
            apt_b,
            &[0]
        ].concat();

        let pre = Command::cmd("CTPP", &total[..],  &self.control);
        let tcp_bytes = [&pre[..], &total].concat();

        // Perhaps store control somewhere?
        self.execute(&tcp_bytes)
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
                let buffer_size = Self::buffer_length(
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

    fn buffer_length(b2: u8, b3: u8) -> usize {
        let b2 = b2 as usize;
        let b3 = b3 as usize;

        (b3 * 255) + b2 + b3
    }
}

struct Command { }

impl Command {
    fn preflight(command: &'static str, control: &[u8]) -> Vec<u8> {
        Command::cmd(command, &[], control)
    }

    fn cmd(command: &'static str,
           extra: &[u8],
           control: &[u8]) -> Vec<u8> {

        let b = command.as_bytes();

        let total = [
            &COMMAND_HEADER,
            &b[..],
            &control[..],
            &extra[..]
        ].concat();

        let (length, second) = Command::byte_lengths(&total);
        let header = [0, 6, length, second, 0, 0, 0, 0];

        [&header, &total[..]].concat()
    }

    fn release(control: &[u8]) -> Vec<u8> {
        let total = [
            &UNKNOWN_HEADER,
            &control[0..2],
        ].concat();

        let (length, second) = Command::byte_lengths(&total);
        let header = [0, 6, length, second, 0, 0, 0, 0];

        [&header, &total[..]].concat()
    }

    fn make(b_com: &[u8], control: &[u8]) -> Vec<u8> {
        let (length, second) = Command::byte_lengths(b_com);

        let header = [
            0,
            6,
            length,
            second,
            control[0],
            control[1],
            control[2],
            0
        ];

        [&header, &b_com[..]].concat()
    }

    fn byte_lengths(bytes: &[u8]) -> (u8, u8) {
        let second = bytes.len() / 255;
        let length = (bytes.len() % 255) - second;

        (length as u8, second as u8)
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
            let b = Command::make(s.as_bytes(), &control);
            assert_eq!(b[2], b2);
            assert_eq!(b[3], b3);
        }
    }

    #[test]
    fn test_command_make() {
        let control = [1, 2, 0];
        let b = Command::cmd("UCFG", &[], &control);
        assert_eq!(b[2], 15);
        assert_eq!(b[3], 0);

        let b = Command::cmd("UCFG", &[10, 10, 10], &control);
        assert_eq!(b[2], 18);
        assert_eq!(b[3], 0);
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

            let bl = ViperClient::buffer_length(head[2], head[3]);
            let mut buf = vec![0; bl];
            socket.read(&mut buf).unwrap();
            socket.write(&[&head, &buf[..]].concat()).unwrap();
        });

        let pre = Command::preflight("UCFG", &client.control);
        let r = client.execute(&pre).unwrap();
        assert_eq!(&r[0..8], &COMMAND_HEADER[..]);
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

            let bl = ViperClient::buffer_length(head[2], head[3]);
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

            let bl = ViperClient::buffer_length(head[2], head[3]);
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
