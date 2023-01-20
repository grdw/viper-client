use std::env;
use std::fs;
use std::io::prelude::*;
use std::net::{UdpSocket, TcpStream};
use std::{thread, time::Duration};

// This merely polls if somebody is at the door or not
const TOKEN: &'static str = "TOKEN";
const LOCAL_IP: &'static str = "0.0.0.0:7432";
const DOORBELL_IP: &'static str = "192.168.1.9";
const DOORBELL_SCAN_PORT: u16 = 24199;
const DOORBELL_PORT: u16 = 64100;

#[derive(Debug)]
struct Device {
    mac_address: String,
    hw_id: String,
    app_id: String,
    app_version: String,
    system_id: String,
    description: String,
    model_id: String,
}

fn main() {
    let token = env::var(TOKEN).unwrap();
    let device = get_info();
    println!("{:?}", device);

    // Then comes a certain command, god knows what, but the first
    // one is a `UAUT` response, probably an authorize call to the
    // doorbell. This probably means, my device is trying to
    // authorize
    let mut s = 117;

    let pre_aut = make_command("UAUT", s);
    let r = tcp_call(&pre_aut);
    println!("{:02x?}", r);

    let pre_aut = make_command("UAUT", s + 1);
    let r = tcp_call(&pre_aut);
    println!("{:02x?}", r);

    let aut = make_uaut_command(&token, s);
    let r = tcp_call(&aut);

    match r {
        Some(aut_b) => {
            println!("{:02x?}", aut_b);
            println!("WE'RE IN!");
            break;
        },
        None => s += 1
    }
}

fn make_command(command: &'static str, control: u8) -> Vec<u8> {
    // This is the command prefix I see flying by
    // every time
    let command_prefix = [
        0,   6,   15, 0, 0, 0, 0, 0,
        205, 171, 1,  0, 7, 0, 0, 0
    ];

    // The command is then made and two control bytes are
    // added to the end
    let b_comm = command.as_bytes();
    [&command_prefix, &b_comm[..], &[control, 95, 0][..]].concat()
}

fn make_uaut_command(token: &String, control: u8) -> Vec<u8> {
    let command_prefix = [
        0, 6, 109, 0, control, 95, 0, 0
    ];
    let raw_com = fs::read_to_string("UAUT.json").unwrap();
    let com = raw_com.replace("USER-TOKEN", token);
    let b_com = com.as_bytes();
    [&command_prefix, &b_com[..]].concat()
}

fn tcp_call(bytes: &[u8]) -> Option<[u8; 256]> {
    println!("{:02x?}", bytes);
    let doorbell = format!("{}:{}",
                           DOORBELL_IP,
                           DOORBELL_PORT);

    let mut stream = TcpStream::connect(doorbell)
        .expect("Doorbell unavailable");

    stream.set_read_timeout(Some(Duration::from_millis(5000))).unwrap();
    stream.set_write_timeout(Some(Duration::from_millis(5000))).unwrap();

    let mut buf = [0; 256];
    return match stream.write(bytes) {
        Ok(_) => {
            match stream.read(&mut buf) {
                Ok(_) => Some(buf),
                Err(_) => None
            }
        },
        Err(_) => None
    }
}

fn get_info() -> Device {
    let info = "INFO".as_bytes();
    let mut try_count = 1;
    let udp_socket = UdpSocket::bind(LOCAL_IP).expect("Boom!");
    udp_socket
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();

    loop {
        let mut buf = [0; 256];
        let doorbell_scan_location = format!("{}:{}",
                                             DOORBELL_IP,
                                             DOORBELL_SCAN_PORT);

        udp_socket.send_to(&info, &doorbell_scan_location).unwrap();
        let receive = udp_socket.recv_from(&mut buf);

        match &receive {
            Ok(_) => {
                try_count = 1;

                return Device {
                    mac_address: format!("{:02X?}", &buf[14..20]),
                    hw_id: to_string(&buf[20..24]),
                    app_id: to_string(&buf[24..28]),
                    app_version: to_string(&buf[32..112]),
                    system_id: to_string(&buf[112..116]),
                    description: to_string(&buf[116..152]),
                    model_id: to_string(&buf[156..160])
                };
            },
            Err(_) => {
                println!("Idle.... ");
                try_count += 1;

                if try_count > 5 {
                    try_count = 5
                }
            }
        }

        thread::sleep(
            Duration::from_millis(500 * try_count)
        );
    }
}

fn to_string(bytes: &[u8]) -> String {
    let mut vec = bytes.to_vec();
    vec.retain(|n| n > &0);

    String::from_utf8(vec).unwrap()
}
