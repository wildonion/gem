





use argon2;
use env_logger::Env;
use openssl::version::number;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::{env, collections::HashMap};
use dotenv::dotenv;
use log::{info, error};

mod pswd;
use pswd::*;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    /* ---------------- generate admin and dev passwords ---------------- */
    gen_passwords();

    let body_content = format!("Use this code to get verified in ----: ----");
    let mut data = HashMap::new();
    data.insert("sender", "----".to_string());
    data.insert("destination", "------".to_string());
    data.insert("content", body_content);

    let otp_endpoint = format!("https://api.thesmsworks.co.uk/v1/message/send");
    
    let otp_response = reqwest::Client::new()
        .post(otp_endpoint.as_str())
        .header("Authorization", "")
        .json(&data)
        .send()
        .await;

    let res_stat = otp_response
        .as_ref()
        .unwrap()
        .status()
        .as_u16();


    println!("status {}", res_stat);


    // other test logics
    // ...

    Ok(())

}