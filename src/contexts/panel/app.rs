


use futures::executor::block_on;
use once_cell::sync::Lazy;
use redis::aio::Connection as RedisConnection;
use redis::FromRedisValue;
use redis::JsonAsyncCommands;
use redis::cluster::ClusterClient;
use redis::AsyncCommands; //// this trait is required to be imported in here to call set() methods on the cluster connection
use redis::RedisResult;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client as SurrealClient;
use hyper::StatusCode;
use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse, get, ResponseError};
use actix_web::middleware::Logger;
use actix_multipart::Multipart;
use env_logger::Env;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

mod apis;
mod misc;
mod constants;
mod services;



#[actix_web::main]
async fn main() -> std::io::Result<()> {


    server!{
        services::init //// passing the api services 
    }


}