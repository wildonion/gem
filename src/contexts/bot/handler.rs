

use crate::*;





//// --------------------------------------------------------------------------------------
//// ---------------- Arc<Mutex<Data>> FOR SHARING BETWEEN SHARDS' THREADS ----------------
//// --------------------------------------------------------------------------------------
//// inside the Value type we'll use a Mutex to mutate 
//// the underlying data inside the Arc<RwLock<TypeKeyMap>> 
pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct GptBot;
impl TypeMapKey for GptBot{
    type Value = Arc<Mutex<ctx::gpt::chat::Gpt>>;
}



pub struct Handler; //// the discord bot commands and events listener/handler for emitted events and webhooks over ws and http 

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
    */
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction.clone() {
            
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
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(response_content)) //// the response to the intraction request for slash commands
                })
                .await;

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
                        
                        let response = ctx::bot::tasks::wrapup(&ctx, wrapup_value as u32, channel_id, init_cmd_time, interaction_response_message_id).await;
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
                            let response = ctx::bot::tasks::stats(&ctx, channel_id, init_cmd_time, interaction_response_message_id).await;
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
                        let response = ctx::bot::tasks::expand(&ctx, exapnd_value as u32, channel_id, init_cmd_time).await;
                        info!("expand process response: {}", response);
                    }
                },
                Err(why) => {
                    info!("can't respond to slash command {:?}", why);
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready){ //// handling ready events, once the bot shards gets ready 
        if let Some(shard) = ready.shard{ //// shard is an slice array of 2 elements, 8 bytes length each as the shard id
            info!("🔗 {} bot is connected on shard id {}/{}", ready.user.name, shard[0], shard[1]);
        }

        //// -------------------------------------------------
        //// --------- REGISTERING GLOBAL COMMANDS -----------
        //// -------------------------------------------------
        //// registering global commands for each 
        //// guild that this bot is added to
        let guilds = ready.guilds;
        for guild in guilds{
            let commands = GuildId::set_application_commands(&guild.id, &ctx.http, |commands| {
                commands
                    .create_application_command(|command| ctx::bot::cmds::slash::wrapup_register(command))
                    .create_application_command(|command| ctx::bot::cmds::slash::expand_register(command))
                    .create_application_command(|command| ctx::bot::cmds::slash::help_register(command))
                    .create_application_command(|command| ctx::bot::cmds::slash::stats_register(command))
            })
            .await;
        }

    }

    async fn message(&self, ctx: Context, msg: Message){ //// handling the message event
        //// ctx is the instance that contains 
        //// the methods and types of the whole
        //// setup bot. 
    }

    async fn resume(&self, _: Context, _: ResumedEvent){
        info!("▶ Resumed");
    }

}