





use argon2;
use env_logger::Env;
use openssl::version::number;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::{collections::HashMap, env, thread};
use dotenv::dotenv;
use log::{info, error};

mod pswd;
use pswd::*;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{

    // env_logger::init_from_env(Env::default().default_filter_or("info"));

    // /* ---------------- generate admin and dev passwords ---------------- */
    // gen_passwords();

    // let body_content = format!("Use this code to get verified in ----: ----");
    // let mut data = HashMap::new();
    // data.insert("sender", "----".to_string());
    // data.insert("destination", "------".to_string());
    // data.insert("content", body_content);

    // let otp_endpoint = format!("https://api.thesmsworks.co.uk/v1/message/send");
    
    // let otp_response = reqwest::Client::new()
    //     .post(otp_endpoint.as_str())
    //     .header("Authorization", "")
    //     .json(&data)
    //     .send()
    //     .await;

    // // if let Ok(resp) = otp_response{
    // //     if resp.status().is_success(){
    // //         println!("success status");
    // //     }
    // // }
        
    // let res_stat = otp_response
    //     .as_ref()
    //     .unwrap()
    //     .status()
    //     .as_u16();


    // println!("status {}", res_stat);


    // other test logics
    // ...


    // let now = chrono::Local::now().naive_local();
    // let (tx, rx) = std::sync::mpsc::channel::<String>();
    // for n in 0..10000{
    //     let tx = tx.clone();
    //     thread::spawn(move ||{
    //         tx.send(String::from("wildonion"));
    //     });

    //     while let Ok(data) = rx.recv(){
    //         println!("received data: {:?}", data);
    //     }

    // }
    // let end = chrono::Local::now().naive_local();
    // let diff = end - now;
    // println!("made: {:?}", diff.num_nanoseconds());


    
    tokio::spawn(async move{
        sleep4().await;
    });

    async fn sleep2(){
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        println!("wake up after 2")
    }

    async fn sleep4(){
        tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
        println!("wake up after 4");
    }

    tokio::spawn(async move{
        sleep2().await;
    });

    // there must be some sleep or loop{} to keeps the app awake
    // so the background workers can do their jobs
    loop{}




    Ok(())

}