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
=======================
COMMUNICATION PROTOCOLS
=======================
gql subs ws client 
    |
    |
    ------riker and tokio server (select!{}, spawn(), jobq channels) -------
                                                                            |
                                                    sharded tlps over noise-protocol and tokio-rustls
                                                                            |
                                                                            ----- sharded instances, nodes and servers -----
                                                                                            hyper
                                                                                            p2p stacks
                                                                                                - kademlia
                                                                                                - gossipsub over tcp and quic
                                                                                                - noise protocol
                                                                                                - ws and webrtc
                                                                                                - muxer and yamux
                                                                                            tokio quic udp and tcp
                                                                                            rpc capnp/json pubsub 
                                                                                            zmq pubsub (a queue that contains the tasks each of which can be solved inside a tokio::spawn(async move{}))
                                                                                            gql subs
                                                                                            ws (push notif on data changes, chatapp, realtime monit, webhook setups, mmq and order matching engine)
                                                                                            connections that implement AsyncWrite and AsyncRead traits for reading/writing streaming of encoded IO future objects 
                                                                                            redis client pubsub + mongodb
                                                                                            
➙ event driven means we must have an event handler or listener on client side to subs to fired or emitted events on the 
 server side, these handlers can be predefined traits or an eventloop like tokio::select!{} which listen to the events 
 coming from the shareded tlp servers over ws, zmq or rpc here is the flow of realtiming:
                    ws, gql, rpc and zmq pubs to fired or emitted events <--
                                                                            |
                                                        notifs or streaming of future io objects
                                                                            |
                                                                            ---> ws, gql, rpc and zmq subs or event handler traits for subscribing to emitted events
                    gql subs + ws + redis client <------> ws server + redis server
                    http request to set push notif <------> http hyper server to publish topic in redis server
                    json/capnp rpc client <------> json/capnp rpc server
                    zmq subs <------> zmq pub server
                    tcp, quic client <------> tcp, quic streaming future io objects server
                    discord client
                            subs to emitted event/webhooks inside each shard <----------------- ws/http -----------------> discord ws and http shards and nodes 
                    discord shards and nodes  
                            shard 1 <---------- full duplex streaming and multiplexing over tokio tcp and quic, zmq/ json and capnp rpc pubsub ----------> shard 2  
                                |                                                                                                                              |
                                --------------------------------------- cassandra and mongodb -----------------------------------------------------------------
*/





// #![allow(unused)] //// will let the unused vars be there - we have to put this on top of everything to affect the whole crate
#![macro_use] //// apply the macro_use attribute to the root cause it's an inner attribute (the `!` mark) and will be effect on all things inside this crate 


use serde::{Serialize, Deserialize};
use tokio_cron_scheduler::{JobScheduler, JobToRun, Job};
use std::time::Duration;
use constants::MainResult;
use serenity::framework::standard::buckets::LimitedFor;
use std::collections::{HashSet, HashMap};
use std::{net::SocketAddr, sync::Arc, env};
use dotenv::dotenv;
use routerify::Router;
use routerify::Middleware;
use uuid::Uuid;
use log::{info, error};
use once_cell::sync::Lazy;
use futures::executor::block_on;
use tokio::sync::oneshot;
use tokio::sync::Mutex; //// async Mutex will be used inside async methods since the trait Send is not implement for std::sync::Mutex
use hyper::{Client, Uri, Body};
use chrono::{TimeZone, Timelike, Datelike, Utc}; //// this trait is rquired to be imported here to call the with_ymd_and_hms() method on a Utc object since every Utc object must be able to call the with_ymd_and_hms() method 



pub mod middlewares;
pub mod misc; //// we're importing the misc.rs in here as a public module thus we can access all the modules, functions, macros and pre defined types inside of it in here publicly
pub mod constants;
pub mod schemas;
pub mod controllers;
pub mod routers;



// first import a a rust crate file a module 
// inside the current crate then use crate::module_name::*
// to load all the methods, types and functions from the 
// that module.
// use crate::*; // load from lib.rs or main.rs
// use self::*; // load from the module itself
// use super::*; // load from the root of the crate 








//// the return type of the error part in Result 
//// is a trait which is behind a pointer or Box 
//// since they have no size at compile time and their
//// implementor will be known at runtime thus they must 
//// be behind a pointer like &dyn or inside a Box
//// if we want to return them as a type.
//
//// in Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> 
//// we can return the structure that implements the Error trait for the error part
//// which can be a custom error struct.
#[tokio::main(flavor="multi_thread", worker_threads=10)] //// use the tokio multi threaded runtime by spawning 10 green threads
async fn main() -> MainResult<(), Box<dyn std::error::Error + Send + Sync + 'static>>{ //// generic types can also be bounded to lifetimes ('static in this case) and traits inside the Box<dyn ... > - since the error that may be thrown has a dynamic size at runtime we've put all these traits inside the Box (a heap allocation pointer) and bound the error to Sync, Send and the static lifetime to be valid across the main function and sendable and implementable between threads
    
    

    

     


    // -------------------------------- environment variables setup
    //
    // ---------------------------------------------------------------------
    env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();
    dotenv().expect("⚠️ .env file not found");
    let db_host = env::var("DB_HOST").expect("⚠️ no db host variable set");
    let db_port = env::var("DB_PORT").expect("⚠️ no db port variable set");
    let db_username = env::var("DB_USERNAME").expect("⚠️ no db username variable set");
    let db_password = env::var("DB_PASSWORD").expect("⚠️ no db password variable set");
    let db_engine = env::var("DB_ENGINE").expect("⚠️ no db engine variable set");
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let environment = env::var("ENVIRONMENT").expect("⚠️ no environment variable set");
    let host = env::var("HOST").expect("⚠️ no host variable set");
    let port = env::var("CONSE_PORT").expect("⚠️ no port variable set");
    let sms_api_token = env::var("SMS_API_TOKEN").expect("⚠️ no sms api token variable set");
    let sms_template = env::var("SMS_TEMPLATE").expect("⚠️ no sms template variable set");
    let io_buffer_size = env::var("IO_BUFFER_SIZE").expect("⚠️ no io buffer size variable set").parse::<u32>().unwrap() as usize; //// usize is the minimum size in os which is 32 bits
    let (sender, receiver) = oneshot::channel::<u8>(); //// oneshot channel for handling server signals - we can't clone the receiver of the oneshot channel
    let (discord_bot_flag_sender, mut discord_bot_flag_receiver) = tokio::sync::mpsc::channel::<bool>(io_buffer_size); //// reading or receiving from the mpsc channel is a mutable process
    

    
    
    

    


    // -------------------------------- app storage setup
    //
    // ------------------------------------------------------------------
    let app_storage = db!{ //// this publicly has exported inside the misc so we can access it here 
        db_name,
        db_engine,
        db_host,
        db_port,
        db_username,
        db_password
    };
    







    // -------------------------------- update to dev access level
    //
    // ------------------------------------------------------------------
    let args: Vec<String> = env::args().collect();
    let mut username_cli = &String::new(); //// this is a mutable reference to the username_cli String location inside the heap since we want to mutate the content inside the heap using the pointer later
    let mut access_level_cli = &String::new(); //// this is a mutable reference to the access_level_cli String location inside the heap since we want to mutate the content inside the heap using the pointer later
    if args.len() > 1{
        username_cli = &args[1];
        access_level_cli = &args[2];
    }
    if username_cli != &"".to_string() && access_level_cli != &"".to_string(){
        match misc::set_user_access(username_cli.to_owned(), access_level_cli.parse::<i64>().unwrap(), app_storage.clone()).await{
            Ok(user_info) => {
                info!("🔓 access level for user {} has been updated successfully", username_cli);
                info!("🧑🏻 updated user {:?}", user_info);
            },
            Err(empty_doc) => {
                info!("🤔 no user found for updating access level");
            },
        }
    } else{
        info!("🫠 no username has passed in to the cli; no updating process is required for access level");
    }






    


    // -------------------------------- initializing the otp info instance
    //
    // ---------------------------------------------------------------------------------------
    let mut otp_auth = misc::otp::Auth::new(sms_api_token, sms_template); //// the return type is impl Otp trait which we can only access the trait methods on the instance - it must be defined as mutable since later we want to get the sms response stream to decode the content, cause reading it is a mutable process
    let otp_info = misc::app::OtpInfo{
        //// since otp_auth is of type trait, in order 
        //// to have a trait in struct field or function
        //// param we have to use it behind a pointer 
        //// by putting it inside the Box<dyn Trait> or use &dyn Trait  
        otp_auth: Box::new(otp_auth), 
    };
    //// following data can be shared between hyper threads
    let arced_mutexd_otp_info = Arc::new( //// in order the OtpInput to be shareable between routers' threads it must be sendable or cloneable and since the Clone trait is not implemented for the OtpInput we're putting it inside the Arc
                                                        Mutex::new( //// in order the OtpInput to be mutable between routers' threads it must be syncable thus we have to put it inside the Mutex which is based on mpsc rule, means that only one thread can mutate it at a time 
                                                            otp_info
                                                        )
                                                    );
    












    
    // -------------------------------- building the conse api server from the registered routers
    //
    //      we're sharing the db_instance state between routers' threads to get the data inside each api
    //      and for this the db data must be shareable and safe to send between threads which must be bounded
    //      to Send + Sync traits 
    // --------------------------------------------------------------------------------------------------------
    let unwrapped_storage = app_storage.unwrap(); //// unwrapping the app storage to create a db instance
    let db_instance = unwrapped_storage.get_db().await; //// getting the db inside the app storage; it might be None
    let api = Router::builder()
        .data(db_instance.unwrap().clone()) //// shared state which will be available to every route handlers is the db_instance which must be Send + Syn + 'static to share between threads
        .data(discord_bot_flag_sender.clone()) //// sharing the discord bot sender between routers' threads which must be Send + Syn + 'static or Arc<Mutex<Sender<bool>>> to share between threads
        .middleware(Middleware::pre(middlewares::logging::logger)) //// enable logging middleware on the incoming request then pass it to the next middleware - pre Middlewares will be executed before any route handlers and it will access the req object and it can also do some changes to the request object if required
        .middleware(Middleware::post(middlewares::cors::allow)) //// the path that will be fallen into this middleware is "/*" thus it has the OPTIONS route in it also post middleware accepts a response object as its param since it only can mutate the response of all the requests before sending them back to the client
        .scope("/auth", routers::auth::register().await)
        .scope("/event", routers::event::register().await)
        .scope("/game", routers::game::register().await)
        .scope("/bot", routers::bot::register().await)
        .scope("/whitelist", routers::whitelist::register().await)
        .scope("/redis", routers::redis::register().await)
        // .scope("/mmq") // TODO - used for match making queue
        // .scope("/ws") // TODO - used for chatapp routes
        // .scope("/gql") // TODO - used for subscriptions like sub to push notifs and chatapp
        .build()
        .unwrap();
    info!("🏃‍♀️ running {} server on port {} - {}", misc::app::APP_NAME, port, chrono::Local::now().naive_local());
    let conse_server = misc::build_server(api).await; //// build the server from the series of api routers
    let conse_graceful = conse_server.with_graceful_shutdown(misc::app::shutdown_signal(receiver)); //// in shutdown_signal() function we're listening to the data coming from the sender   
    // sender.send(0).unwrap(); //// sending the shutdown signal to the downside of the channel, the receiver part will receive the signal once the server gets shutdown gracefully on ctrl + c
    if let Err(e) = conse_graceful.await{ //// awaiting on the server to start and handle the shutdown signal if there was any error
        unwrapped_storage.db.clone().unwrap().mode = misc::app::Mode::Off; //// set the db mode of the app storage to off
        error!("😖 conse server error {} - {}", e, chrono::Local::now().naive_local());
    }
    
    
    
    

    Ok(())
    





}












// -------------------------------- conse test apis
//
// -----------------------------------------------------------------

#[cfg(test)]
mod tests{

    use super::*;

    #[tokio::test]
    async fn home() -> Result<(), hyper::Error>{
        
        //// building the server for testing
        dotenv().expect("⚠️ .env file not found");
        let host = env::var("HOST").expect("⚠️ no host variable set");
        let port = env::var("CONSE_PORT").expect("⚠️ no port variable set");
        let api = Router::builder()
                .scope("/auth", routers::auth::register().await)
                .build()
                .unwrap();
        let conse_server = misc::build_server(api).await;
        if let Err(e) = conse_server.await{ //// we should await on the server to run for testing
            eprintln!("conse server error in testing: {}", e);
        }

        //// sending the started conse server a get request to auth home 
        let uri = format!("http://{}:{}/auth/home", host, port).as_str().parse::<Uri>().unwrap(); //// parsing it to a hyper based uri
        let client = Client::new();
        let Ok(res) = client.get(uri).await else{
            panic!("conse test failed");
        };
        
        
        Ok(())
        

    }

}