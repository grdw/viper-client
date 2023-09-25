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
    println!("UAUT: {:?}", client.authorize(token)?);
    // NOTE: There's never a reason to call it with "none", but
    // it's still an option...
     client.configuration("none".to_string())?;
    let config = client.configuration("all".to_string())?;
    println!("UCFG: {:?}", config);
    println!("INFO: {:?}", client.info()?);
    println!("FCRG: {:?}", client.face_recognition_params()?);

    client.open_door(&config["vip"])?;
    client.shutdown();

    Ok(())
}
