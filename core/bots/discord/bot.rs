





/*


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


since the underlying infrastructure in discord is based on ws
thus the event handler for parsing and handling the commands 
is a websocket handler

*/


use redis::FromRedisValue;
use redis::JsonAsyncCommands;
use redis::cluster::ClusterClient;
use redis::AsyncCommands; // this trait is required to be imported in here to call set() methods on the cluster connection
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
use tokio::sync::Mutex; // async Mutex will be used inside async methods since the trait Send is not implement for std::sync::Mutex
use hyper::{Client, Uri, Body};
use chrono::{TimeZone, Timelike, Datelike, Utc}; // this trait is rquired to be imported here to call the with_ymd_and_hms() method on a Utc object since every Utc object must be able to call the with_ymd_and_hms() method 
use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt, CpuExt, DiskExt}; // methods of trait DiskExt can be used on each Disk instance to get information of the disk because Disk struct has private methods and we can access them by call the trait DiskExt methods which has been implemented for the Disk struct  
use openai::{ // openai crate is using the reqwest lib under the hood
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole}
};
use openai::set_key;
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


#[path="gpt.rs"]
pub mod gpt;
pub mod daemon;
pub mod schemas;
pub mod handlers;
pub mod cmds;
pub mod tasks;




pub static GPT: Lazy<gpt::chat::Gpt> = Lazy::new(|| {
    block_on(gpt::chat::Gpt::new(None)) // this gets triggered once so it's ok to use block_on instead of asyn
});


pub static USER_RATELIMIT: Lazy<HashMap<u64, u64>> = Lazy::new(||{
    HashMap::new()
});

pub static GUILD_RATELIMIT: Lazy<HashMap<u64, u64>> = Lazy::new(||{
    HashMap::new()
});




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{



    env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();
    dotenv().expect("⚠️ .env file not found");
    let openai_key = env::var("OPENAI_KEY").expect("⚠️ no openai key variable set");
    let discord_token = env::var("DISCORD_TOKEN").expect("⚠️ no discord token variable set");
    let serenity_shards = env::var("SERENITY_SHARDS").expect("⚠️ no shards variable set");
    let io_buffer_size = env::var("IO_BUFFER_SIZE").expect("⚠️ no io buffer size variable set").parse::<u32>().unwrap() as usize; // usize is the minimum size in os which is 32 bits
    let (discord_bot_flag_sender, mut discord_bot_flag_receiver) = tokio::sync::mpsc::channel::<bool>(io_buffer_size); // reading or receiving from the mpsc channel is a mutable process cause we're receiving from the channel that must be written to the type
    set_key(openai_key);



    // -------------------------------- setting up discord bot
    //
    // ---------------------------------------------------------------------------------------
    // we're using tokio event loop handler to activate the discord bot in such
    //  a way that once we received the flag from the mpsc channel inside the event
    // loop, other branches will be canceled
    discord_bot_flag_sender.send(true).await.unwrap(); // set this to false if you don't want to start the bot
    tokio::select!{
        bot_flag = discord_bot_flag_receiver.recv() => {
            if let Some(flag) = bot_flag{
                if flag == true{
                    info!("🏳️ receiving discord bot true flag");
                    daemon::activate_discord_bot(discord_token.as_str(), 
                                                serenity_shards.parse::<u64>().unwrap(), 
                                                GPT.clone(), // GPT is of type Lazy<ctx::gpt::chat::Gpt> thus to get the Gpt instance we can clone the static type since clone returns the Self
                                                USER_RATELIMIT.clone(),
                                                GUILD_RATELIMIT.clone()
                                            ).await; 
                }
            }    
        }
    }


    Ok(())




}