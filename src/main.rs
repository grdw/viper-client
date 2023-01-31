mod device;
mod viper_client;

use actix_files::Files;
use actix_web::{
    App,
    HttpRequest,
    HttpResponse,
    HttpServer,
    Responder,
    error,
    get,
    http::{header::ContentType, StatusCode},
    post,
    web,
};
use derive_more::{Display, Error};
use device::Device;
use dotenv::dotenv;
use serde::Serialize;
use std::{io, env};
use viper_client::{ViperClient};
use viper_client::command::{CommandKind};

#[derive(Debug, Display, Error)]
enum ViperError {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "not json error")]
    NotJSONError,

    #[display(fmt = "unautorized")]
    Unauthorized
}

impl From<io::Error> for ViperError {
    fn from(_error: io::Error) -> Self {
        ViperError::InternalError
    }
}

impl From<serde_json::Error> for ViperError {
    fn from(_error: serde_json::Error) -> Self {
        ViperError::NotJSONError
    }
}

impl error::ResponseError for ViperError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ViperError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ViperError::NotJSONError => StatusCode::INTERNAL_SERVER_ERROR,
            ViperError::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }
}

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
      ) -> Result<impl Responder, ViperError> {

    let mut client = ViperClient::new(&config.ip, &config.port);

    let uaut = CommandKind::UAUT(config.token.to_string());
    let ucfg = CommandKind::UCFG("all".to_string());
    let uaut_channel = client.channel("UAUT");
    let ucfg_channel = client.channel("UCFG");

    let auth_json = {
        client.execute(&uaut_channel.open())?;
        let uaut_bytes = client.execute(&uaut_channel.com(uaut))?;
        ViperClient::json(&uaut_bytes)?
    };

    if auth_json["response-code"] == 200 {
        client.execute(&ucfg_channel.open())?;
        let ucfg_bytes = client.execute(&ucfg_channel.com(ucfg))?;
        let ucfg_json = ViperClient::json(&ucfg_bytes)?;

        client.execute(&uaut_channel.close())?;
        client.execute(&ucfg_channel.close())?;
        client.shutdown();
        Ok(web::Json(ucfg_json["vip"].clone()))
    } else {
        println!("{:?}", auth_json);
        Err(ViperError::Unauthorized)
    }
}

#[post("/api/v1/open")]
async fn open_door(
          _req: HttpRequest,
          config: web::Data<Config>
      ) -> Result<impl Responder, ViperError> {

    let mut client = ViperClient::new(&config.ip, &config.port);
    let uaut = CommandKind::UAUT(config.token.to_string());
    let uaut_channel = client.channel("UAUT");

    let auth_json = {
        client.execute(&uaut_channel.open())?;
        let uaut_bytes = client.execute(&uaut_channel.com(uaut))?;
        ViperClient::json(&uaut_bytes)?
    };

    if auth_json["response-code"] == 200 {
        let addr = "SB000006".to_string();
        let sub  = "SB0000062".to_string();
        let act  = "SB1000001".to_string();
        let mut ctpp_channel = client.ctpp_channel();
        client.execute(&ctpp_channel.open(&sub))?;
        client.write(&ctpp_channel.connect_hs(&sub, &addr))?;

        loop {
            // You need to read until you get a [0x60, 0x18]
            // as a response; which means success :^)
            let bytes = client.read()?;
            if &bytes[0..2] == &[0x60, 0x18] {
                break;
            }
        }

        client.write(&ctpp_channel.ack(0x00, &sub, &addr))?;
        client.write(&ctpp_channel.ack(0x20, &sub, &addr))?;

        client.write(&ctpp_channel.link_actuators(&act, &sub))?;
        println!("{:?}", client.read());
        println!("{:?}", client.read());
        client.write(&ctpp_channel.ack(0x00, &act, &sub))?;
        client.write(&ctpp_channel.ack(0x20, &act, &sub))?;

        client.execute(&uaut_channel.close())?;
        client.execute(&ctpp_channel.close())?;
        client.shutdown();

        Ok(web::Json(DoorOpenRequest { success: true, error: vec![] }))
    } else {
        println!("{:?}", auth_json);
        Err(ViperError::Unauthorized)
    }
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
            .service(poll_door)
            .service(list_doors)
            .service(open_door)
            .service(Files::new("/", "./demo").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
