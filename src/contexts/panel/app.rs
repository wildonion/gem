


use redis::aio::Connection as RedisConnection;
use redis::FromRedisValue;
use redis::JsonAsyncCommands;
use redis::cluster::ClusterClient;
use redis::AsyncCommands; //// this trait is required to be imported in here to call set() methods on the cluster connection
use redis::RedisResult;
use hyper::StatusCode;
use uuid::Uuid;
use log::{info, error};
use mongodb::Client;
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
mod events;



#[actix_web::main]
async fn main() -> std::io::Result<()> {


    server!{
        services::init //// passing the api services 
    }



}