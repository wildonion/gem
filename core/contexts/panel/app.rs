


use sha2::{Digest, Sha256};
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use redis::aio::Connection as RedisConnection;
use redis::Client as RedisClient;
use redis::AsyncCommands; //// this trait is required to be imported in here to call set() methods on the cluster connection
use redis::RedisResult;
use hyper::StatusCode;
use uuid::Uuid;
use log::{info, error};
use mongodb::Client;
use actix_cors::Cors;
use actix_web::{web, cookie::{self, Cookie, Expiration, time::Duration, time::OffsetDateTime}, 
                web::Data, http::header, App, HttpRequest, 
                HttpServer, Responder, HttpResponse, get, post, ResponseError};
use actix_web::middleware::Logger;
use actix_multipart::Multipart;
use env_logger::Env;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::env;
use chrono::Utc;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, TokenData};


mod apis;
mod misc;
mod constants;
mod services;
mod events;
mod models;
mod schema;



#[actix_web::main]
async fn main() -> std::io::Result<()> {


    let server = server!
    {
        /* SERVER SETUP */
    };
    server


}