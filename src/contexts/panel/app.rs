

use hyper::StatusCode;
use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse, get, ResponseError};
use actix_web::middleware::Logger;
use actix_multipart::Multipart;
use env_logger::Env;
use serde::{Serialize, Deserialize};
use crate::apis::dev::dev_service_init;

mod apis;
mod misc;
mod constants;



#[actix_web::main]
async fn main() -> std::io::Result<()> {
  
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let redis_node_addr = std::env::var("REDIS_HOST").expect("⚠️ no redis host variable set");
    let host = std::env::var("HOST").expect("⚠️ no host variable set");
    let port = std::env::var("PANEL_PORT").expect("⚠️ no panel port variable set").parse::<u16>().unwrap();
    let client = redis::Client::open(redis_node_addr.as_str()).unwrap();
    let mut redis_conn = client.get_async_connection().await.unwrap();
    

    // TODO - surrealdb setups
    // ...




    let server = HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i %t %P %r %s %b %T %D"))
            .configure(dev_service_init) // dev_service_init is a closure of type traits which can be called inside the configure method
        })
        .bind((host.as_str(), port))?
        .workers(10)
        .run()
        .await;

    server


}