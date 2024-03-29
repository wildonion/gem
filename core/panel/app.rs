/*



Coded by


 █     █░ ██▓ ██▓    ▓█████▄  ▒█████   ███▄    █  ██▓ ▒█████   ███▄    █ 
▓█░ █ ░█░▓██▒▓██▒    ▒██▀ ██▌▒██▒  ██▒ ██ ▀█   █ ▓██▒▒██▒  ██▒ ██ ▀█   █ 
▒█░ █ ░█ ▒██▒▒██░    ░██   █▌▒██░  ██▒▓██  ▀█ ██▒▒██▒▒██░  ██▒▓██  ▀█ ██▒
░█░ █ ░█ ░██░▒██░    ░▓█▄   ▌▒██   ██░▓██▒  ▐▌██▒░██░▒██   ██░▓██▒  ▐▌██▒
░░██▒██▓ ░██░░██████▒░▒████▓ ░ ████▓▒░▒██░   ▓██░░██░░ ████▓▒░▒██░   ▓██░
░ ▓░▒ ▒  ░▓  ░ ▒░▓  ░ ▒▒▓  ▒ ░ ▒░▒░▒░ ░ ▒░   ▒ ▒ ░▓  ░ ▒░▒░▒░ ░ ▒░   ▒ ▒ 
  ▒ ░ ░   ▒ ░░ ░ ▒  ░ ░ ▒  ▒   ░ ▒ ▒░ ░ ░░   ░ ▒░ ▒ ░  ░ ▒ ▒░ ░ ░░   ░ ▒░
  ░   ░   ▒ ░  ░ ░    ░ ░  ░ ░ ░ ░ ▒     ░   ░ ░  ▒ ░░ ░ ░ ▒     ░   ░ ░ 
    ░     ░      ░  ░   ░        ░ ░           ░  ░      ░ ░           ░ 
                      ░                                                  


    -----------------------------------------------------------------
   |          NOTE ON CODE ORDER EXECUTION OF ASYNC METHODS
   |-----------------------------------------------------------------
   | in rust the order of execution is not async by default but rather it's thread safe 
   | and without having race conditions due to its rules of mutable and immutable pointers 
   | of types although if there might be async methods but it's not expected that they must 
   | be executed asyncly, the early one gets executed first and then the second one goes, 
   | an example of that would be calling async_method_one() method with async operations 
   | inside, and other_async_method_two() method, both of them are async however, but the 
   | code waits till all the async operations inside the first one get executed then run the 
   | second one, this gets criticized if we have some delay and sleep methods inside the first 
   | one which gets us into trouble with the whole process of code order execution if we don't 
   | want to have disruption in their execution, though in some cases it's needed to have this 
   | logic but in here it would be a bad idea, the solution to this is running both of them 
   | asyncly in their own seprate threadpool which can be done by putting each of them inside 
   | tokio::spawn() in this case there would be no disruption in their order execution at all 
   | and we'd have a fully async execution of methods in the background.
   | to catch any result data inside the tokio::spawn() we would have to use mpsc channel to
   | send the data to the channel inside the tokio::spawn() and receive it outside of tokio
   | scope and do the rest of the logics with that.
   | notice of putting .await after async task call, it consumes the future objcet by pinning
   | it into the ram for future solvation also it suspend the function execution until the future
   | gets compeleted allows other parts of the app get executed without having any disruption,
   | later once the future has completed waker sends the value back to the caller to update the 
   | state and its value, this behaviour is called none blocking completion but based on the 
   | beginning notes the order of async task execution is not async by itself and if we have 
   | async_task1 first followed by async_task2 the order of executing them is regular and it's 
   | not pure async like goroutines in Go or event loop in NodeJS, to achive this we should spawn 
   | them insie tokio::spawn which runs all the futures in the background like what goroutines 
   | and NodeJS event loop do
   |
   | conclusion: 
   | use tokio::spawn() to execute any async task in the background without having
   | any disruption in other order execution of async methods.
   | 


    -----------------------------------------------------------------
   |                  ACTIX RUNTIME VS TOKIO RUNTIME
   |-----------------------------------------------------------------
   | to use actix actors, http and ws wee need to add #[actix_web::main] on top of main method
   | but this doesn't mean we can't use the tokio::spawn() to execute task in the background 
   | putting #[tokio::main] doesn't allow us to run the actix tasks in actix runtime itself, 
   | note that we should avoid starting any actor in the background inside tokio::spawn otherwise
   | we'll face the error of `spawn_local` called from outside of a `task::LocalSet` which says
   | that we can't start an actor inside the tokio runtime instead we should use the actix runtime
   | and start the actor in the context of actix runtime itself where there is no #[tokio::main]
   | on top of main method, good to know that #[tokio::main] simplifies the sentup and execution
   | of async tasks within the tokio runtime on the other hans tokio::spawn is used for manually
   | spawning, managing and scheduling async task in the background you need to handle task lifecycles, 
   | cancellation, and coordination between tasks explicitly
   |

*/

use lettre::{
    message::header::ContentType as LettreContentType,
    transport::smtp::authentication::Credentials, 
    AsyncSmtpTransport, AsyncTransport, Message as LettreMessage,
    Tokio1Executor, 
};
use std::io::BufWriter;
use std::str::FromStr;
use std::{fs::OpenOptions, io::BufReader};
use rand::Rng;
use rand::random;
use sha2::{Digest, Sha256};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use redis::aio::Connection as RedisConnection;
use redis::Client as RedisClient;
use redis::AsyncCommands; // this trait is required to be imported in here to call set() methods on the cluster connection
use redis::RedisResult;
use redis::Commands;
use redis_async::client::{self, PubsubConnection, ConnectionBuilder};
use redis::RedisError;
use hyper::StatusCode;
use uuid::Uuid;
use log::{info, error};
use mongodb::Client;
use actix_session::{Session, SessionMiddleware, storage::RedisActorSessionStore};
use actix_redis::{Command, RedisActor, resp_array, RespValue};
use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
use actix_cors::Cors;
use actix_web::{App, Error, web, cookie::{self, Cookie, time::Duration, time::OffsetDateTime}, 
                web::Data, http::header, HttpRequest, 
                HttpServer, Responder, HttpResponse, get, post, ResponseError};
use actix_web::middleware::Logger;
use actix_multipart::Multipart;
use env_logger::Env;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use std::fmt::Write;
use tokio::io::AsyncReadExt;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
use tokio::time::Duration as TokioDuration;
use std::env;
use chrono::Utc;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, TokenData};
use std::time::Instant;
use std::collections::HashSet;
use rand::rngs::ThreadRng;
use futures::StreamExt; /* is required to call the next() method on the streams */
use once_cell::sync::Lazy;
use constants::PanelHttpResponse;
use panel_macros::passport; /* loading from lib.rs which contains proc macros */
use snowflake::SnowflakeIdGenerator;
use std::rc::Weak;
use tokio::sync::RwLock;
use crate::constants::*;
use tokio_util::codec::{BytesCodec, FramedRead};
use spacetimedb_sdk::{
    Address,
    disconnect,
    identity::{load_credentials, once_on_connect, save_credentials, Credentials as SpacetimedbCredentials, Identity},
    on_disconnect, on_subscription_applied,
    reducer::Status,
    subscribe,
    table::{TableType, TableWithPrimaryKey},
};



/* ----------------------------------------- */
/* ------------ loading modules ------------ */
/* ----------------------------------------- */
#[path="spacetimedb/client/chatdb/mod.rs"] /* mod.rs contains all modules, methods, structures and functions */
mod spacetimchatdb; /* contains spacetimedb client interfaces for chatdb server */
mod apis;           /* contains http routes and model call logics */
mod constants;      /* contains constant and static types */
mod services;       /* contains service handler to register routes */
mod events;         /* contains realtiming event pubsub logics based on ws, tcp and redis */
mod models;         /* contains models, schemas structures and db query calls */
mod schema;         /* contains diesel db schemas */
mod helpers;        /* contains miscellaneous and utilities methods and modules, server handler methods and macros, error handler, kyc process of the api body, passport traits for HttpRequest and all env vars */
mod adapters;       /* contains all third party apis */
mod middlewares;    /* contains all middlewares logics */



#[actix_web::main]
async fn main() -> std::io::Result<()> {

    /*
        >_ running a tcp listener server, actix will use this to accept 
        incoming tcp based connections in its threadpool
    */
    dotenv::dotenv().expect(".env file must be in here!");
    let tcp_listener = std::net::TcpListener::bind(
    format!("{}:{}", 
            std::env::var("HOST").expect("⚠️ no host variable set"), 
            std::env::var("PANEL_PORT").expect("⚠️ no panel port variable set").parse::<u16>().unwrap()
    )).unwrap();


    let server = bootsteap!
    {
        /* SERVER CONFIGS */
        tcp_listener
    };

    server

}