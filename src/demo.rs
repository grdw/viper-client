mod device;
mod viper_client;

use device::Device;
use dotenv::dotenv;
use std::env;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use viper_client::{ViperClient};
use viper_client::command::CommandKind;

fn main() -> Result<(), io::Error> {
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
            on_connect(&doorbell_ip, &doorbell_port, &token)?;
        } else if !is_up && prev {
            println!("Disconnected!");
        } else if !is_up && !prev {
            println!("Idle...")
        }

        prev = is_up;
        thread::sleep(Duration::from_millis(1000));
    }

    Ok(())
}

fn on_connect(doorbell_ip: &String,
              doorbell_port: &String,
              token: &String) -> Result<(), io::Error> {

    let mut client = ViperClient::new(doorbell_ip, doorbell_port);

    // This is an example run purely for testing
    {
        let uaut = CommandKind::UAUT(token.to_string());
        let uaut_channel = client.channel("UAUT");

        println!("\n === Authorize:");
        client.execute(&uaut_channel.open())?;

        let uaut_bytes = client.execute(&uaut_channel.com(uaut))?;
        println!("{:?}", ViperClient::json(&uaut_bytes));
        client.execute(&uaut_channel.close())?;
    }

    let ucfg = CommandKind::UCFG("none".to_string());
    let ucfg_all = CommandKind::UCFG("all".to_string());

    // This channel is opened but closed at the very end
    let ucfg_channel = client.channel("UCFG");
    println!("\n === Config:");
    client.execute(&ucfg_channel.open())?;
    let ucfg_bytes = client.execute(&ucfg_channel.com(ucfg))?;
    let ucfg_json = ViperClient::json(&ucfg_bytes)?;
    println!("{:?}", ucfg_json);

    // Test run for info
    {
        println!("\n === Info:");
        let info_channel = client.channel("INFO");
        client.execute(&info_channel.open())?;
        let info_bytes = client.execute(&info_channel.com(CommandKind::INFO))?;
        println!("{:?}", ViperClient::json(&info_bytes));
        client.execute(&info_channel.close())?;
    }

    // Test run for facial recognition
    {
        println!("\n === Facial rec:");
        let frcg_channel = client.channel("FRCG");
        client.execute(&frcg_channel.open())?;
        let frcg_bytes = client.execute(&frcg_channel.com(CommandKind::FRCG))?;
        println!("{:?}", ViperClient::json(&frcg_bytes));
        client.execute(&frcg_channel.close())?;
    }

    // Test run for CTPP:
    println!("\n === CTPP:");
    let addr = ucfg_json["vip"]["apt-address"].to_string();
    let sub = format!("{}{}",
                      addr,
                      ucfg_json["vip"]["apt-subaddress"]);

    let mut ctpp_channel = client.ctpp_channel();
    client.execute(&ctpp_channel.open(&sub))?;

    println!("\n === CSPB:");
    let cspb_channel = client.channel("CSPB");
    let cspb_bytes = client.execute(&cspb_channel.open())?;
    println!("{:?}", cspb_bytes);

    // @madchicken is right, you need to read twice
    client.write(&ctpp_channel.connect_hs(&sub, &addr))?;
    println!("{:?}", client.read());
    println!("{:?}", client.read());

    client.write(&ctpp_channel.connect_actuators(0, &sub, &addr))?;
    client.write(&ctpp_channel.connect_actuators(1, &sub, &addr))?;

    // NOTE:
    // The first run always fails. The second run right after the first
    // one, succeeds, and I'm able to see the rest of the actuators.
    // Why does the first run fail? It seems like I'm missing a
    // call to CTPP
    println!("\n === Config:");
    let ucfg_all_bytes = client.execute(&ucfg_channel.com(ucfg_all))?;
    let ucfg_all_json = ViperClient::json(&ucfg_all_bytes)?;

    let act = ucfg_all_json["vip"]
                           ["user-parameters"]
                           ["opendoor-address-book"]
                           [0]
                           ["apt-address"].to_string();

    client.write(&ctpp_channel.link_actuators(&act, &sub))?;
    println!("{:?}", client.read());
    println!("{:?}", client.read());
    client.write(&ctpp_channel.connect_actuators(0, &act, &sub))?;
    client.write(&ctpp_channel.connect_actuators(1, &act, &sub))?;

    // Close the remaining channels
    client.execute(&ucfg_channel.close())?;
    client.execute(&ctpp_channel.close())?;
    client.execute(&cspb_channel.close())?;
    client.shutdown();

    Ok(())
}
