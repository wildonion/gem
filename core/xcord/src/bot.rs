

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



========================
DISCORD BOT ARCHITECTURE
========================
- rustls and native_tls to communicate with secured socket discord server
- tokio::spawn(async move{}) : handle async event in concurrent manner 
- tokio::select!{}           : eventloop to listen and subs to incoming events
- tokio::sync::mpsc          : jobq channel to share Arc<Mutex<SharedStateData>> between tokio green threads
 

discord client app ----------------------------------------------------------- send response to the client ---------------------------------------------------------------------------------------------
    |                                                                                                                                                                                                   |             
     ---------------- trigger / commands of a bot                                                                                                                                                       |
                            |                                                                                                                                                                           |
                             --------ws channel-------- send requests to the discord ws server to fire the / event                                                                                      |
                                                                    |                                                                                                                                   | 
                                                                     -------ws client channel--------                                                                                                   |
                                                                                                    |                                                                                                   |
                                                                        tokio::select!{} eventloop inside the bot code catches the fired / event through                                                |
                                                                        ws client which is connected to the discord ws server                                                                           |
                                                                                                    |                                                                                                   | 
                                                                                                    -------------- tokio::spawn(async move{handle the / event inside the bot code})                     |     
                                                                                                                                                |                                                       |
                                                                                                                                                ------------ send response back to the discord ws server 


since the underlying infrastructure of discord bot is based on ws
thus the event handler for parsing and handling the slash commands 
is a websocket handler

*/


use futures_util::StreamExt;
use misc::NewTask;
use redis::FromRedisValue;
use redis::JsonAsyncCommands;
use redis::cluster::ClusterClient;
use redis::AsyncCommands; // this trait is required to be imported in here to call set() methods on the cluster connection
use redis::RedisResult;
use redis_async::resp::FromResp;
use serde::{Serialize, Deserialize};
use std::{rc::Rc, cell::RefCell};
use std::collections::{HashSet, HashMap};
use std::{net::SocketAddr, sync::Arc, env};
use std::time::Duration;
use dotenv::dotenv;
use uuid::Uuid;
use log::{info, error};
use once_cell::sync::Lazy;
use futures::executor::block_on;
use tokio::sync::oneshot;
use tokio::sync::Mutex; // async Mutex will be used inside async methods since the trait Send is not implement for std::sync::Mutex
use chrono::{TimeZone, Timelike, Datelike, Utc}; // this trait is rquired to be imported here to call the with_ymd_and_hms() method on a Utc object since every Utc object must be able to call the with_ymd_and_hms() method 
use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt, CpuExt, DiskExt}; // methods of trait DiskExt can be used on each Disk instance to get information of the disk because Disk struct has private methods and we can access them by call the trait DiskExt methods which has been implemented for the Disk struct  
use redis_async::client::ConnectionBuilder;
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



mod misc;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{

    env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();
    dotenv().expect("⚠️ .env file not found");
    let discord_token = env::var("XCORD_TOKEN").expect("⚠️ no discord token variable set");
    let serenity_shards = env::var("SERENITY_SHARDS").expect("⚠️ no shards variable set");
    let buffer_size = env::var("IO_BUFFER_SIZE").unwrap().parse::<usize>().unwrap();
    let redis_password = env::var("REDIS_PASSWORD").unwrap_or("".to_string());
    let redis_username = env::var("REDIS_USERNAME").unwrap_or("".to_string());
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let redis_port = std::env::var("REDIS_PORT").unwrap_or("6379".to_string()).parse::<u64>().unwrap();
    let mut redis_conn_builder = ConnectionBuilder::new(redis_host, redis_port as u16).unwrap();
    redis_conn_builder.password(redis_password);
    let redis_async_pubsub_connection = redis_conn_builder.pubsub_connect().await.unwrap();
    

    let (discord_bot_flag_sender, mut discord_bot_flag_receiver) = 
        tokio::sync::mpsc::channel::<bool>(buffer_size);
    let (new_task_sender, mut new_task_receiver) = 
        tokio::sync::mpsc::channel::<NewTask>(buffer_size);


    /* sending true flag to start the bot, this can be done any where inside the app using the mpsc sender */
    if let Err(why) = discord_bot_flag_sender.send(true).await{
        error!("can't send true flag to start the bot because: {}", why);
    }


    /* 
        start subscribing in a separate threadpool rather than using the main's threads 
        which is the default tokio runtime thread cause we have tokio::main macro on 
        top of the main function which means this method will be executed inside tokio
        runtime
    */
    tokio::spawn(async move{

        let cloned_sender = new_task_sender.clone();

        loop{

            /*  
                subscribing asyncly and constantly using async redis crate to 
                XTASK redis pubsub channel by streaming over the incoming future 
                tasks topics to decode the published topics
            */
            let get_stream_messages = redis_async_pubsub_connection
                .subscribe(misc::TASK_TOPIC_CHANNEL)
                .await;

            let Ok(mut pubsubstreamer) = get_stream_messages else{
                panic!("can't get pubsub stream");
            };

            while let Some(message) = pubsubstreamer.next().await{

                let resp_val = message.unwrap();
                let json_stringified_new_task = String::from_resp(resp_val).unwrap();
                let new_task_data = serde_json::from_str::<NewTask>(&json_stringified_new_task).unwrap();
                
                if let Err(why) = cloned_sender.send(new_task_data.clone()).await{
                    error!("can't send into mpsc channel cause {}", why.to_string());
                }

            }
             
        }

    });


    /* 
        we're using tokio event loop handler to activate the discord bot in such
        a way that once we received the flag from the mpsc channel inside the event
        loop, other branches will be cancelled
    */
    tokio::select!{
        bot_flag = discord_bot_flag_receiver.recv() => {
            if let Some(flag) = bot_flag{
                if flag == true{
                    info!("🏳️ receiving discord bot true flag in tokio event loop");
                    misc::daemon::activate_bot(
                        discord_token.as_str(), 
                        serenity_shards.parse::<u64>().unwrap(), 
                        new_task_receiver
                    ).await; 
                }
            }    
        }
    }


    Ok(())


}