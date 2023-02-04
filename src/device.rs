use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const POLL_TIMEOUT: u64 = 100;

#[derive(Debug)]
pub struct Device {}

impl Device {
    pub fn poll(ip: &String, port: &String) -> bool {
        let location = format!("{}:{}", ip, port);
        let duration = Duration::from_millis(POLL_TIMEOUT);
        let addr = location.to_socket_addrs().unwrap().next().unwrap();

        TcpStream::connect_timeout(&addr, duration).is_ok()
    }
}
