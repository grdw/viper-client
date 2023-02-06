use crate::command::Command;
use std::io::prelude::*;
use std::net::TcpListener;

pub struct SimpleTcpListener{
    listener: TcpListener
}

impl SimpleTcpListener {
    pub fn new(ip: &'static str) -> SimpleTcpListener {
        SimpleTcpListener {
            listener: TcpListener::bind(ip).unwrap()
        }
    }

    pub fn echo(&self) {
        let (mut socket, _addr) = self.listener.accept().unwrap();
        let mut head = [0; 8];
        socket.read(&mut head).unwrap();

        let bl = Command::buffer_length(head[2], head[3]);
        let mut buf = vec![0; bl];
        socket.read(&mut buf).unwrap();
        socket.write(&[&head, &buf[..]].concat()).unwrap();
    }
}
