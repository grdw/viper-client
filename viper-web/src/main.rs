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
use dotenv::dotenv;
use serde::Serialize;
use serde_json::json;
use std::{io, env};
use viper_client::{ViperClient, ViperError};
use viper_client::device::Device;

#[derive(Debug, Display, Error)]
enum ViperHTTPError {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "not json error")]
    NotJSONError,

    #[display(fmt = "unautorized")]
    Unauthorized
}

impl From<ViperError> for ViperHTTPError {
    fn from(_error: ViperError) -> Self {
        ViperHTTPError::InternalError
    }
}

impl From<io::Error> for ViperHTTPError {
    fn from(_error: io::Error) -> Self {
        ViperHTTPError::InternalError
    }
}

impl From<serde_json::Error> for ViperHTTPError {
    fn from(_error: serde_json::Error) -> Self {
        ViperHTTPError::NotJSONError
    }
}

impl error::ResponseError for ViperHTTPError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ViperHTTPError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ViperHTTPError::NotJSONError => StatusCode::INTERNAL_SERVER_ERROR,
            ViperHTTPError::Unauthorized => StatusCode::UNAUTHORIZED,
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
      ) -> Result<impl Responder, ViperHTTPError> {

    let mut client = ViperClient::new(&config.ip, &config.port);
    let auth_json = client.authorize(config.token.to_string())?;

    if auth_json["response-code"] == 200 {
        let config = client.configuration("all".to_string())?;
        client.shutdown();
        Ok(web::Json(config["vip"].clone()))
    } else {
        println!("{:?}", auth_json);
        Err(ViperHTTPError::Unauthorized)
    }
}

#[post("/api/v1/open")]
async fn open_door(
          _req: HttpRequest,
          config: web::Data<Config>
      ) -> Result<impl Responder, ViperHTTPError> {

    let mut client = ViperClient::new(&config.ip, &config.port);
    let auth_json = client.authorize(config.token.to_string())?;

    if auth_json["response-code"] == 200 {
        let thingy = json!({
            "apt-address": "SB000006",
            "apt-subaddress": 2,
            "user-paramaters": {
                "opendoor-address-book": [
                    {
                        "apt-address": "SB1000001"
                    }
                ]
            }
        });

        client.open_door(&thingy)?;
        client.shutdown();

        Ok(web::Json(DoorOpenRequest { success: true, error: vec![] }))
    } else {
        println!("{:?}", auth_json);
        Err(ViperHTTPError::Unauthorized)
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    println!("Booting a server at http://127.0.0.1:8080");
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
