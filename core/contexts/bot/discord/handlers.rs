






use std::{path::Component, fmt::format};

use redis::RedisResult;
use serenity::model::prelude::ReactionType;
use tokio::{io::AsyncWriteExt, fs::{OpenOptions, self}};

use crate::*;

// --------------------------------------------------------------------------------------
// ---------------- Arc<Mutex<Data>> FOR SHARING BETWEEN SHARDS' THREADS ----------------
// --------------------------------------------------------------------------------------
// inside the Value type we'll use a Mutex to mutate 
// the underlying data inside the Arc<RwLock<TypeKeyMap>> 
pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<tokio::sync::Mutex<ShardManager>>;
}

pub struct GptBot;
impl TypeMapKey for GptBot{
    type Value = Arc<async_std::sync::Mutex<gpt::chat::Gpt>>;
}

pub struct GuildRateLimit;
impl TypeMapKey for GuildRateLimit{
    type Value = Arc<async_std::sync::Mutex<HashMap<u64, u64>>>; // guild_id and total usage
}


#[derive(Serialize, Deserialize)] // serde traits are required to get data from redis
pub struct RateLimit;
impl TypeMapKey for RateLimit{
    type Value = Arc<async_std::sync::Mutex<HashMap<u64, u64>>>; // use async_std::sync::Mutex since it's faster that tokio::sync::Mutex
}


type CommandQueueSender = tokio::sync::mpsc::Sender<(Context, ApplicationCommandInteraction)>;
type CommandQueueReceiver = tokio::sync::mpsc::Receiver<(Context, ApplicationCommandInteraction)>;

// the discord bot commands and events listener/handler 
// for emitted events and webhooks over ws and http 
pub struct Handler{
    pub command_queue_sender: CommandQueueSender,
}

impl Handler{

    pub async fn new(command_queue_sender: CommandQueueSender) -> Self{
        Self{
            command_queue_sender,
        }
    }

    pub async fn send_ephemeral_bot_is_thinking(
        ctx: Context,
        interaction: &ApplicationCommandInteraction,
        content: &str
    ){
        let interaction_response = interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::DeferredChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .content("")
                            .flags(MessageFlags::EPHEMERAL)
                    }) // bot is thinking
            })
            .await;
    }

    pub async fn wait_for_user_reaction(
        ctx: &Context,
        message: &Message,
        user_id: UserId
    ) -> Result<(), SerenityError>{
        let reaction_type = ReactionType::Unicode("✅".to_string());
        let reaction_users = message.reaction_users(&ctx.http, reaction_type, None, None).await.unwrap();
        if !reaction_users.is_empty(){
            for user in reaction_users{
                if user.id.as_u64() == user_id.as_u64(){
                    break;
                } else{
                    continue;
                }
            }
        }

        Ok(())
    }

    // reading from the channel is a mutable process since we're mutating the 
    // state of the mpsc channel structure by receiving the data from the 
    // upside of the channel to store in the queue of the structure.
    pub async fn handle_interaction_command(mut command_queue_receiver: CommandQueueReceiver){
        // receiving each command from the upside of the channel 
        // to handle them asyncly inside the tokio green threadpool
        // to avoid discord rate limit and getting The application 
        // did not respond message when handling multiple command 
        // at the same time.
        tokio::spawn(async move{
            // waiting to receive every command asyncly to handle them 
            // asyncly and concurrently inside tokio green threadpool
            while let Some(command_data) = command_queue_receiver.recv().await{
                /*
                    to share data from the main function between threads and other methods we must 
                    build a context type that has all the setup structures and data inside of it to 
                    mutating them during the lifetime of the app in other scopes, threads and methods.
                */
                let ctx = command_data.0;
                let command = command_data.1;

                // ----------------------------------------------------------------------------------
                // --------------------- handling user command rate limit ---------------------------
                // ----------------------------------------------------------------------------------
                /*
                    since we have a rate limit checker that must do a checkup every 15 seconds 
                    thus it's better to put the entire command handling process inside the 
                    tokio::spawn() to check each incoming command from the upside of the channel 
                    asyncly for user rate limit usage.

                    since the flow of the code in here depends on the user rate limit result thus 
                    we must put the whole if and else block inside the tokio::spawn(async move{})
                    because of the async nature of the whole code, a user may sends two requests 
                    at a same time to here and before handling the rest of the code it might get
                    halted in handling the first one logic which causes to application didn't 
                    respond error from discord, thus these two if and else block logic must 
                    be handled asyncly.  
                */
                tokio::spawn(async move{ // spawning the rate limit checker async task to be solved in tokio green threadpool
                    if let Err(_) = Handler::check_rate_limit(&ctx, &command).await {
                        command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message
                                        .flags(MessageFlags::EPHEMERAL)
                                        .embed(|e|{ // param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                                            e.color(Colour::from_rgb(204, 0, 0));
                                            e.description("🥶 cooldown"); // cooldown for 15 seconds to bypass discord rate limit
                                            e.title("");
                                            e.footer(|f|{ // since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                                                f
                                                .text("")
                                            });
                                            return e;
                                        });
                                        message
                                    })
                            })
                            .await
                            .expect("Failed to send rate limit message");
                        return;
                    } else{

                        // ----------------------------------------------------------------------------------
                        // --------------- send the bot is thinking interaction response --------------------
                        // ----------------------------------------------------------------------------------
                        Handler::send_ephemeral_bot_is_thinking(ctx.clone(), &command, "").await;
                    
                        match command.data.name.as_str(){
                            "catchup" => {
                                let value = command
                                    .data
                                    .options
                                    .get(0)
                                    .and_then(|opt| opt.value.as_ref())
                                    .and_then(|val| val.as_i64())
                                    .unwrap_or(1); // default: fetch 1 hour ago
                                // --------------------------------------------------------
                                // -------------------- CATCHUP TASK ----------------------
                                // -------------------------------------------------------- 
                                // the following timestamp is approximate and may not exactly 
                                // match the time when the command was executed.
                                let channel_id = command.channel_id;
                                let interaction_response_message = channel_id
                                                                                .messages(&ctx.http, |retriever| retriever.limit(1))
                                                                                .await
                                                                                .unwrap()
                                                                                .into_iter()
                                                                                .next()
                                                                                .unwrap();
                                let interaction_response_message_id = interaction_response_message.id.0;
                                let init_cmd_time = command.id.created_at(); // id of the channel is a snowflake type that we can use it as the timestamp
                                let user_id = command.user.id;
                                let guild_id = command.guild_id.unwrap().0;
                                // spwaning the catchup task inside tokio green threadpool
                                // to be able to handle multiple commands at a same time 
                                // to avoid discord rate limit issue
                                tokio::spawn(async move{
                                    let response = tasks::catchup(&ctx, value as u32, channel_id, init_cmd_time, interaction_response_message_id, user_id.0, guild_id).await;
                                    // ----------------------------------------------------------------------------------------
                                    // --------------- editing interaction response since our task is done --------------------
                                    // ----------------------------------------------------------------------------------------
                                    // if the above task gets halted in a logic that doesn't have proper 
                                    // error handling we'll face the discord timeout which is the message 
                                    // inside the interaction response frame: The application did not respond
                                    let edited_interaction_response = command
                                        .edit_original_interaction_response(&ctx.http, |edit| {
                                            edit
                                                .embed(|e|{ // param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                                                    e.color(Colour::from_rgb(235, 204, 120));
                                                    e.description(response.0);
                                                    e.title(response.2);
                                                    e.footer(|f|{ // since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                                                        f
                                                        .text(response.1.as_str())
                                                    });
                                                    return e;
                                                })
                                                .components(|c|{
                                                    return c;
                                                });
                                                edit
                                        }) // edit the thinking message with the command response
                                        .await;    
                                });
                            },            
                            "help" => {
                                let footer = "".to_string();
                                let title = "".to_string();
                                let content = format!("**Examples**:\nGet a CatchUp for the past 2 hours : use `/catchup 2`\n");                                
                                let edited_interaction_response = command
                                    .edit_original_interaction_response(&ctx.http, |edit| {
                                        edit
                                            .embed(|e|{ // param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                                                e.color(Colour::from_rgb(235, 204, 120));
                                                e.description(content);
                                                e.title(title);
                                                e.footer(|f|{ // since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                                                    f
                                                    .text(footer)
                                                });
                                                return e;
                                            });
                                            edit
                                    }) // edit the thinking message with the command response
                                    .await;
                            },
                            "stats" => {
                                // ------------------------------------------------------
                                // -------------------- STATS TASK ----------------------
                                // ------------------------------------------------------ 
                                // the following timestamp is approximate and may not exactly 
                                // match the time when the command was executed.
                                let channel_id = command.channel_id;
                                let interaction_response_message = channel_id
                                                                                .messages(&ctx.http, |retriever| retriever.limit(1))
                                                                                .await
                                                                                .unwrap()
                                                                                .into_iter()
                                                                                .next()
                                                                                .unwrap();
                                let interaction_response_message_id = interaction_response_message.id.0;
                                let init_cmd_time = command.id.created_at(); // id of the channel is a snowflake type that we can use it as the timestamp
                                // spwaning the stats task inside tokio green threadpool
                                // to be able to handle multiple commands at a same time 
                                // to avoid discord rate limit issue
                                tokio::spawn(async move{
                                    let guild_ids = ctx.cache.guilds();
                                    let mut pretty_names = String::from("");
                                    let server_names = guild_ids
                                                            .into_iter()
                                                            .map(|g| {
                                                                let g_name = g.name(&ctx.cache).unwrap();
                                                                let pretty = format!("- {},\n", g_name.clone());
                                                                pretty_names.push_str(pretty.as_str());
                                                                g_name
                                                            })
                                                            .collect::<Vec<String>>();
                                    let server_name_json_string = serde_json::to_string_pretty(&server_names).unwrap();
                                    let response = tasks::stats(&ctx, channel_id, init_cmd_time, interaction_response_message_id).await;
                                    let description = format!("🥞 **{}** servers\n {}\n{}", server_names.len(), pretty_names, response.0);
                                    let edited_interaction_response = command
                                        .edit_original_interaction_response(&ctx.http, |edit| {
                                            edit
                                                .embed(|e|{ // param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                                                    e.color(Colour::from_rgb(235, 204, 120));
                                                    e.description(description);
                                                    e.title(response.2);
                                                    e.footer(|f|{ // since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                                                        f
                                                        .text(response.1.as_str())
                                                    });
                                                    return e;
                                                });
                                                edit
                                        }) // edit the thinking message with the command response
                                        .await; 
                                }); 
                            },
                            _ => {
                                let footer = "".to_string();
                                let title = "".to_string();
                                let content = format!("**Uknown Command**");
                                if let Err(why) = command
                                    .edit_original_interaction_response(&ctx.http, |edit| {
                                        edit
                                            .allowed_mentions(|mentions| mentions.replied_user(true))
                                            .embed(|e|{ // param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                                                e.color(Colour::from_rgb(204, 0, 0));
                                                e.description(content);
                                                e.title(title);
                                                e.footer(|f|{ // since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                                                    f
                                                    .text(footer)
                                                });
                                                return e;
                                            });
                                            edit
                                    }) // edit the thinking message with the command response
                                    .await
                                {
                                    error!("error editing original interaction response since {:#?}", why);
                                }
                            }
                        }
                    }
                });
            }
        });
    }

    async fn check_rate_limit(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), ()>{
        
        /* -=-=-=-=-=-=-=-=-=-=-= REDIS SETUP -=-=-=-=-=-=-=-=-=-=-= */
        /* making a new connection once the `check_rate_limit` method gets called */
        
        let redis_password = env::var("REDIS_PASSWORD").unwrap_or("".to_string());
        let redis_username = env::var("REDIS_USERNAME").unwrap_or("".to_string());
        let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
        let redis_port = std::env::var("REDIS_PORT").unwrap_or("6379".to_string()).parse::<u64>().unwrap();

        let redis_conn_url = if !redis_password.is_empty(){
            format!("redis://:{}@{}:{}", redis_password, redis_host, redis_port)
        } else if !redis_password.is_empty() && !redis_username.is_empty(){
            format!("redis://{}:{}@{}:{}", redis_username, redis_password, redis_host, redis_port)
        } else{
            format!("redis://{}:{}", redis_host, redis_port)
        };

        let client = redis::Client::open(redis_conn_url.as_str()).unwrap();
        let mut connection = client.get_async_connection().await.unwrap();

        /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */


        // --------------------------------------------------------------------------
        // ---------------------------- RATE LIMIT LOGIC ----------------------------
        // --------------------------------------------------------------------------
        /*
        
            data inside the bot client must be safe to be shared between event and 
            command handlers' threads thus they must be of type Arc<RwLock<TypeMapKey>>
            in which TypeMapKey is a trait that has implemented for the underlying data 
            which is of type Arc<Mutex<Data>> acquiring a write lock will block other 
            event and command handlers which don't allow them to use the data until 
            the lock is released.

            IDEA: we can store the last usage timestamp of the user that 
            has called this command inside a Arc<Mutex<HashMap<u64, u64>>> 
            then use that inside the command handler threads
            to check the rate limit since we can send 50 messages 
            per second per server (guild) and up to 5 messages per second 
            per user in a direct message (DM) channel. For slash commands, 
            you can have a maximum of 100 commands per 60 seconds per application
            means that sending like 5 messages in less that 5 seconds 
            makes discord angry :) also async_std Mutex is faster than tokio Mutex
            thus we won't face the timeout issue of the discord while we're locking 
            the mutex to acquire the underlying data.
        
            ctx.data in order to be shared between tokio threads and shards must be 
            static, sync and send or Arc also it must be safe to mutate it in other 
            threads means it must be inside the mutex or rwlock to aqcuire the lock 
            in other scopes and methods safely.

            ctx.data in order to be shared between clusters and app instances must be 
            stored in redis which can be published to the redis servers so other 
            subscribers can subcribe to it once the redis server sent the broadcasted 
            message or publish the topic contains the data.

        */
        let chill_zone_duration = 15_000u64; // 15 seconds rate limit
        let user_id = command.user.id.0;
        let now = chrono::Local::now().timestamp_millis() as u64;
        let mut is_rate_limited = false;
        

        {
            // ------------------------------------------------------------------------
            // -------------------- reading from ctx.data to redis --------------------
            // ------------------------------------------------------------------------
            // reading the mutexed data to acquire the lock on the ctx.data
            let data = ctx.data.read().await; // writing safely to the RateLimit instance also write lock returns a mutable reference to the underlying map instance also data is of type Arc<RwLock<TypeMapKey>>
            let rate_limit_data = data.get::<handlers::RateLimit>().unwrap();
            let mut rate_limiter_mutexed = rate_limit_data.lock().await;

            // ------------------------------------------------------------
            // -------------------- reading from redis --------------------
            // ------------------------------------------------------------
            let redis_result_rate_limiter: RedisResult<String> = connection.get("rate_limiter").await;
            let mut redis_rate_limiter = match redis_result_rate_limiter{
                Ok(data) => {
                    let rl_data = serde_json::from_str::<HashMap<u64, u64>>(data.as_str()).unwrap();
                    rl_data
                },
                Err(e) => {
                    let empty_rate_limiter = HashMap::<u64, u64>::new();
                    let rl_data = serde_json::to_string(&empty_rate_limiter).unwrap();
                    let _: () = connection.set("rate_limiter", rl_data).await.unwrap();
                    let log_name = format!("[{}]", chrono::Local::now());
                    let filepath = format!("logs/error-kind/{}-ratelimit-redis-log-file.log", log_name);
                    let mut error_kind_log = tokio::fs::File::create(filepath.as_str()).await.unwrap();
                    error_kind_log.write_all(e.to_string().as_bytes()).await.unwrap();
                    HashMap::new()
                }
            };

            if let Some(last_used) = redis_rate_limiter.get(&user_id){
                if now - *last_used < chill_zone_duration{
                    is_rate_limited = true;
                }
            }

            if !is_rate_limited{
                
                // -------------------------------------------------------------
                // -------------------- writing to ctx.data  --------------------
                // -------------------------------------------------------------
                // this will be used to handle shared state between shards
                rate_limiter_mutexed.insert(user_id, now);
                
                // ----------------------------------------------------------
                // -------------------- writing to redis --------------------
                // ----------------------------------------------------------
                // this will be used to handle shared state between clusters
                redis_rate_limiter.insert(user_id, now); // updating the redis rate limiter map
                let rl_data = serde_json::to_string(&redis_rate_limiter).unwrap();
                let _: () = connection.set("rate_limiter", rl_data).await.unwrap(); // writing to redis ram

                // -------------------------------------------------
                // -------------------- logging --------------------
                // -------------------------------------------------
                let filepath = format!("logs/rate-limiter/usage.log");
                let log_content = format!("userId:{}|lastUsage:{}\n", user_id, now);
                let mut ratelimit_log; 

                match fs::metadata("logs/rate-limiter/usage.log").await {
                    Ok(_) => { // if the file was there then append to it
                        let mut file = OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(filepath.as_str())
                            .await.unwrap();
                        file.write_all(log_content.as_bytes()).await.unwrap(); // Write the data to the file
                    },
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => { // if the file wasn't there then create a new one
                        ratelimit_log = tokio::fs::File::create(filepath.as_str()).await.unwrap();
                        ratelimit_log.write_all(log_content.as_bytes()).await.unwrap();
                    },
                    Err(e) => {
                        let log_name = format!("[{}]", chrono::Local::now());
                        let filepath = format!("logs/error-kind/{}-ratelimit-reading-log-file.log", log_name);
                        let mut error_kind_log = tokio::fs::File::create(filepath.as_str()).await.unwrap();
                        error_kind_log.write_all(e.to_string().as_bytes()).await.unwrap();
                    }
                }
            }

        }
        
        if is_rate_limited{
            Err(())
        } else{
            Ok(())
        }
        
        // -----------------------------------------------------------------------------
        // -----------------------------------------------------------------------------
        // -----------------------------------------------------------------------------
    }

}


// following we're implementing the EventHandler trait
// for the Handler struct to handle all the bot events
// which will be fired or emitted through the discrod ws
// server thus in here we're subscribing to those events. 
#[async_trait]
impl EventHandler for Handler{
    /*
                        --------------------
                        ABOUT ctx.data FIELD
                        -------------------- 
        data field in hyper and serenity are atomic types that can be 
        shread between shards' and other threads safely is of type 
        Arc<RwLock<TypeMapKey>> in which TypeMapKey::Value can 
        be of type Arc<Mutex<Data>> and if we want to update the
        type inside the data field we call write() method on the data
        to acquire the lock on the type which during the lock acquisition
        other event handlers remain block until the lock gets released
        also it must be bounded to Sync and Send traits to be safe and
        cloneable to be shared between threads using tokio channels.  

        in Serenity, when handling an interaction_create event, the Interaction 
        object does not directly contain the message instance. The reason is 
        that slash commands can be used without being tied to a specific message.

        in bot design process there must be a ctx type that can be passed to other 
        handlers and used to access whole methods and bot setup functions 
        like each ws shard inside each event handler.



        ┏———————————————————————————————┓
           INTERACTION CREATION HANDLER
        ┗———————————————————————————————┛


    */
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {

        if let Interaction::ApplicationCommand(command) = interaction.clone() {
            /*
                sending the incoming slash commands to the downside 
                of the mpsc channel to handle them asyncly to avoid
                deadlocking and rate limiting by calling the 
                command_queue_sender field of the event handler struct
            */
            self.command_queue_sender.send((ctx, command)).await; // don't unwrap() since Context doesn't implement Debug trait
        }
    }

    /*
        
        ┏———————————————————┓
           READY HANDLER
        ┗———————————————————┛

    */

    async fn ready(&self, ctx: Context, ready: Ready){ // handling ready events, once the bot shards gets ready 
        if let Some(shard) = ready.shard{ // shard is an slice array of 2 elements, 8 bytes length each as the shard id
            info!("🔗 {} bot is connected on shard id {}/{}", ready.user.name, shard[0], shard[1]);
            
            let guilds = ctx.cache.guilds(); // getting all the guild that the bot is inside of them
            for guild_id in guilds{
                let channels_result = guild_id.channels(&ctx.http).await;
                if let Ok(channels) = channels_result{
                    for (cid, gc) in channels{
                        let id = cid.0;
                        let channel_id = ChannelId(id);
                        let initial_message = "Okay, I just woke up :/";
                        channel_id.send_message(&ctx.http, |m|{
                            m
                                .allowed_mentions(|mentions| mentions.replied_user(true))
                                .embed(|e|{ // param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                                    e.color(Colour::from_rgb(235, 204, 120));
                                    e.description(initial_message);
                                    e.title("");
                                    e.footer(|f|{ // since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                                        f
                                        .text("")
                                    });
                                    return e;
                                });
                                m
                            }) // edit the thinking message with the command response
                            .await
                            .unwrap();
                    } 
                }
            }
        }

        // -------------------------------------------------
        // --------- REGISTERING GLOBAL COMMANDS -----------
        // -------------------------------------------------
        // registering global commands for each 
        // guild that this bot is added to
        let _ = Command::create_global_application_command(&ctx.http, |command| {
            cmds::slash::catchup_register(command)
        })
        .await;

        let _ = Command::create_global_application_command(&ctx.http, |command| {
            cmds::slash::help_register(command)
        })
        .await;

        let _ = Command::create_global_application_command(&ctx.http, |command| {
            cmds::slash::stats_register(command)
        })
        .await;

    }


    /*
        
        ┏———————————————————┓
           MESSAGE HANDLER
        ┗———————————————————┛

    */

    async fn message(&self, ctx: Context, msg: Message){ // handling the message event
        // ctx is the instance that contains 
        // the methods and types of the whole
        // setup bot. 
    }



    /*
        
        ┏———————————————————┓
           RESUME HANDLER
        ┗———————————————————┛

    */

    async fn resume(&self, _: Context, _: ResumedEvent){
        info!("▶ Resumed");
    }

}