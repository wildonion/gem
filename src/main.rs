




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
                                                                                            quic and udp
                                                                                            tcp 
                                                                                            rpc capnp/json pubsub 
                                                                                            zmq pubsub (a queue that contains the tasks each of which can be solved inside a tokio::spawn(async move{}))
                                                                                            gql subs
                                                                                            ws (push notif on data changes, chatapp, realtime monit, webhook setups, mmq and order matching engine)
                                                                                            connections that implement AsyncWrite and AsyncRead traits for reading/writing IO future objects 
                                                                                            redis client pubsub + mongodb

➙ an eventloop or event listener server can be one of the above sharded tlps which contains an event handler trait 
 (like riker and senerity EventHanlder traits, tokio channels and tokio::select!{} or ws, zmq and rpc pubsub server) 
 to handle the incoming published topics over zmq and rpc (json and capnp) server, 
  emitted events over ws server or webhooks over http


➙ event driven means we must have an event handler or listener on client side to subs to fired or emitted events on the 
 server side, these handlers can be predefined traits or an event loop like tokio::select!{} which listen to the events 
 coming from the server over ws, zmq or rpc here is the flow of realtiming:
                    ws, gql, rpc and zmq pubs to fired or emitted events <--
                                                                            |
                                                        notifs or streaming of future io objects
                                                                            |
                                                                            ---> ws, gql, rpc and zmq subs or event handler traits for firing or emit events

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


use tokio_cron_scheduler::{JobScheduler, JobToRun, Job};
use std::time::Duration;
use constants::MainResult;
use serenity::framework::standard::buckets::LimitedFor;
use std::collections::HashSet;
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
use hyper::{Client, Uri};
use openai::set_key;
use crate::ctx::bot::cmds::framework_command::{ASKGPT_GROUP, BOT_HELP};
use self::contexts as ctx; // use crate::contexts as ctx; - ctx can be a wrapper around a predefined type so we can access all its field and methods
use serenity::{prelude::*, framework::StandardFramework, http, Client as BotClient};
use chrono::{TimeZone, Timelike, Datelike, Utc}; //// this trait is rquired to be imported here to call the with_ymd_and_hms() method on a Utc object since every Utc object must be able to call the with_ymd_and_hms() method 
use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt, CpuExt};
use openai::{ //// openai crate is using the reqwest lib under the hood
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole}
};
use serenity::{async_trait, model::prelude::{MessageId, UserId, ChannelId, 
                interaction::application_command::{CommandDataOption, CommandDataOptionValue}, command::CommandOption}, 
                framework::standard::{macros::{help, hook}, 
                HelpOptions, help_commands, CommandGroup}
            };
use serenity::model::Timestamp;
use serenity::builder;
use serenity::utils::Colour;
use serenity::model::prelude::command::CommandOptionType;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::application::command::Command;
use serenity::model::channel::Message;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::{prelude::*, 
                model::prelude::ResumedEvent, 
                framework::standard::{
                    Args,
                    CommandResult, macros::{command, group}
                }
            };

pub mod middlewares;
pub mod misc; //// we're importing the misc.rs in here as a public module thus we can access all the modules, functions and macros inside of it in here publicly
pub mod constants;
pub mod contexts;
pub mod schemas;
pub mod controllers;
pub mod routers;





pub static GPT: Lazy<ctx::gpt::chat::Gpt> = Lazy::new(|| {
    block_on(ctx::gpt::chat::Gpt::new())
});







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
    
    


    
    let (sender_flag, mut receiver_flag) = 
        tokio::sync::mpsc::channel::<u8>(1024); //// mpsc means multiple thread can read the data but only one of them can mutate it at a time
    tokio::spawn(async move{

        // solve heavy async task inside tokio green threadpool
        // send data inside the pool to receive it in different 
        // parts of the app
        sender_flag.send(1).await.unwrap(); //// sending data to the downside of the tokio jobq channel

    });
    while let Some(data) = receiver_flag.recv().await{
        // do whatever with data 
        // ...
    }



     


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
    let openai_key = env::var("OPENAI_KEY").expect("⚠️ no openai key variable set");
    let discord_token = env::var("DISCORD_TOKEN").expect("⚠️ no discord token variable set");
    let serenity_shards = env::var("SERENITY_SHARDS").expect("⚠️ no shards variable set");
    let host = env::var("HOST").expect("⚠️ no host variable set");
    let port = env::var("CONSE_PORT").expect("⚠️ no port variable set");
    let sms_api_token = env::var("SMS_API_TOKEN").expect("⚠️ no sms api token variable set");
    let sms_template = env::var("SMS_TEMPLATE").expect("⚠️ no sms template variable set");
    let io_buffer_size = env::var("IO_BUFFER_SIZE").expect("⚠️ no io buffer size variable set").parse::<u32>().unwrap() as usize; //// usize is the minimum size in os which is 32 bits
    let (sender, receiver) = oneshot::channel::<u8>(); //// oneshot channel for handling server signals - we can't clone the receiver of the oneshot channel
    let (discord_bot_flag_sender, mut discord_bot_flag_receiver) = tokio::sync::mpsc::channel::<bool>(io_buffer_size); //// reading or receiving from the mpsc channel is a mutable process
    set_key(openai_key);
    

    
    
    

    


    // -------------------------------- app storage setup
    //
    // ------------------------------------------------------------------
    let app_storage = db!{ //// this publicly has exported inside the utils so we can access it here 
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
    let otp_info = ctx::app::OtpInfo{
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
    








    // -------------------------------- setting up discord bot
    //
    // ---------------------------------------------------------------------------------------
    // sending the start bot flag to the downside of the channel
    discord_bot_flag_sender.send(true).await.unwrap(); //// TODO - an event that set this to true is required 
    //// waiting to receive the flag from the sender
    //// to activate the bot if it was a true flag  
    if let Some(flag) = discord_bot_flag_receiver.recv().await{
        if flag{
            misc::activate_discord_bot(discord_token.as_str(), 
                                        serenity_shards.parse::<u64>().unwrap(), 
                                        GPT.clone()).await;
        }
    }

    








    
    // -------------------------------- building the conse server from the router
    //
    //      we're sharing the db_instance state between routers' threads to get the data inside each api
    //      and for this the db data must be shareable and safe to send between threads which must be bounded
    //      to Send + Sync traits 
    // --------------------------------------------------------------------------------------------------------
    let unwrapped_storage = app_storage.unwrap(); //// unwrapping the app storage to create a db instance
    let db_instance = unwrapped_storage.get_db().await; //// getting the db inside the app storage; it might be None
    let api = Router::builder()
        .data(db_instance.unwrap().clone()) //// shared state which will be available to every route handlers is the db_instance which must be Send + Syn + 'static to share between threads
        .middleware(Middleware::pre(middlewares::logging::logger)) //// enable logging middleware on the incoming request then pass it to the next middleware - pre Middlewares will be executed before any route handlers and it will access the req object and it can also do some changes to the request object if required
        .middleware(Middleware::post(middlewares::cors::allow)) //// the path that will be fallen into this middleware is "/*" thus it has the OPTIONS route in it also post middleware accepts a response object as its param since it only can mutate the response of all the requests before sending them back to the client
        .scope("/auth", routers::auth::register().await)
        .scope("/event", routers::event::register().await)
        .scope("/game", routers::game::register().await)
        .scope("/whitelist", routers::whitelist::register().await)
        .scope("/redis", routers::redis::register().await)
        // .scope("/ws") // TODO - used for chatapp routes
        // .scope("/gql") // TODO - used for subscriptions like sub to push notifs and chatapp
        .build()
        .unwrap();
    info!("🏃‍♀️ running {} server on port {} - {}", ctx::app::APP_NAME, port, chrono::Local::now().naive_local());
    let conse_server = misc::build_server(api).await; //// build the server from the series of api routers
    let conse_graceful = conse_server.with_graceful_shutdown(ctx::app::shutdown_signal(receiver)); //// in shutdown_signal() function we're listening to the data coming from the sender   
    if let Err(e) = conse_graceful.await{ //// awaiting on the server to start and handle the shutdown signal if there was any error
        unwrapped_storage.db.clone().unwrap().mode = ctx::app::Mode::Off; //// set the db mode of the app storage to off
        error!("😖 conse server error {} - {}", e, chrono::Local::now().naive_local());
    }
    
    
    // TODO - send the 0 flag on any error
    // sender.send(0).unwrap(); //// sending the shutdown signal to the downside of the channel, the receiver part will receive the signal once the server gets shutdown gracefully on ctrl + c


    
        
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
