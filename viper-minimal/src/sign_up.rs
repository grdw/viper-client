use dotenv::dotenv;
use std::env;
use std::{thread, time::Duration};
use viper_client::{ViperClient, ViperError};
use viper_client::device::Device;

fn main() -> Result<(), ViperError> {
    dotenv().ok();

    let args: Vec<String> = env::args().collect();
    let doorbell_ip = env::var("DOORBELL_IP").unwrap();
    let doorbell_port = env::var("DOORBELL_PORT").unwrap();
    let email = &args[1];

    loop {
        let is_up = Device::poll(&doorbell_ip, &doorbell_port);

        if is_up {
            println!("Connected!");
            on_connect(&doorbell_ip, &doorbell_port, &email)?;
            return Ok(())
        }

        thread::sleep(Duration::from_millis(1000));
    }
}

// This is an example run purely for testing
fn on_connect(doorbell_ip: &String,
              doorbell_port: &String,
              email: &String) -> Result<(), ViperError> {

    let mut client = ViperClient::new(doorbell_ip, doorbell_port);
    let sign_up = client.sign_up(email)?;
    println!("Your token is: {}", sign_up["user-token"].to_string());
    client.shutdown();

    Ok(())
}
