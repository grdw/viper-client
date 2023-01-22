mod device;
mod viper_client;

use device::Device;
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use viper_client::ViperClient;

fn main() {
    dotenv().ok();

    let doorbell_ip = env::var("DOORBELL_IP").unwrap();
    let doorbell_port = env::var("DOORBELL_PORT").unwrap();
    let token = env::var("TOKEN").unwrap();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");

        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut prev = false;
    while running.load(Ordering::SeqCst) {
        let is_up = Device::poll(&doorbell_ip, &doorbell_port);

        if is_up && !prev {
            println!("Connected!");

            let mut client = ViperClient::new(
                &doorbell_ip,
                &doorbell_port,
                &token
            );

            // This is an example run purely for testing
            println!("{:?}", client.uaut().unwrap().to_string());
            let cfg = client.ucfg().unwrap();
            println!("{:?}", cfg.to_string());
            //println!("{:?}", client.frcg().unwrap().to_string());

            // This returns raw bytes or JSON:
            println!("CTPP: {:02x?}", client.ctpp(&cfg["vip"]));
            //println!("{:02x?}", client.cspb());
            println!("{:?}", client.info().unwrap().to_string());
            //println!("RELEASE: {:02x?}", client.release_control());

            // Hi :)

            // Perhaps print UCFG again: ??
            // println!("{:?}", client.ucfg());
        } else if !is_up && prev {
            println!("Disconnected!");
        } else if !is_up && !prev {
            println!("Idle...")
        }

        prev = is_up;
        thread::sleep(Duration::from_millis(1000));
    }
}
