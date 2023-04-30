





use crate::*;

//// --------------------------------------------------------------------------------------
//// ---------------- Arc<Mutex<Data>> FOR SHARING BETWEEN SHARDS' THREADS ----------------
//// --------------------------------------------------------------------------------------
//// inside the Value type we'll use a Mutex to mutate 
//// the underlying data inside the Arc<RwLock<TypeKeyMap>> 
pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<tokio::sync::Mutex<ShardManager>>;
}

pub struct GptBot;
impl TypeMapKey for GptBot{
    type Value = Arc<async_std::sync::Mutex<gpt::chat::Gpt>>;
}

pub struct RateLimit;
impl TypeMapKey for RateLimit{
    type Value = Arc<async_std::sync::Mutex<HashMap<u64, u64>>>; //// use async_std::sync::Mutex since it's faster that tokio::sync::Mutex
}



type CommandQueueSender = tokio::sync::mpsc::Sender<(Context, ApplicationCommandInteraction)>;
type CommandQueueReceiver = tokio::sync::mpsc::Receiver<(Context, ApplicationCommandInteraction)>;
//// the discord bot commands and events listener/handler for emitted events and webhooks over ws and http 
pub struct Handler{
    pub command_queue_sender: CommandQueueSender,
}

impl Handler{

    pub async fn new(command_queue_sender: CommandQueueSender) -> Self{
        Self{
            command_queue_sender,
        }
    }

    //// reading from the channel is a mutable process since we're mutating the 
    //// state of the mpsc channel structure by receiving the data from the 
    //// upside of the channel to store in the queue of the structure.
    pub async fn handle_interaction_command(mut command_queue_receiver: CommandQueueReceiver){
        //// receiving each command from the upside of the channel 
        //// to handle them asyncly inside the tokio green threadpool
        //// to avoid discord rate limit and getting The application did not respond message
        tokio::spawn(async move{
            while let Some(command_data) = command_queue_receiver.recv().await{
            
                let ctx = command_data.0;
                let command = command_data.1;

                // ----------------------------------------------------------------------------------
                // --------------- send the bot is thinking interaction response --------------------
                // ----------------------------------------------------------------------------------
                let interaction_response = command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::DeferredChannelMessageWithSource)
                            .interaction_response_data(|message| message.content("")) //// bot is thinking
                    })
                    .await;

                // ---------------------------------------------------------
                // --------------- rate limit handler ----------------------
                // ---------------------------------------------------------
                // it's not needed to uncommnet it since we must handle each
                // command as a separate async task coming from the jobq channel
                // inside the tokio green threadpool to avoid discord rate limit
                // Handler::handle_rate_limited_command(ctx.clone(), interaction.clone()).await;
                
                // ----------------------------------------------------------------
                // --------------- handling slash commands ------------------------
                // ----------------------------------------------------------------
                match command.data.name.as_str() {
                    "catchup" => {
                        let value = command
                            .data
                            .options
                            .get(0)
                            .and_then(|opt| opt.value.as_ref())
                            .and_then(|val| val.as_i64())
                            .unwrap_or(1); //// default: fetch 1 hour ago
                        //// --------------------------------------------------------
                        //// -------------------- CATCHUP TASK ----------------------
                        //// -------------------------------------------------------- 
                        //// the following timestamp is approximate and may not exactly 
                        //// match the time when the command was executed.
                        let channel_id = command.channel_id;
                        let interaction_response_message = channel_id
                                                                        .messages(&ctx.http, |retriever| retriever.limit(1))
                                                                        .await
                                                                        .unwrap()
                                                                        .into_iter()
                                                                        .next()
                                                                        .unwrap();
                        let interaction_response_message_id = interaction_response_message.id.0;
                        let init_cmd_time = command.id.created_at(); //// id of the channel is a snowflake type that we can use it as the timestamp
                        let user_id = command.user.id.0;
                        let guild_id = command.guild_id.unwrap().0;
                        //// spwaning the catchup task inside tokio green threadpool
                        //// to be able to handle multiple commands at a same time 
                        //// to avoid discord rate limit issue
                        tokio::spawn(async move{
                            let response = tasks::catchup(&ctx, value as u32, channel_id, init_cmd_time, interaction_response_message_id, user_id, guild_id).await;
                            // ----------------------------------------------------------------------------------------
                            // --------------- editing interaction response since our task is done --------------------
                            // ----------------------------------------------------------------------------------------
                            //// if the above task gets halted in a logic that doesn't have proper 
                            //// error handling we'll face the discord timeout which is the message 
                            //// inside the interaction response frame: The application did not respond
                            if let Err(why) = command
                                .edit_original_interaction_response(&ctx.http, |edit| {
                                    edit
                                        .allowed_mentions(|mentions| mentions.replied_user(true))
                                        .embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                                            e.color(Colour::from_rgb(235, 204, 120));
                                            e.description(response.0);
                                            e.title(response.2);
                                            e.footer(|f|{ //// since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                                                f
                                                .text(response.1.as_str())
                                            });
                                            return e;
                                        });
                                        edit
                                }) //// edit the thinking message with the command response
                                .await
                            {
                                error!("error editing original interaction response since {:#?}", why);
                            }
                        });
                    },
                    "stats" => {
                        //// --------------------------------------------------------
                        //// -------------------- STATS TASK -----------------------
                        //// -------------------------------------------------------- 
                        //// the following timestamp is approximate and may not exactly 
                        //// match the time when the command was executed.
                        let channel_id = command.channel_id;
                        let interaction_response_message = channel_id
                                                                        .messages(&ctx.http, |retriever| retriever.limit(1))
                                                                        .await
                                                                        .unwrap()
                                                                        .into_iter()
                                                                        .next()
                                                                        .unwrap();
                        let interaction_response_message_id = interaction_response_message.id.0;
                        let init_cmd_time = command.id.created_at(); //// id of the channel is a snowflake type that we can use it as the timestamp
                        //// spwaning the stats task inside tokio green threadpool
                        //// to be able to handle multiple commands at a same time 
                        //// to avoid discord rate limit issue
                        tokio::spawn(async move{
                            let response = tasks::stats(&ctx, channel_id, init_cmd_time, interaction_response_message_id).await;
                            // ----------------------------------------------------------------------------------------
                            // --------------- editing interaction response since our task is done --------------------
                            // ----------------------------------------------------------------------------------------
                            //// if the above task gets halted in a logic that doesn't have proper 
                            //// error handling we'll face the discord timeout which is the message 
                            //// inside the interaction response frame: The application did not respond
                            if let Err(why) = command
                                .edit_original_interaction_response(&ctx.http, |edit| {
                                    edit
                                        .allowed_mentions(|mentions| mentions.replied_user(true))
                                        .embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                                            e.color(Colour::from_rgb(235, 204, 120));
                                            e.description(response.0);
                                            e.title(response.2);
                                            e.footer(|f|{ //// since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                                                f
                                                .text(response.1.as_str())
                                            });
                                            return e;
                                        });
                                        edit
                                }) //// edit the thinking message with the command response
                                .await
                            {
                                error!("error editing original interaction response since {:#?}", why);
                            }
                        });
                    },
                    "help" => {
                        // ----------------------------------------------------------------------------------------
                        // --------------- editing interaction response since our task is done --------------------
                        // ----------------------------------------------------------------------------------------
                        //// if the above task gets halted in a logic that doesn't have proper 
                        //// error handling we'll face the discord timeout which is the message 
                        //// inside the interaction response frame: The application did not respond
                        let footer = "".to_string();
                        let title = "".to_string();
                        let content = format!("**Examples**:\nGet a CatchUp for the past 2 hours : use `/catchup 2`\n");
                        if let Err(why) = command
                            .edit_original_interaction_response(&ctx.http, |edit| {
                                edit
                                    .allowed_mentions(|mentions| mentions.replied_user(true))
                                    .embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                                        e.color(Colour::from_rgb(235, 204, 120));
                                        e.description(content);
                                        e.title(title);
                                        e.footer(|f|{ //// since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                                            f
                                            .text(footer)
                                        });
                                        return e;
                                    });
                                    edit
                            }) //// edit the thinking message with the command response
                            .await
                        {
                            error!("error editing original interaction response since {:#?}", why);
                        }
                    },
                    _ => {
                        // ----------------------------------------------------------------------------------------
                        // --------------- editing interaction response since our task is done --------------------
                        // ----------------------------------------------------------------------------------------
                        //// if the above task gets halted in a logic that doesn't have proper 
                        //// error handling we'll face the discord timeout which is the message 
                        //// inside the interaction response frame: The application did not respond
                        let footer = "".to_string();
                        let title = "".to_string();
                        let content = format!("**Uknown Command**");
                        if let Err(why) = command
                            .edit_original_interaction_response(&ctx.http, |edit| {
                                edit
                                    .allowed_mentions(|mentions| mentions.replied_user(true))
                                    .embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                                        e.color(Colour::from_rgb(235, 204, 120));
                                        e.description(content);
                                        e.title(title);
                                        e.footer(|f|{ //// since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                                            f
                                            .text(footer)
                                        });
                                        return e;
                                    });
                                    edit
                            }) //// edit the thinking message with the command response
                            .await
                        {
                            error!("error editing original interaction response since {:#?}", why);
                        }
                    }
                }
            }
      });
    }

    async fn handle_rate_limited_command(ctx: Context, interaction: Interaction){

            if let Interaction::ApplicationCommand(command) = interaction.clone() {

                ///// --------------------------------------------------------------------------
                ///// ---------------------------- RATE LIMIT LOGIC ----------------------------
                ///// --------------------------------------------------------------------------
                //// data inside the bot client must be safe to be shared between event and 
                //// command handlers' threads thus they must be of type Arc<RwLock<TypeMapKey>>
                //// in which TypeMapKey is a trait that has implemented for the underlying data 
                //// which is of type Arc<Mutex<Data>> acquiring a write lock will block other 
                //// event and command handlers which don't allow them to use the data until 
                //// the lock is released.

                let mut data = ctx.data.write().await; //// writing safely to the RateLimit instance also write lock returns a mutable reference to the underlying map instance also data is of type Arc<RwLock<TypeMapKey>>
                match data.get_mut::<handlers::RateLimit>(){ //// getting a mutable reference to the underlying data of the Arc<RwLock<TypeMapKey>> the RateLimit structure
                    Some(rate_limiter) => {
                        
                        // IDEA: we can store the last usage timestamp of the user that 
                        // has called this command inside a Arc<Mutex<HashMap<u64, u64>>> 
                        // then use that inside the command handler threads
                        // to check the rate limit since we can send 50 messages 
                        // per second per server (guild) and up to 5 messages per second 
                        // per user in a direct message (DM) channel. For slash commands, 
                        // you can have a maximum of 100 commands per 60 seconds per application
                        // means that sending like 5 messages in less that 5 seconds 
                        // makes discord angry :) also async_std Mutex is faster than tokio Mutex
                        // thus we won't face the timeout issue of the discord while we're locking 
                        // the mutex to acquire the underlying data.

                        let mut rate_limiter = rate_limiter.lock().await;
                        let chill_zone_duration = 10_000u64; //// 10 seconds rate limit
                        let user_id = command.user.id.0;
                        let now = chrono::Local::now().timestamp_millis() as u64;
                        let mut is_rate_limited = false;

                        if let Some(last_used) = rate_limiter.get(&user_id){
                            if now - *last_used < chill_zone_duration{ //// check that if the time elapsed since the last usage of this command is less than the rate limit 
                                is_rate_limited = true;
                            }
                        } else{
                            rate_limiter.insert(user_id, now);
                        }
                        
                        if is_rate_limited{
                            if let Err(why) = command.channel_id
                                .send_message(&ctx.http, |message| {
                                    message.allowed_mentions(|mentions| mentions.replied_user(true))
                                    .embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                                        e.color(Colour::from_rgb(235, 204, 120));
                                        e.description("**🤕 You broke me, entering chill zone!**");
                                        return e;
                                    });
                                    message
                                })
                            .await{
                                error!("can't send chill zone message since {:#?}", why);
                            }
                        }
                    },
                    None => {
                        if let Err(why) = command.channel_id
                                .send_message(&ctx.http, |message| {
                                    message.allowed_mentions(|mentions| mentions.replied_user(true))
                                    .embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                                        e.color(Colour::from_rgb(235, 204, 120));
                                        e.description("**rate limiter is not available!**");
                                        return e;
                                    });
                                    message
                                })
                            .await{
                                error!("can't send message since {:#?}", why);
                            }
                    },
                }
                ///// -----------------------------------------------------------------------------
                ///// -----------------------------------------------------------------------------
                ///// -----------------------------------------------------------------------------
            }
        // });
    }

}


//// following we're implementing the EventHandler trait
//// for the Handler struct to handle all the bot events
//// which will be fired or emitted through the discrod ws
//// server thus in here we're subscribing to those events. 
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

        in bot design there must be a ctx type that can be passed to other 
        handlers and used to access whole methods and bot setup functions 
        like each ws shard inside each event handler.



        ┏———————————————————————————————┓
           INTERACTION CREATION HANDLER
        ┗———————————————————————————————┛


    */
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {

        if let Interaction::ApplicationCommand(command) = interaction.clone() {
            self.command_queue_sender.send((ctx, command)).await;
        }
    }

    /*
        
        ┏———————————————————┓
           READY HANDLER
        ┗———————————————————┛

    */

    async fn ready(&self, ctx: Context, ready: Ready){ //// handling ready events, once the bot shards gets ready 
        if let Some(shard) = ready.shard{ //// shard is an slice array of 2 elements, 8 bytes length each as the shard id
            info!("🔗 {} bot is connected on shard id {}/{}", ready.user.name, shard[0], shard[1]);
        }

        //// -------------------------------------------------
        //// --------- REGISTERING GLOBAL COMMANDS -----------
        //// -------------------------------------------------
        //// registering global commands for each 
        //// guild that this bot is added to
        let _ = Command::create_global_application_command(&ctx.http, |command| {
            cmds::slash::catchup_register(command)
        })
        .await;

        let _ = Command::create_global_application_command(&ctx.http, |command| {
            cmds::slash::help_register(command)
        })
        .await;

    }


    /*
        
        ┏———————————————————┓
           MESSAGE HANDLER
        ┗———————————————————┛

    */

    async fn message(&self, ctx: Context, msg: Message){ //// handling the message event
        //// ctx is the instance that contains 
        //// the methods and types of the whole
        //// setup bot. 
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