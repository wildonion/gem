




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

gql ws client 
    |
    |
    ------riker and tokio server (select!{}, spawn(), jobq channels) -------
                                                                            |
                                                    sharded tlps over noise-protocol and tokio-rustls
                                                                            |
                                                                            ----- sharded instances -----
                                                                                        hyper
                                                                                        p2p stacks
                                                                                            - kademlia
                                                                                            - gossipsub over tcp and quic
                                                                                            - noise protocol
                                                                                            - ws and webrtc
                                                                                            - muxer
                                                                                        quic and udp
                                                                                        tcp 
                                                                                        rpc capnp pubsub 
                                                                                        zmq pubsub
                                                                                        gql subs
                                                                                        ws (push notif on data changes, chatapp, realtime monit, webhook setups, mmq and order matching engine)
                                                                                        connections that implement AsyncWrite and AsyncRead traits for reading/writing IO future objects 
                                                                                        redis client pubsub + mongodb

→ an eventloop server can be one of the above sharded tlps which contains an event handler trait 
 (like riker and senerity EventHanlder traits, tokio::select!{} or ws, zmq and rpc pubsub server) 
 to handle the incoming published topics, emitted events or webhooks 


gql subs + ws + redis client <------> ws server + redis server
http request to set push notif <------> http hyper server to publish topic in redis server
json/capnp rpc client <------> json/capnp rpc server
zmq subs <------> zmq pub server
tcp, quic client <------> tcp, quic streaming future io objects server



*/





// #![allow(unused)] //// will let the unused vars be there - we have to put this on top of everything to affect the whole crate
#![macro_use] //// apply the macro_use attribute to the root cause it's an inner attribute (the `!` mark) and will be effect on all things inside this crate 



use std::time::Duration;
use constants::MainResult;
use std::collections::HashSet;
use std::{net::SocketAddr, sync::Arc, env};
use dotenv::dotenv;
use routerify::Router;
use routerify::Middleware;
use uuid::Uuid;
use log::{info, error};
use tokio::sync::oneshot;
use tokio::sync::Mutex; //// async Mutex will be used inside async methods since the trait Send is not implement for std::sync::Mutex
use hyper::{Client, Uri};
use openai::set_key;
use crate::ctx::bot::wwu_bot::GENERAL_GROUP;
use self::contexts as ctx; // use crate::contexts as ctx; - ctx can be a wrapper around a predefined type so we can access all its field and methods
use serenity::{prelude::*, framework::{standard::macros::group, StandardFramework}, 
                http, model::prelude::*, Client as BotClient,
                client::bridge::gateway::ShardManager};


pub mod middlewares;
pub mod misc; //// we're importing the utils.rs in here as a public module thus we can access all the modules, functions and macros inside of it in here publicly
pub mod constants;
pub mod contexts;
pub mod schemas;
pub mod controllers;
pub mod routers;













//// the return type of the error part in Result 
//// is a trait which is behind a pointer or Box 
//// since they have no size at compile time and their
//// implementor will be known at runtime thus they must 
//// be behind a pointer like &dyn or inside a Box
//// if we want to return them as a type.
#[tokio::main(flavor="multi_thread", worker_threads=10)] //// use the tokio multi threaded runtime by spawning 10 threads
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
    let openai_key = env::var("OPENAI_KEY").expect("⚠️ no openai key variable set");
    let discord_token = env::var("DISCORD_TOKEN").expect("⚠️ no discord token variable set");
    let host = env::var("HOST").expect("⚠️ no host variable set");
    let port = env::var("CONSE_PORT").expect("⚠️ no port variable set");
    let sms_api_token = env::var("SMS_API_TOKEN").expect("⚠️ no sms api token variable set");
    let sms_template = env::var("SMS_TEMPLATE").expect("⚠️ no sms template variable set");
    let io_buffer_size = env::var("IO_BUFFER_SIZE").expect("⚠️ no io buffer size variable set").parse::<u32>().unwrap() as usize; //// usize is the minimum size in os which is 32 bits
    let (sender, receiver) = oneshot::channel::<u8>(); //// oneshot channel for handling server signals - we can't clone the receiver of the oneshot channel
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
        info!("🫠 no username has passed in to the cli; passing updating access level process");
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
    let arced_mutexd_otp_info = Arc::new( //// in order the OtpInput to be shareable between routers' threads it must be sendable or cloneable and since the Clone trait is not implemented for the OtpInput we're putting it inside the Arc
                                                        Mutex::new( //// in order the OtpInput to be mutable between routers' threads it must be syncable thus we have to put it inside the Mutex which based on mpsc rule means that only one thread can mutate it at a time 
                                                            otp_info
                                                        )
                                                    );
    
















    // -------------------------------- discord bot setups
    //
    // ---------------------------------------------------------------------------------------
    //// each shard is a ws client to the discrod ws server also discord 
    //// requires that there be at least one shard for every 2500 guilds 
    //// (discrod servers) that a bot is on.
    //
    //// data of each bot client must be safe to send between other shards' 
    //// threads means they must be Arc<Mutex<Data>> + Send + Sync + 'static
    //// or an RwLock type also each shard must be Arced and Mutexed to 
    //// be shareable between threads.
    let http = http::Http::new(&discord_token);
    let (owners, _bot_id) = match http.get_current_application_info().await{
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (Some(owners), Some(info.id))
        },
        Err(why) => {
            error!("😖 could not access discord bot application info: {:?}", why);
            (None, None)
        },
    };
    if owners.is_some(){
        let framework = StandardFramework::new().configure(|c| c.owners(owners.unwrap()).prefix("~")).group(&GENERAL_GROUP);
        let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
        let mut bot_client = BotClient::builder(discord_token, intents)
                                                        .framework(framework)
                                                        .event_handler(ctx::bot::wwu_bot::Handler)
                                                        .await
                                                        .expect("creating discord bot client error");
        { 
            //// since we want to borrow the bot_client as immutable we must define 
            //// a new scope to do this because if a mutable pointer exists 
            //// an immutable one can't be there otherwise we get this Error:
            //// cannot borrow `bot_client` as mutable because it is also borrowed as immutable
            let mut data = bot_client.data.write().await; //// data of the bot client is of type RwLock which can be written safely in other threads
            data.insert::<ctx::bot::wwu_bot::ShardManagerContainer>(bot_client.shard_manager.clone()); //// writing a shard manager inside the bot client data
        }
        //// moving the shreable shard (Arc<Mutex<ShardManager>>) 
        //// into tokio green threadpools
        let shard_manager = bot_client.shard_manager.clone(); //// each shard is an Arced Mutexed data that can be shared between other threads safely
        tokio::spawn(async move{
            tokio::signal::ctrl_c().await.expect("😖 failed to plugin CTRL+C signal to the server");
            shard_manager.lock().await.shutdown_all().await; //// once we received the ctrl + c we'll shutdown all shards or ws clients 
            //// we'll print the current statuses of the two shards to the 
            //// terminal every 30 seconds. This includes the ID of the shard, 
            //// the current connection stage, (e.g. "Connecting" or "Connected"), 
            //// and the approximate WebSocket latency (time between when a heartbeat 
            //// is sent to discord and when a heartbeat acknowledgement is received),
            //// note that it may take a minute or more for a latency to be recorded or to
            //// update, depending on how often Discord tells the client to send a heartbeat.
            loop{ //// here we're logging the shard status every 30 seconds
                tokio::time::sleep(Duration::from_secs(30)).await; //// wait for 30 seconds hearbeat
                let lock = shard_manager.lock().await;
                let shard_runners = lock.runners.lock().await;
                for (id, runner) in shard_runners.iter(){
                    info!(
                        "🧩 shard ID {} is {} with a latency of {:?}",
                        id, runner.stage, runner.latency,
                    );
                }
            }
        });
        //// start the bot client with 2 shards or ws clients for listening
        //// for events, there is an ~5 second ratelimit period
        //// between when one shard can start after another.
        if let Err(why) = bot_client.start_shards(2).await{
            error!("😖 discord bot client error: {:?}", why);
        }
    }


    









                                                    

    // -------------------------------- GPT requests
    //
    // ---------------------------------------------------------------------------------------
    let mut gpt = ctx::bot::wwu_bot::Gpt::new().await;
    let mut response = "".to_string();
    let mut gpt_request_command = "";
    
    gpt_request_command = "can you summerize the content inside the bracket like news title as a numbered bullet? [This is a chat log from a group discussion on a messaging platform. The conversation is somewhat disjointed, and it is unclear what the main topic of conversation is. However, members of the group discussed a range of issues related to NFTs and cryptocurrency. LC makes several comments about the modus operandi of ruggers and incentives to buy and raid floors. SolCultures shares a tweet that highlights the sale of YugiSauce #217 on Magic Eden. Several members discuss the risks and losses associated with NFTs. Oxygencube expresses disappointment about their NFT losses and suggests leaving NFTs, while GoatZilla suggests they might have infinite bags that they haven't realized yet. Dead King Dylan advises sticking with two or three projects, while sm0lfish mentions a King who does the same. LC shares an image that generates some laughter, and other members share emoji reactions. Theude mentions a good call he had earlier, and Dead King Dylan observes that he buys every rev share project. Sm0lfish shares a tweet that suggests there might be another big airdrop in Sol.]";
    response = gpt.feed(gpt_request_command).await.current_response;
    info!("ChatGPT Response: {:?}", response);
    
    gpt_request_command = "can you expand the second bulletlist?";
    response = gpt.feed(gpt_request_command).await.current_response;
    info!("ChatGPT Response: {:?}", response);







    
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
    let conse_graceful = conse_server.with_graceful_shutdown(ctx::app::shutdown_signal(receiver));
    if let Err(e) = conse_graceful.await{ //// awaiting on the server to receive the shutdown signal
        unwrapped_storage.db.clone().unwrap().mode = ctx::app::Mode::Off; //// set the db mode of the app storage to off
        error!("😖 conse server error {} - {}", e, chrono::Local::now().naive_local());
    }







        
        
    tokio::signal::ctrl_c().await?;
    println!("conse server stopped due to receiving [ctrl-c]");
        
        
        
        
        
        
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
        let uri = format!("http://{}:{}/auth/home", host, port).as_str().parse::<Uri>().unwrap(); //// parsing it to hyper based uri
        let client = Client::new();
        let Ok(res) = client.get(uri).await else{
            panic!("conse test failed");
        };
        
        
        Ok(())
        

    }

}
