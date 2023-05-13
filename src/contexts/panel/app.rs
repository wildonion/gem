


use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse, get, ResponseError};
use actix_web::middleware::Logger;
use actix_multipart::Multipart;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use env_logger::Env;
use serde::{Serialize, Deserialize};
use crate::apis::dev::dev_service_init;

mod apis;




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


    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("devops/openssl/conse_key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("devops/openssl/conse_cert.pem").unwrap();

    let server = HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i %t %P %r %s %b %T %D"))
            .configure(dev_service_init) // dev_service_init is a closure of type traits which can be called inside the configure method
        })
        .bind_openssl("127.0.0.1:8080", builder)?
        // .bind((host.as_str(), port))?
        .workers(10)
        .run()
        .await;

    server


}