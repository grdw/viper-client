use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const POLL_TIMEOUT: u64 = 750;

#[derive(Debug)]
pub struct Device {}

impl Device {
    pub fn poll(doorbell_ip: &'static str, doorbell_port: u16) -> bool {
        let location = format!("{}:{}", doorbell_ip, doorbell_port);
        let duration = Duration::from_millis(POLL_TIMEOUT);
        let addr = location.to_socket_addrs().unwrap().next().unwrap();

        TcpStream::connect_timeout(&addr, duration).is_ok()
    }
}
