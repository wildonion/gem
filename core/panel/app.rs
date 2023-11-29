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
use utoipa::OpenApi;
use utoipa::{ToSchema, IntoParams};
use utoipa::{
    openapi::security::{
        ApiKey, 
        ApiKeyValue, 
        SecurityScheme
    },
};
use utoipa::Modify;
use utoipa::openapi::security::{Http, HttpAuthScheme};
use utoipa_swagger_ui::{SwaggerUi, Url};
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
use tokio_cron_scheduler::{JobScheduler, Job};
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
#[path="spacetimedb/client/chatdb/mod.rs"]
mod spacetimchatdb; /* contains spacetimedb client interfaces for chatdb server */
mod apis;           /* contains http routes and model call logics */
mod misc;           /* contains miscellaneous and utilities methods and modules */
mod constants;      /* contains constant and static types */
mod services;       /* contains service handler to register routes */
mod events;         /* contains realtiming event pubsub logics based on ws */
mod models;         /* contains models, schemas structures and db query calls */
mod schema;         /* contains diesel db schemas */
mod error;          /* contains error handler logis */
mod adapters;       /* contains all third party apis */
mod server;         /* contains server handler methods and macros */
mod kyced;          /* contains kyc process of the api body */
mod passport;       /* contains passport traits for HttpRequest */




#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv::dotenv().expect(".env file must be in here!");
    let tcp_listener = std::net::TcpListener::bind(
    format!("{}:{}", 
            std::env::var("HOST").expect("⚠️ no host variable set"), 
            std::env::var("PANEL_PORT").expect("⚠️ no panel port variable set").parse::<u16>().unwrap()
    )).unwrap();


    let server = server!
    {
        /* SERVER CONFIGS */
        tcp_listener
    };

    server

}