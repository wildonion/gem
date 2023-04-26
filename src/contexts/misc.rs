



use crate::*;

pub mod cmds{

    use super::*;

    //// --------------------------------------
    //// ----------- SLASH COMMANDS -----------
    //// --------------------------------------
    pub mod slash{
        
        use super::*; //// loading all the crates that has loaded outside of this module
        
        //// command param will be over written later thus it must be defined mutable
        pub fn wrapup_register(command: &mut builder::CreateApplicationCommand) -> &mut builder::CreateApplicationCommand {
            command
                .name("wrapup")
                .description("conse wrap up summarizer")
                .create_option(|opt| {
                    opt
                        .name("hours")
                        .description("hours ago from 1 to 24")
                        .kind(CommandOptionType::Integer)
                        .min_int_value(1)
                        .max_int_value(24)
                        .required(true)
                })
        }

        //// command will be over written later thus it must be defined mutable
        pub fn expand_register(command: &mut builder::CreateApplicationCommand) -> &mut builder::CreateApplicationCommand {
            command
                .name("expand")
                .description("conse wrap up expand")
                .create_option(|opt| {
                    opt
                        .name("bullet")
                        .description("bullet list number for expantion")
                        .kind(CommandOptionType::Integer)
                        .min_int_value(1)
                        .max_int_value(1000)
                        .required(true)
                })
        }

        //// command will be over written later thus it must be defined mutable
        pub fn help_register(command: &mut builder::CreateApplicationCommand) -> &mut builder::CreateApplicationCommand {
            command
                .name("help")
                .description("conse wrap up help")

        }

        //// command will be over written later thus it must be defined mutable
        pub fn stats_register(command: &mut builder::CreateApplicationCommand) -> &mut builder::CreateApplicationCommand {
            command
                .name("stats")
                .description("conse server stats")

        }



    }


    #[hook]
    pub async fn delay_action(ctx: &Context, msg: &Message) {
        let _ = msg.react(ctx, '⏱').await;
    }
}



pub mod tasks{

    use super::*;


    /*  
         ------------------------------------------------
        |              SLASH COMMAND TASKS
        |------------------------------------------------
        | followings are related to slash commands' tasks
        |

    */



    pub async fn wrapup(ctx: &Context, hours_ago: u32, channel_id: ChannelId, init_cmd: Timestamp, command_message_id: u64) -> String{

        //// ---------------------------
        //// setting up the GPT instance
        //// ---------------------------
        //// data inside the bot client must be safe to 
        //// be shared between event and command handlers'
        //// threads thus they must be of type Arc<RwLock<TypeMapKey>>
        //// in which TypeMapKey is a trait that has implemented for 
        //// the underlying data which is of type Arc<Mutex<Data>>
        //// acquiring a write lock will block other event and 
        //// command handlers which don't allow them to use 
        //// the data until the lock is released.
        let mut data = ctx.data.write().await; //// writing safely to the GptBot instance also write lock returns a mutable reference to the underlying gpt::Gpt instance also data is of type Arc<RwLock<TypeMapKey>>
        let gpt_data = match data.get_mut::<misc::handlers::GptBot>(){ //// getting a mutable reference to the underlying data of the Arc<RwLock<TypeMapKey>> the GptBot structure
            Some(gpt) => gpt,
            None => {
                let resp = format!("ChatGPT is not online :(");
                if let Err(why) = channel_id.send_message(&ctx.http, |m|{
                    m.content("ChatGPT is not online :(")
                }).await{
                    error!("can't send message {:#?}", why);
                }
                return resp;
            },
        };
        
        let mut gpt_bot = gpt_data.lock().await;
        let mut response = "".to_string();
        let mut gpt_request_command = "".to_string();

        
        //// ----------------------------------------------------------------------------
        //// fetching all channel messages before the initialized /wrap command timestamp
        //// ----------------------------------------------------------------------------
        
        let command_time_offset = init_cmd.offset();
        let command_time_naive_local = init_cmd.naive_local(); //// initial command message datetime
        let date = command_time_naive_local.date();
        let time = command_time_naive_local.time();

        let start_fetching_year = date.year();
        let mut start_fetching_day = date.day();
        let start_fetching_month = date.month();
        let start_fetching_mins = time.minute();
        let start_fetching_secs = time.second();
        
        //// if the requested time was smaller than the passed 
        //// in hours ago means we must fetch all the 
        //// messages from a day ago at the calculated 
        //// correct time (see the time calculation logic).
        let ago = time.hour() as i32 - hours_ago as i32; 
        start_fetching_day = if ago < 0{ // a day ago 
            start_fetching_day = date.day() - 1;
            start_fetching_day as u32
        } else{
            start_fetching_day as u32 
        };

        //// ----------------------------------------------
        //// ----------- TIME CALCULATION LOGIC -----------
        //// ----------------------------------------------
        /*  
            -------------
            Logic Example
            -------------

            requested time hour : 10 in the morning
            hours ago           : 17
            10 < 17{
                start from hour = 10 + 24 - 17 = 34 - 17 = 17 or 5 in the evening
                start from day  = 10 - 17 = -7 
                -7 means that we've fallen into a day ago and must 
                fetch from a day ago started at 17 or 5 in the morning 
            }
            
            requested time hour : 10 in the morning
            hours ago           : 10
            10 == 10{
                start from = 10 - 10 = 00 or 12 late night
            }

            requested time hour : 10 in the morning
            hours ago           : 6
            10 > 6{
                start from = 10 - 6 = 4 in the morning
            }

        */
        //// if the requested time was greater than the 
        //// passed in hours ago time simply the start time
        //// will be the hours ago of the requested time.
        let start_fetching_hours = if time.hour() > hours_ago{
            time.hour() - hours_ago
        } 
        //// if the requested time was smaller than the 
        //// passed in hours ago time simply the start time
        //// will be the hours ago of the requested time + 24
        //// since the hours ago is greater than the requested time
        //// we have to add 24 hours to the requested time.
        else if time.hour() < hours_ago{
            (time.hour() + 24) - hours_ago
        } 
        //// if the requested time was equal to the 
        //// passed in hours ago time simply the start time
        //// will be the hours ago of the requested time 
        //// which will be 00 time or 12 late night.
        else{
            //// this can be 00
            time.hour() - hours_ago 
        };
        //// ----------------------------------------------
        //// ----------------------------------------------

        let d = chrono::NaiveDate::from_ymd_opt(start_fetching_year, start_fetching_month, start_fetching_day).unwrap();
        let t = chrono::NaiveTime::from_hms_opt(start_fetching_hours, start_fetching_mins, start_fetching_secs).unwrap();
        let start_fetching_from_timestamp = chrono::NaiveDateTime::new(d, t).timestamp() as u64;
        let start_fetching_from_string = chrono::NaiveDateTime::new(d, t).to_string();

        /*

            the snowflake ID is generated based on the timestamp, but it also includes 
            other information like worker ID, process ID, and an incrementing sequence number. 
            So, it's not possible to convert a timestamp directly to a snowflake ID without 
            knowing the other components. However, if you want to generate a Discord snowflake 
            ID where the timestamp part of the ID corresponds to your given timestamp, 
            you can follow the Discord snowflake format:

                42 bits for the timestamp (in milliseconds) - Discord's epoch is 1420070400000 (2015-01-01T00:00:00.000Z)
                5 bits for the worker ID
                5 bits for the process ID
                12 bits for the incrementing sequence number
        
            we can't create snowflake id directly from the message id since it depends 
            on the worker or thread id and the process id inside the server and we don't 
            know these to create the snowflake id thus the best way to fetch messages 
            after the passed in hours ago is to fetch all the messages before the 
            interaction response message timestamp and filter them to feed GPT those 
            messages that their timestamp is greater than the start_fetching_from_timestamp 
            
            let after_message_id = MessageId(start_fetching_from_timestamp);
        
        */

        let messages = channel_id    
            .messages(&ctx.http, |gm| {
                gm
                    //// we can convert the message id of the interaction response message to timestamp 
                    //// using https://snowsta.mp/?l=en-us&z=g&f=axciv6sznf-9zc for example for 1096776315631312936 
                    //// its timestamp would be 1681562250 or 2023-04-15T12:37:30.765Z
                    .before(command_message_id) //// fetch all messages before the initialized command timestamp
        }).await;

        //// -----------------------------------------------------------
        //// concatenating all the channel messages into a single string
        //// -----------------------------------------------------------
        let channel_messages = messages.unwrap_or_else(|_| vec![]);
        let messages = if channel_messages.len() > 1{
            let mut hours_ago_messages = vec![]; 
            let mut messages_iterator = channel_messages.into_iter();
            while let Some(m) = messages_iterator.next(){
                if (m.timestamp.timestamp() as u64) > start_fetching_from_timestamp{ //// only those messages that their timestamp is greater than the calculated starting timestamp are the ones that are n hours ago
                    let user_message = format!("@{}: {}", m.author.name, m.content);
                    hours_ago_messages.push(user_message);
                } else{
                    break;
                }
            }
            hours_ago_messages.concat()
        } else{
            "".to_string()
        };

        if messages.is_empty(){
            if let Err(why) = channel_id.send_message(&ctx.http, |m|{
                let response = format!("**Nothing to WrapUp in the past {} hours ago**", hours_ago);
                m.embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                    e.color(Colour::from_rgb(235, 204, 120));
                    e.description(response);
                    e.footer(|f|{ //// since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                        let content = format!("📨 WrapUp requested at: {} \n 🧩 WrappedUp from: {} \n 🕰️ timezone: {:#?}", command_time_naive_local.to_string(), start_fetching_from_string, command_time_offset);
                        f
                            .text(content.as_str())
                    });
                    return e;
                });
                m
            }).await{
                error!("can't send message {:#?}", why);
            }
            return "no messages in the past hours ago".to_string();
        }
        
        let typing = channel_id.start_typing(&ctx.http).unwrap();
        
        //// --------------------------------------------------------------------
        //// feed the messages to the chat GPT to do a long summarization process
        //// --------------------------------------------------------------------
        gpt_request_command = format!("Summarize each member's contribution to the discussion. Then put it in a numbered list so its easy to read. Also there is a user called JOE, do not add JOE's contributions to your summary.

        Here is how you should format your summaries: 
        
        1.  user1: summarize everything user 1 contributed to the discussion. 
        2. user2: summarize everything user 2 contributed to the discussion.\n
        
                                        
                                        {}", messages);
        let req_cmd = gpt_request_command.clone();
        response = gpt_bot.feed(req_cmd.as_str()).await.current_response;
        info!("ChatGPT Response: {:?}", response);

        typing.stop().unwrap(); //// stop typing after feeding GPT

        //// ----------------------------------------------
        //// sending the GPT response to the channel itself 
        //// ----------------------------------------------
        let title = format!("Here is your WrapUp from {} hour(s) ago", hours_ago);
        if let Err(why) = channel_id.send_message(&ctx.http, |m|{
            m.embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                e.color(Colour::from_rgb(235, 204, 120));
                e.title(title.as_str());
                e.description(response);
                e.footer(|f|{ //// since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                    let content = format!("📨 /wrapup requested at: {} \n 🧩 WrappedUp from: {} \n 🕰️ timezone: {:#?}", command_time_naive_local.to_string(), start_fetching_from_string, command_time_offset);
                    f
                        .text(content.as_str())
                });
                return e;
            });
            m
        }).await{
            error!("can't send message embedding because {:#?}", why);
            return format!("can't send message embedding because {:#?}", why);
        } else{
            return format!(""); //// embedding has sent
        }

        //// no need to update the ctx.data with the updated gpt_bot field 
        //// since we're already modifying it directly through the 
        //// write lock on the RwLock
        //// ...

    }



    pub async fn expand(ctx: &Context, expand_which: u32, channel_id: ChannelId, init_cmd: Timestamp) -> String{
        
        //// ---------------------------
        //// setting up the GPT instance
        //// ---------------------------
        //// data inside the bot client must be safe to 
        //// be shared between event and command handlers'
        //// threads thus they must be of type Arc<RwLock<TypeMapKey>>
        //// in which TypeMapKey is a trait that has implemented for 
        //// the underlying data which is of type Arc<Mutex<Data>>
        //// acquiring a write lock will block other event and 
        //// command handlers which don't allow them to use 
        //// the data until the lock is released.
        let mut data = ctx.data.write().await; //// write lock returns a mutable reference to the underlying Gpt instance also data is of type Arc<RwLock<TypeMapKey>>
        let gpt_data = match data.get_mut::<misc::handlers::GptBot>(){ //// getting a mutable reference to the underlying data of the Arc<RwLock<TypeMapKey>> which is GptBot
            Some(gpt) => gpt,
            None => {
                let resp = format!("ChatGPT is not online :(");
                if let Err(why) = channel_id.send_message(&ctx.http, |m|{
                    m.content("ChatGPT is not online :(")
                }).await{
                    error!("can't send message {:#?}", why);
                }
                return resp;
            },
        };

        let mut gpt_bot = gpt_data.lock().await; //// acquiring the mutex by locking on the gpt_data task which blocks the current thread
        let mut response = "".to_string();
        let mut gpt_request_command = "".to_string();


        //// ------------------------------------------------------------
        //// feed the messages to the chat GPT to do an expanding process
        //// ------------------------------------------------------------
        let ordinal = if expand_which == 1{
            "1st".to_string()
        } else if expand_which == 2{
            "2nd".to_string()
        } else if expand_which == 3{
            "3nd".to_string()
        } else{
            format!("{}th", expand_which)
        };

        let typing = channel_id.start_typing(&ctx.http).unwrap();

        gpt_request_command = format!("can you expand and explain more about the {} bullet list in the summarization discussion", ordinal);
        let req_cmd = gpt_request_command.clone();
        response = gpt_bot.feed(req_cmd.as_str()).await.current_response;
        info!("ChatGPT Response: {:?}", response);

        typing.stop().unwrap();

        //// ----------------------------------------------
        //// sending the GPT response to the channel itself 
        //// ----------------------------------------------
        let gpt_bot_messages = &gpt_bot.messages; //// since messages is a vector of String which doesn't implement the Copy trait we must borrow it in here 
        let messages_json_response = serde_json::to_string_pretty(&gpt_bot_messages).unwrap(); //// all the chat GPT messages  
        let title = format!("Here is the {} bullet list expanded from your WrapUp", ordinal);
        if let Err(why) = channel_id.send_message(&ctx.http, |m|{
            m.embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                e.color(Colour::from_rgb(235, 204, 120));
                e.title(title.as_str());
                e.description(response);
                e.footer(|f|{ //// since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                    let content = format!("📨 /expand requested at: {}", init_cmd.naive_local().to_string());
                    f
                        .text(content.as_str())
                });
                return e;
            });
            m
        }).await{
            error!("can't send message embedding because {:#?}", why);
            return format!("can't send message embedding because {:#?}", why);
        } else{
            return format!(""); //// embedding has sent
        }

        //// no need to update the ctx.data with the updated gpt_bot field 
        //// since we're already modifying it directly through the 
        //// write lock on the RwLock
        //// ...

    }


    pub async fn stats(ctx: &Context, channel_id: ChannelId, init_cmd: Timestamp, command_message_id: u64) -> String{


        // TODO - https://crates.io/crates/sysinfo

        let mut sys = System::new_all();
        sys.refresh_all();
        
        let memory = sys.available_memory();
        let mut cpus = vec![];
        let mut disks = vec![];
        
        for cpu in sys.cpus() {
            cpus.push(cpu.cpu_usage());
        }

        for disk in sys.disks() {
            disks.push(disk.total_space());
        }

        let json = serde_json::json!({
            "cpu_core_usage": cpus,
            "available_memory": memory,
            "disks_total_space": disks
        });
        let cpu_info_json = serde_json::to_string_pretty(&json).unwrap();

        let title = format!("Here is the resources info of the conse server");
        if let Err(why) = channel_id.send_message(&ctx.http, |m|{
            m.embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                e.color(Colour::from_rgb(235, 204, 120));
                e.title(title.as_str());
                e.description(cpu_info_json);
                e.footer(|f|{ //// since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                    let content = format!("📨 /stats requested at: {}", init_cmd.naive_local().to_string());
                    f
                        .text(content.as_str())
                });
                return e;
            });
            m
        }).await{
            error!("can't send message embedding because {:#?}", why);
            return format!("can't send message embedding because {:#?}", why);
        } else{
            return format!(""); //// embedding has sent
        }

    }
}



pub mod handlers{

    use super::*;

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
        type Value = Arc<Mutex<gpt::chat::Gpt>>;
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
                            
                            let response = misc::tasks::wrapup(&ctx, wrapup_value as u32, channel_id, init_cmd_time, interaction_response_message_id).await;
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
                                let response = misc::tasks::stats(&ctx, channel_id, init_cmd_time, interaction_response_message_id).await;
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
                            let response = misc::tasks::expand(&ctx, exapnd_value as u32, channel_id, init_cmd_time).await;
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
            let _ = Command::create_global_application_command(&ctx.http, |command| {
                misc::cmds::slash::wrapup_register(command)
            })
            .await;

            let _ = Command::create_global_application_command(&ctx.http, |command| {
                misc::cmds::slash::expand_register(command)
            })
            .await;

            let _ = Command::create_global_application_command(&ctx.http, |command| {
                misc::cmds::slash::help_register(command)
            })
            .await;

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
}