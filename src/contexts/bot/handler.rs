

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
    
    //// in Serenity, when handling an interaction_create event, 
    //// the Interaction object does not directly contain the 
    //// message instance. The reason is that slash commands 
    //// can be used without being tied to a specific message
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction.clone() {
            

            let (wrapup_sender, mut wrapup_receiver) = oneshot::channel::<i64>(); //// reading from the wrapup channel is a mutable process
            let (expand_sender, mut expand_receiver) = oneshot::channel::<i64>(); //// reading from the expand channel is a mutable process
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
            //// response later to fetch all the messages before that.
            let interaction_response = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(response_content)) //// the response to the intraction request for slash commands
                })
                .await;

            match interaction_response{ //// matching on interaction response to do the computational tasks
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
                        /////// here response takes a long time to gets solved
                        /////// and because of this halting issue the interaction 
                        /////// response will say The application did not respond
                        /////// since discrod timeout is 3 seconds to send the 
                        /////// response back to the user.
                        let response = ctx::bot::tasks::wrapup(&ctx, wrapup_value as u32, channel_id, init_cmd_time, interaction_response_message_id).await;
                        info!("wrapup process response: {}", response);
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