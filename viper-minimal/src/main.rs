use dotenv::dotenv;
use std::env;
use std::{thread, time::Duration};
use viper_client::{ViperClient, ViperError};
use viper_client::device::Device;

fn main() -> Result<(), ViperError> {
    dotenv().ok();

    let token = env::var("TOKEN").unwrap();
    let doorbell_ip = env::var("DOORBELL_IP").unwrap();
    let doorbell_port = env::var("DOORBELL_PORT").unwrap();

    let mut prev = false;
    loop {
        let is_up = Device::poll(&doorbell_ip, &doorbell_port);

        if is_up && !prev {
            println!("Connected!");
            on_connect(&doorbell_ip, &doorbell_port, &token)?;
        } else if !is_up && prev {
            println!("Disconnected!");
        } else if !is_up && !prev {
            println!("Idle...")
        }

        prev = is_up;
        thread::sleep(Duration::from_millis(1000));
    }
}

// This is an example run purely for testing
fn on_connect(doorbell_ip: &String,
              doorbell_port: &String,
              token: &String) -> Result<(), ViperError> {

    let mut client = ViperClient::new(doorbell_ip, doorbell_port);
    println!("INFO: {:?}\n", client.info()?);
    println!("UAUT: {:?}\n", client.authorize(String::from(token))?);
    println!("UCFG: {:?}\n", client.configuration("all".to_string())?);
    println!("FCRG: {:?}\n", client.face_recognition_params()?);

    client.shutdown();

    Ok(())
}
