





use redis::RedisError;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use futures::future;
use redis::FromRedisValue;
use redis::JsonAsyncCommands;
use redis::cluster::ClusterClient;
use redis::AsyncCommands; //// this trait is required to be imported in here to call set() methods on the cluster connection
use redis::RedisResult;
use serde::{Serialize, Deserialize};
use std::{rc::Rc, cell::RefCell};
use std::collections::{HashSet, HashMap};
use std::{net::SocketAddr, sync::Arc, env};
use std::time::Duration;
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
use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt, CpuExt, DiskExt}; //// methods of trait DiskExt can be used on each Disk instance to get information of the disk because Disk struct has private methods and we can access them by call the trait DiskExt methods which has been implemented for the Disk struct  
use serenity::{async_trait, model::prelude::{MessageId, UserId, ChannelId, 
                interaction::application_command::{CommandDataOption, CommandDataOptionValue}, command::CommandOption}, 
                framework::standard::{macros::{help, hook}, 
                HelpOptions, help_commands, CommandGroup}
            };
use serenity::model::application::interaction::MessageFlags;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::{prelude::*, framework::StandardFramework, http, Client as BotClient};
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



pub mod daemon;
pub mod schemas;
pub mod handlers;





#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{



    env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();
    dotenv().expect("⚠️ .env file not found");
    let discord_token = env::var("TWISCORD_DISCORD_TOKEN").expect("⚠️ no discord token variable set");
    let serenity_shards = env::var("SERENITY_SHARDS").expect("⚠️ no shards variable set");
    let io_buffer_size = env::var("IO_BUFFER_SIZE").expect("⚠️ no io buffer size variable set").parse::<u32>().unwrap() as usize; //// usize is the minimum size in os which is 32 bits
    let (discord_bot_flag_sender, mut discord_bot_flag_receiver) = tokio::sync::mpsc::channel::<bool>(io_buffer_size); //// reading or receiving from the mpsc channel is a mutable process
    let redis_password = env::var("REDIS_PASSWORD").expect("⚠️ no redis password variable set");
    let redis_host = std::env::var("REDIS_HOST").expect("⚠️ no redis host variable set");
    let redis_conn_url = format!("redis://:{}@{}", redis_password, redis_host);
    let redis_client = redis::Client::open(redis_conn_url.as_str()).unwrap();
    let (redis_pubsub_msg_sender, redis_pubsubs_msg_receiver) = tokio::sync::mpsc::channel::<String>(io_buffer_size);
    let (response_sender, mut response_receiver) = tokio::sync::mpsc::channel::<Vec<schemas::Mention>>(io_buffer_size);






    // -------------------------------- sending twitter mentions responses to mpsc channel
    //
    // -------------------------------------------------------------------------------------------
    /* 
        receiving is a mutable process since the underlying 
        method which is recv() has &mut self as it's first param 
        which is a mutable pointer to the response_receiver 
        instance in which mutating the content of the pointer
        will mutate the content of the instance too.
    */
    tokio::spawn(async move{
        
        let res = reqwest::get("https://some-domain/get-twitter-mentions")
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let Ok(_) = response_sender.send(res).await else{
            panic!("couldn't send to response sender mpsc channel");
        };

    });








    // -------------------------------- subscribing to redis pubsub channel
    //
    // ---------------------------------------------------------------------------------------
    /* 
        we must put the whole loop{} inside 
        the tokio::spawn(async move{}) to 
        avoid blocking issues.
    */
    tokio::spawn(async move{



        /* once we received the mention response from the sender we'll publish it to the redis pubsub channel */
        while let Some(res) = response_receiver.recv().await{

            let mut redis_conn = redis_client.clone().get_async_connection().await.unwrap();
            info!("📢 publishing twitter user mentions response to redis pubsub [twitter-mentions-response] channel");  
            let pubsub_message = serde_json::to_string_pretty(&res).unwrap();
            let _: Result<_, RedisError> = redis_conn.publish::<String, String, String>("twitter-mentions-response".to_string(), pubsub_message).await;
            
        }


        /* we should constantly subscribing to the redis channel once we received the topics from channels */
        loop{

            let mut redis_conn = redis_client.get_connection().unwrap();
            let mut pubsub = redis_conn.as_pubsub();
            
            pubsub.subscribe("user_mentions").unwrap();

            let msg = pubsub.get_message().unwrap();
            let payload: String = msg.get_payload().unwrap();


            /* UNCOMMENT THE FOLLOWING IF YOU WANT TO SEND TEST MESSAGES */

            // /* cloning the redis pubsub mpsc sender to prevent from moving in each iteration before tokio spawn */
            // let redis_pubsub_msg_sender = redis_pubsub_msg_sender.clone();
            // tokio::spawn(async move{

            //     /* sending a test payload to the redis pubsub mpsc channel */
            //     redis_pubsub_msg_sender.clone().send(payload.clone()).await;
            // });


            let mentions_response = serde_json::from_str::<Vec<schemas::Mention>>(payload.as_str()).unwrap();

            let log_name = format!("[{}]", chrono::Local::now());
            let filepath = format!("logs/pubsub/mentions-{log_name:}.log");
            let log_content = format!("{mentions_response:?}");
            let mut error_kind_log = tokio::fs::File::create(filepath.as_str()).await.unwrap();
            error_kind_log.write_all(log_content.as_bytes()).await.unwrap();

            for mention in mentions_response{

                let mention_text = mention.text;

                /* cloning the redis pubsub mpsc sender to prevent from moving in each iteration before going into tokio spawn */
                let redis_pubsub_msg_sender = redis_pubsub_msg_sender.clone();

                
                /* 
                                -------------------------
                                THIS IS IMPORTANT TO KNOW
                                -------------------------

                    this tokio spawn will parse and send all the mentions asyncly to 
                    the downside of the redis pubsub mpsc channel, if we don't put 
                    the sending part intside the tokio spawn we'll face a blocking 
                    situation and will stuck in inside the for each mention loop.

                */
                tokio::spawn(async move{

                        /* EXTRA PARSING LOGICS HERE */
                        // ...

                        /* sending each mention text that satisfy all the above criteria to the mpsc channel asyncly */
                        let Ok(_) = redis_pubsub_msg_sender.clone().send(mention_text.clone()).await else{
                            panic!("couldn't send to redis pubsub mpsc channel");
                        };
                    
                });
                
            }

        }

    });




    // -------------------------------- setting up discord bot
    //
    // ---------------------------------------------------------------------------------------
    /* 
        we're using tokio event loop handler to activate the discord bot in such
        a way that once we received the flag from the mpsc channel inside the event
        loop, other branches will be canceled
    */
    discord_bot_flag_sender.send(true).await.unwrap(); /* set this to false if you don't want to start the bot */ 
    tokio::select!{
        bot_flag = discord_bot_flag_receiver.recv() => {
            if let Some(flag) = bot_flag{
                if flag == true{
                    info!("🏳️ receiving discord bot true flag");
                    daemon::activate_discord_bot(discord_token.as_str(), 
                                                serenity_shards.parse::<u64>().unwrap(), 
                                                redis_pubsubs_msg_receiver
                                            ).await; 
                }
            }    
        }
    }


    Ok(())




}