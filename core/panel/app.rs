

use twitter_v2::User as TwitterUser;
use twitter_v2::id::NumericId;
use twitter_v2::TwitterApi;
use twitter_v2::Tweet;
use twitter_v2::oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier};
use twitter_v2::authorization::{Oauth2Token, BearerToken, Oauth2Client, Scope};
use twitter_v2::query::{TweetField, UserField};
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
use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
use std::env;
use chrono::Utc;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, TokenData};
use tokio_cron_scheduler::{JobScheduler, Job};
use std::time::Instant;
use std::collections::HashSet;
use rand::rngs::ThreadRng;
use futures::StreamExt; /* is required to call the next() method on the streams */


mod apis;
mod misc;
mod constants;
mod services;
mod events;
mod models;
mod schema;
mod error;


#[actix_web::main]
async fn main() -> std::io::Result<()> {


    let server = server!
    {
        /* SERVER CONFIGS */
    };

    server

}