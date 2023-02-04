use std::io;
use std::io::prelude::*;
use std::net::{TcpStream, Shutdown};
use std::time::Duration;
use crate::command::Command;

const TIMEOUT: u64 = 1000;

type ByteResult = Result<Vec<u8>, io::Error>;

pub struct StreamWrapper {
    stream: TcpStream
}

impl StreamWrapper {
    pub fn new(ip: String) -> StreamWrapper {
        let stream = TcpStream::connect(ip)
            .expect("Doorbell unavailable");

        stream
            .set_read_timeout(Some(Duration::from_millis(TIMEOUT)))
            .unwrap();

        stream
            .set_write_timeout(Some(Duration::from_millis(TIMEOUT)))
            .unwrap();

        StreamWrapper { stream: stream }
    }

    pub fn execute(&mut self, b: &[u8]) -> ByteResult {
        match self.write(b) {
            Ok(_) => self.read(),
            Err(e) => Err(e)
        }
    }

    pub fn die(&mut self) {
        self.stream
            .shutdown(Shutdown::Both)
            .expect("shutdown call failed");
    }

    pub fn write(&mut self, b: &[u8]) -> Result<usize, io::Error> {
        self.stream.write(b)
    }

    pub fn read(&mut self) -> ByteResult {
        let mut head = [0; 8];
        self.stream.read(&mut head)?;
        let buffer_size = Command::buffer_length(
            head[2],
            head[3]
        );

        let mut buf = vec![0; buffer_size];
        self.stream.read(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;
    use std::thread;
    use std::net::TcpListener;
    use crate::command::CommandKind;

    #[test]
    fn test_execute() {
        let listener = TcpListener::bind("127.0.0.1:3333").unwrap();
        let mut client = StreamWrapper::new(
            String::from("127.0.0.1:3333")
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
        let mut client = StreamWrapper::new(
            String::from("127.0.0.1:3334")
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
        let pre = Command::channel(&command, &[0, 0], None);
        let r = client.execute(&pre).unwrap();
        assert_eq!(&r[0..8], &[205, 171, 1,  0, 7, 0, 0, 0]);
    }

    #[test]
    fn test_make_uat_command() {
        let listener = TcpListener::bind("127.0.0.1:3335").unwrap();
        let mut client = StreamWrapper::new(
            String::from("127.0.0.1:3335")
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

        let aut = Command::for_kind(
            CommandKind::UAUT("ABCDEFG".to_string()),
            &[0, 0]
        );
        let r = client.execute(&aut).unwrap();
        assert_eq!(r.len(), 83);
    }
}
