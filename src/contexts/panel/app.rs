


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

/*
    
    ---- shared state data between clusters using redis 
    ---- shared state data between tokio::spawn() green threadpool using jobq channels by locking on the mutexed data if we want to mutate it   
            and routers' threads using arc, mutex and rwlock also data must be Send + Sync + 'static
            also handle incoming async events into the server using tokio::select!{} eventloop 

    todo - load env var issue
    todo - streaming over ws <----> tokio tcp, upd quic, rpc, zmq pubsub and hyper and actix for game backend on surrealdb and redis
    todo - enum storage key 
    todo - passport!{} macro 
    todo - pointers and borrowing 
    todo - return traits
    todo - god and dev panel using yew and tauri 

*/


#[actix_web::main]
async fn main() -> std::io::Result<()> {


    server!{
        services::init
    }


}