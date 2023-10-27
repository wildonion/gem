

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
use uuid::Uuid;
use log::{info, error};
use once_cell::sync::Lazy;
use futures::executor::block_on;
use tokio::sync::oneshot;
use tokio::sync::Mutex; // async Mutex will be used inside async methods since the trait Send is not implement for std::sync::Mutex
use chrono::{TimeZone, Timelike, Datelike, Utc}; // this trait is rquired to be imported here to call the with_ymd_and_hms() method on a Utc object since every Utc object must be able to call the with_ymd_and_hms() method 
use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt, CpuExt, DiskExt}; // methods of trait DiskExt can be used on each Disk instance to get information of the disk because Disk struct has private methods and we can access them by call the trait DiskExt methods which has been implemented for the Disk struct  
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


/* a global mutex that must be in RwLock in order to be mutable safely in threadpool */
pub static USER_RATELIMIT: Lazy<HashMap<u64, u64>> = Lazy::new(||{
    HashMap::new()
});





#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{


    /* start subscribing in a separate threadpool rather than using the main's threads */
    tokio::spawn(async move{

        loop{

            // remember to publish tasks in Task::insert() method

            // subscribe to redis pubsub channel
            // send the subscribed data to mpsc jobq channel

            // inside misc.rs in discord task handler we'll
            // receive the data from mpsc jobq channel and 
            // broadcast it to discord channel using serenity ws
             
        }

    });


    Ok(())
}