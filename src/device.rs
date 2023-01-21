use std::net::{UdpSocket};
use std::{thread, time::Duration};

const LOCAL_IP: &'static str = "0.0.0.0:7432";
const DOORBELL_SCAN_PORT: u16 = 24199;

#[derive(Debug)]
pub struct Device {
    mac_address: String,
    hw_id: String,
    app_id: String,
    app_version: String,
    system_id: String,
    description: String,
    model_id: String,
}

fn to_string(bytes: &[u8]) -> String {
    let mut vec = bytes.to_vec();
    vec.retain(|n| n > &0);

    String::from_utf8(vec).unwrap()
}

impl Device {
    pub fn get_info(doorbell_ip: &'static str) -> Device {
        let info = "INFO".as_bytes();
        let mut try_count = 1;
        let udp_socket = UdpSocket::bind(LOCAL_IP).expect("Boom!");
        udp_socket
            .set_read_timeout(Some(Duration::from_millis(10)))
            .unwrap();

        loop {
            let mut buf = [0; 256];
            let doorbell_scan_location = format!("{}:{}",
                                                 doorbell_ip,
                                                 DOORBELL_SCAN_PORT);

            udp_socket.send_to(&info, &doorbell_scan_location).unwrap();
            let receive = udp_socket.recv_from(&mut buf);

            match &receive {
                Ok(_) => {
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
}
