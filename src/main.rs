mod device;
mod viper_client;

use device::Device;
use std::env;
use std::fs;
use std::str;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use viper_client::ViperClient;

const TOKEN: &'static str = "TOKEN";
const LOCAL_IP: &'static str = "127.0.0.1:7878";
const DOORBELL_IP: &'static str = "192.168.1.9";
const DOORBELL_PORT: u16 = 64100;

fn main() {
    let token = env::var(TOKEN).unwrap();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");

        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut prev = false;
    while running.load(Ordering::SeqCst) {
        let is_up = Device::poll(DOORBELL_IP, DOORBELL_PORT);

        if is_up && !prev {
            println!("Connect...");
        } else if !is_up && prev {
            println!("Disconnect...");
        } else if !is_up && !prev {
            println!("Idle...")
        }

        prev = is_up;
        thread::sleep(Duration::from_millis(1000));
    }
}
