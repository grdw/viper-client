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
    let uaut = CommandKind::UAUT(token.to_string());
    let ucfg = CommandKind::UCFG("none".to_string());
    let ucfg_all = CommandKind::UCFG("all".to_string());

    let uaut_channel = client.channel("UAUT");
    println!("\n === Authorize:");
    client.execute(&uaut_channel.open())?;
    let uaut_bytes = client.execute(&uaut_channel.com(uaut))?;
    println!("{:?}", ViperClient::json(&uaut_bytes));

    let ucfg_channel = client.channel("UCFG");
    let ucfg_json = {
        println!("\n === Config:");
        client.execute(&ucfg_channel.open())?;
        let ucfg_bytes = client.execute(&ucfg_channel.com(ucfg))?;
        ViperClient::json(&ucfg_bytes)?
    };
    println!("{:?}", ucfg_json);

    println!("\n === Info:");
    let info_channel = client.channel("INFO");
    client.execute(&info_channel.open())?;
    let info_bytes = client.execute(&info_channel.com(CommandKind::INFO))?;
    println!("{:?}", ViperClient::json(&info_bytes));
    client.execute(&info_channel.close())?;

    println!("\n === Facial rec:");
    let frcg_channel = client.channel("FRCG");
    client.execute(&frcg_channel.open())?;
    let frcg_bytes = client.execute(&frcg_channel.com(CommandKind::FRCG))?;
    println!("{:?}", ViperClient::json(&frcg_bytes));

    println!("\n === CTPP:");
    let addr = ucfg_json["vip"]["apt-address"].as_str().unwrap();
    let sub = format!("{}{}",
                      addr,
                      ucfg_json["vip"]["apt-subaddress"]);

    let ctpp_channel = client.ctpp_channel(
        addr.to_string(),
        sub.to_string()
    );
    client.execute(&ctpp_channel.open())?;

    println!("\n === CSPB:");
    let cspb_channel = client.channel("CSPB");
    let cspb_bytes = client.execute(&cspb_channel.open())?;
    println!("{:?}", cspb_bytes);

    let ctpp_hs_bytes = client.execute(&ctpp_channel.connect_hs())?;
    println!("{:?}", ctpp_hs_bytes);
    //let ctpp_re1_bytes = client.execute(&ctpp_channel.connect_reply())?;
    //println!("{:?}", ctpp_re1_bytes);
    //let ctpp_re2_bytes = client.execute(&ctpp_channel.connect_second_reply())?;
    //println!("{:?}", ctpp_re2_bytes);

    println!("\n === Config:");
    let ucfg_all_bytes = client.execute(&ucfg_channel.com(ucfg_all))?;
    let ucfg_all_json = ViperClient::json(&ucfg_all_bytes)?;
    println!("{:?}", ucfg_all_json);

    Ok(())
}
