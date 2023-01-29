mod device;
mod viper_client;

use actix_web::{Error, get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use device::Device;
use dotenv::dotenv;
use serde::Serialize;
use std::{env, fs};
use viper_client::{ViperClient};
use viper_client::command::{CommandKind};

#[derive(Clone)]
struct Config {
    ip: String,
    port: String,
    token: String
}

#[derive(Serialize)]
struct Poll {
    available: bool
}

#[derive(Serialize)]
struct DoorOpenRequest {
    success: bool,
    error: Vec<u8>
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct Door {
    id: u8,
    name: String,
    apt_address: String,
    output_index: u8,
    secure_mode: bool
}

#[get("/")]
async fn index() -> impl Responder {
    let index = fs::read_to_string("demo/index.html").unwrap();

    HttpResponse::Ok().body(index)
}

#[get("/api/v1/poll")]
async fn poll_door(_req: HttpRequest,
                   config: web::Data<Config>) -> impl Responder {

    let available = Device::poll(&config.ip, &config.port);

    web::Json(Poll { available: available })
}

#[get("/api/v1/doors")]
async fn list_doors(
        _req: HttpRequest,
        config: web::Data<Config>
    ) -> Result<impl Responder, Error> {

    let mut client = ViperClient::new(&config.ip, &config.port);

    let uaut = CommandKind::UAUT(config.token.to_string());
    let ucfg = CommandKind::UCFG("all".to_string());
    let uaut_channel = client.channel("UAUT");
    let ucfg_channel = client.channel("UCFG");

    // TODO: Return a 403 FORBIDDEN if the auth doesn't succeed
    {
        client.execute(&uaut_channel.open())?;
        let uaut_bytes = client.execute(&uaut_channel.com(uaut))?;
        let json = ViperClient::json(&uaut_bytes).unwrap();
        println!("{:?}", json.to_string());
    }

    client.execute(&ucfg_channel.open())?;
    let ucfg_bytes = client.execute(&ucfg_channel.com(ucfg))?;
    let ucfg_json = ViperClient::json(&ucfg_bytes)?;

    client.execute(&uaut_channel.close())?;
    client.execute(&ucfg_channel.close())?;
    client.shutdown();

    Ok(web::Json(ucfg_json["vip"].clone()))
}

#[post("/api/v1/open")]
async fn open_door(_req: HttpRequest,
                   config: web::Data<Config>) -> impl Responder {

    let mut client = ViperClient::new(&config.ip, &config.port);
    client.shutdown();

    web::Json(DoorOpenRequest { success: true, error: vec![] })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(|| {
        App::new()
            .app_data(
                web::Data::new(
                    Config {
                        ip: env::var("DOORBELL_IP").unwrap(),
                        port: env::var("DOORBELL_PORT").unwrap(),
                        token: env::var("TOKEN").unwrap()
                    }
                )
            )
            .service(index)
            .service(poll_door)
            .service(list_doors)
            .service(open_door)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
