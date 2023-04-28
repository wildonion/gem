


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
    type Value = Arc<tokio::sync::Mutex<gpt::chat::Gpt>>;
}
pub struct Storage;
impl TypeMapKey for Storage{
    type Value = Arc<async_std::sync::Mutex<mongodb::Client>>;
}

pub struct RateLimit;
impl TypeMapKey for RateLimit{
    type Value = Arc<async_std::sync::Mutex<HashMap<u64, u64>>>; //// use async_std::sync::Mutex since it's faster that tokio::sync::Mutex
}



pub struct Handler; //// the discord bot commands and events listener/handler for emitted events and webhooks over ws and http 


impl Handler{

    async fn handle_rate_limited_command(&self, ctx: Context, interaction: Interaction){

        //// spawning a separate task per each command to handle 
        //// each command asyncly inside tokio green threadpool
        tokio::spawn(async move{
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
                        let chill_zone_duration = 10_000u64;
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
                                        e.description("**🤕 You broke me, entering chill zone!**");
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
        });
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



        ┏———————————————————————┓
           INTERACTION HANDLER
        ┗———————————————————————┛


    */
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction.clone() {

            self.handle_rate_limited_command(ctx.clone(), interaction.clone()).await;

            //// oneshot channels are not async because there
            //// is only one value can be sent at a time to 
            //// the downside of the channel
            let (wrapup_sender, mut wrapup_receiver) = oneshot::channel::<i64>(); //// reading from the wrapup channel is a mutable process
            let (expand_sender, mut expand_receiver) = oneshot::channel::<i64>(); //// reading from the expand channel is a mutable process
            let (stats_sender, mut stats_receiver) = oneshot::channel::<i64>();
            let response_content = match command.data.name.as_str() {
                "wrapup" => {
                    let value = command
                        .data
                        .options
                        .get(0)
                        .and_then(|opt| opt.value.as_ref())
                        .and_then(|val| val.as_i64())
                        .unwrap_or(1); //// default: fetch 1 hour ago
                    wrapup_sender.send(value).unwrap(); //// once we received the argument we'll send the value of this command to the downside of the channel to do its related task 
                    format!("9-5s are hard. WrapUps are easy. I’m on it!")
                },
                "expand" => {
                    let value = command
                        .data
                        .options
                        .get(0)
                        .and_then(|opt| opt.value.as_ref())
                        .and_then(|val| val.as_i64())
                        .unwrap_or(1); //// default: expand first bullet list
                    expand_sender.send(value).unwrap(); //// once we received the argument we'll send the value of this command to the downside of the channel to do its related task 
                    format!("Details make perfection, and perfection is not a detail")
                
                },
                "stats" => {
                    stats_sender.send(1).unwrap();
                    format!("Checking server status")
                },
                "help" => {
                    format!("**Examples**:\nGet a WrapUp for the past 2 hours : use `/wrapup 2`\nExpand on the 3rd bullet point from your WrapUp:  use `/expand 3`")
                } 
                _ => {
                    format!("Uknown command!")
                }
            };

            //// we first send the interaction response back to the slash command caller then 
            //// after that we'll do our computation once we get the interaction response
            //// message, the reason is the timeout of the interacton response is 3 seconds 
            //// and any computation higher than 3 seconds will send the `The application did not respond`
            //// error first then do the computations also we want to use the message id of the interaction
            //// response later to fetch all the messages before that, the solution to this is
            //// to use a oneshot channel in such a way that once the command argument received
            //// we'll send the value to downside of the channel to in order not to wait on the
            //// `response` of each tasks since each of them takes a long time to gets solved
            //// and the interaction response timeout is 3 seconds thus we can't wait a long time
            //// on their response to send the interaction response back to where it's called.
            let interaction_response = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::DeferredChannelMessageWithSource)
                        .interaction_response_data(|message| message.content("")) //// bot is thinking
                })
                .await;

            if let Err(why) = command
                    .edit_original_interaction_response(&ctx.http, |edit| edit.content(response_content.clone())) //// edit the thinking message with the command response
                    .await
                {
                    error!("error editing original interaction response since {:#?}", why);
                }

            match interaction_response{ //// matching on interaction response to do the computational tasks after sending the initial command response
                Ok(_) => {
                    //// sleep 1 seconds for the interaction response message to be created 
                    //// so its ID gets created inside the discrod db
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    //// once we received data from the downside of each channel
                    //// we'll do the heavy computational process
                    if let Ok(wrapup_value) = wrapup_receiver.try_recv(){
                        //// --------------------------------------------------------
                        //// -------------------- WRAPUP TASK -----------------------
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
                        let response = tasks::wrapup(&ctx, wrapup_value as u32, channel_id, init_cmd_time, interaction_response_message_id, user_id, guild_id).await;
                        info!("wrapup process response: {}", response);
                    }
                    
                    if let Ok(stats_flag) = stats_receiver.try_recv(){
                        if stats_flag == 1{
                            //// --------------------------------------------------------
                            //// -------------------- STATS TASK ------------------------
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
                            let response = tasks::stats(&ctx, channel_id, init_cmd_time, interaction_response_message_id).await;
                            info!("stats process response: {}", response);
                        }
                    }

                    if let Ok(exapnd_value) = expand_receiver.try_recv(){
                        //// --------------------------------------------------------
                        //// -------------------- EXAPND TASK -----------------------
                        //// -------------------------------------------------------- 
                        //// the following timestamp is approximate and may not exactly 
                        //// match the time when the command was executed.
                        let channel_id = command.channel_id;
                        let init_cmd_time = command.id.created_at(); //// id of the channel is a snowflake type that we can use it as the timestamp
                        let response = tasks::expand(&ctx, exapnd_value as u32, channel_id, init_cmd_time).await;
                        info!("expand process response: {}", response);
                    }
                },
                Err(why) => {
                    info!("can't respond to slash command {:?}", why);
                }
            }
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
            cmds::slash::wrapup_register(command)
        })
        .await;

        let _ = Command::create_global_application_command(&ctx.http, |command| {
            cmds::slash::expand_register(command)
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