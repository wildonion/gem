





use argon2;
use env_logger::Env;
use openssl::version::number;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::env;
use dotenv::dotenv;
use log::{info, error};

mod pswd;
use pswd::*;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    gen_passwords();


    Ok(())

}