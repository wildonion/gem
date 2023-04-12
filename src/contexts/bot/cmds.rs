


use crate::*;



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



}



//// ------------------------------------------
//// ----------- FRAMEWORK COMMANDS -----------
//// ------------------------------------------
pub mod framework_command{

    use super::*; //// loading all the crates that has loaded outside of this module
    
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

    */



    //// ---------------------------------------------
    //// ----------- DISCORD BOT STRUCTURE -----------
    //// ---------------------------------------------
    // https://github.com/serenity-rs/serenity/blob/current/examples/

    #[group] //// grouping the following commands into the AskGPT group
    #[prefix = "conse"]
    #[commands(wrapup, expand, stats)]
    struct AskGPT; //// this can be accessible by GENERAL_GROUP inside the main.rs




    //// -----------------------------------------------------
    //// ---------------- BOT HOOKS AND HELPS ----------------
    //// -----------------------------------------------------
    #[help]
    async fn bot_help(
        context: &Context,
        msg: &Message,
        args: Args,
        help_options: &'static HelpOptions,
        groups: &[&'static CommandGroup],
        owners: HashSet<UserId>,
    ) -> CommandResult {
        let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
        Ok(())
    }

    #[hook]
    pub async fn delay_action(ctx: &Context, msg: &Message) {
        let _ = msg.react(ctx, '⏱').await;
    }


    //// ----------------------------------------------
    //// ---------------- BOT COMMANDS ----------------
    //// ----------------------------------------------
    //// in bot design there must be a ctx type that 
    //// can be passed to other handlers and used to 
    //// access whole methods and bot setup functions 
    //// like each ws shard inside each event handler 

    #[command] //// news command
    #[bucket="summerize"] //// required to define the bucket limitations on the news command event handler
    async fn wrapup(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult{

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
        let mut data = ctx.data.write().await; //// write lock returns a mutable reference to the underlying ctx::gpt::Gpt instance also data is of type Arc<RwLock<TypeMapKey>>
        let gpt_data = match data.get_mut::<ctx::bot::handler::GptBot>(){ //// getting a mutable reference to the underlying data of the Arc<RwLock<TypeMapKey>> which is ctx::bot::handler::GptBot
            Some(gpt) => gpt,
            None => {
                msg.reply(ctx, "ChatGPT is not online :(").await?;
                return Ok(());
            },
        };
        
        let mut gpt_bot = gpt_data.lock().await;
        let mut response = "".to_string();
        let mut gpt_request_command = "".to_string();

        //// ---------------------------------
        //// parsing the bot command arguments 
        //// ---------------------------------
        let mut args = _args.iter::<u32>();
        let hours_ago = args.next().unwrap_or(Ok(1)).unwrap(); //// default value will be set to 1 hour ago
        if hours_ago > 24{
            msg.reply(&ctx.http, "👎 Enter correct hour!").await.unwrap();
        }
        
        //// ------------------------------------------------------
        //// fetching all channel messages based on above criterias
        //// ------------------------------------------------------ 
        // example:
        // get 10 hours ago messages from the inital command
        // inital command is     : 2023-10-4 16:24:00
        // until 10 hours ago is : 2023-10-4 06:24:00
        // start fetching from   : 2023-10-4 06:24:00
        
        let command_time_offset = msg.timestamp.offset();
        let command_time_naive_local = msg.timestamp.naive_local(); //// initial command message datetime
        let date = command_time_naive_local.date();
        let time = command_time_naive_local.time();

        let start_fetching_year = date.year();
        let start_fetching_month = date.month();
        let start_fetching_day = date.day();

        let start_fetching_hours = time.hour() - hours_ago;
        let start_fetching_mins = time.minute();
        let start_fetching_secs = time.second();

        let d = chrono::NaiveDate::from_ymd_opt(start_fetching_year, start_fetching_month, start_fetching_day).unwrap();
        let t = chrono::NaiveTime::from_hms_opt(start_fetching_hours, start_fetching_mins, start_fetching_secs).unwrap();
        let start_fetching_from_timestamp = chrono::NaiveDateTime::new(d, t).timestamp() as u64;
        let start_fetching_from_string = chrono::NaiveDateTime::new(d, t).to_string();
        let after_message_id = MessageId(start_fetching_from_timestamp); //// creating the snowflake id from the timestamp (serde will do this)

        let messages = msg.channel_id    
            .messages(&ctx.http, |gm| {
                gm
                    .after(after_message_id) //// fetch messages after the passed snowflake id
        }).await;

        //// -----------------------------------------------------------
        //// concatenating all the channel messages into a single string
        //// -----------------------------------------------------------
        let channel_messages = messages.unwrap_or_else(|_| vec![]);
        let messages = if channel_messages.len() > 1{
            channel_messages
                .into_iter()
                .map(|m|{
                    let user_message = format!("@{}: {}", m.author.name, m.content);
                    user_message
                })
                .collect::<Vec<String>>()
                .concat()
        } else{
            "".to_string()
        };
        
        let _ = msg.react(ctx, '📰').await; //// send the reaction through the created ws shards won't be disconnected from the shard since it's a realtime communication
        
        let typing = msg.channel_id.start_typing(&ctx.http)?;
        
        //// ---------------------------------------------------------------
        //// feed the messages to the chat GPT to do a summarization process
        //// ---------------------------------------------------------------
        gpt_request_command = format!("can you summerize what users said inside the bracket as a numbered bullet list along with their username? [{}]", messages);
        let req_cmd = gpt_request_command.clone();
        response = gpt_bot.feed(req_cmd.as_str()).await.current_response;
        info!("ChatGPT Response: {:?}", response);

        typing.stop().unwrap(); //// stop typing after feeding GPT

        //// ----------------------------------------------
        //// sending the GPT response to the channel itself 
        //// ----------------------------------------------
        let title = format!("Here is all conse wrap ups for {} hour(s) ago", hours_ago);
        if let Err(why) = msg.channel_id.send_message(&ctx.http, |m|{
            m.embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                e.title(title.as_str());
                e.description(response);
                e.footer(|f|{ //// since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                    let content = format!("📨 wrap up requested at: {} \n 🧩 wrapped up from: {} \n 🕰️ timezone: {:#?}", command_time_naive_local.to_string(), start_fetching_from_string, command_time_offset);
                    f
                        .text(content.as_str())
                });
                return e;
            });
            m
        }).await{
            error!("can't send message embedding because {:#?}", why);
        }
        

        //// no need to update the ctx.data with the updated gpt_bot field 
        //// since we're already modifying it directly through the 
        //// write lock on the RwLock
        //// ...

        Ok(())

    }

    #[command] //// expand the summarization
    #[bucket="bullet"] //// required to define the bucket limitations on the expand command event handler
    async fn expand(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult{
        
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
        let gpt_data = match data.get_mut::<ctx::bot::handler::GptBot>(){ //// getting a mutable reference to the underlying data of the Arc<RwLock<TypeMapKey>> which is ctx::bot::handler::GptBot
            Some(gpt) => gpt,
            None => {
                msg.reply(ctx, "ChatGPT is not online :(").await?;
                return Ok(());
            },
        };

        let mut gpt_bot = gpt_data.lock().await;
        let mut response = "".to_string();
        let mut gpt_request_command = "".to_string();

        //// ---------------------------------
        //// parsing the bot command arguments 
        //// ---------------------------------
        let mut args = _args.iter::<u8>();
        let expand_which = args.next().unwrap().unwrap_or(1); 
        
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

        let _ = msg.react(ctx, '🔎').await; //// send the reaction through the created ws shards won't be disconnected from the shard since it's a realtime communication

        let typing = msg.channel_id.start_typing(&ctx.http)?;

        gpt_request_command = format!("can you expand and explain more about the {} bullet list in the summarization discussion", ordinal);
        let req_cmd = gpt_request_command.clone();
        response = gpt_bot.feed(req_cmd.as_str()).await.current_response;
        info!("ChatGPT Response: {:?}", response);

        typing.stop().unwrap();

        //// ----------------------------------------------
        //// sending the GPT response to the channel itself 
        //// ----------------------------------------------
        let gpt_bot_messages = &gpt_bot.messages; //// since messages is a vector of String which doesn't implement the Copy trait we must borrow it in here 
        let messages_json_response = serde_json::to_string_pretty(&gpt_bot_messages).unwrap();
        let title = format!("Here is the expanded version of the {} bullet list of the last warp up", ordinal);
        if let Err(why) = msg.channel_id.send_message(&ctx.http, |m|{
            m.embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                e.title(title.as_str());
                e.description(response);
                return e;
            });
            m
        }).await{
            error!("can't send message embedding because {:#?}", why);
        }

        //// no need to update the ctx.data with the updated gpt_bot field 
        //// since we're already modifying it directly through the 
        //// write lock on the RwLock
        //// ...
        
        Ok(())
        
    }

    #[command] //// conse server stats command
    async fn stats(ctx: &Context, msg: &Message, _args: Args) -> CommandResult{
        
        // TODO - https://crates.io/crates/sysinfo

        let mut sys = System::new_all();
        sys.refresh_all();
        let mut cpus = vec![];
        sys.refresh_cpu();
        for cpu in sys.cpus() {
            cpus.push(cpu.cpu_usage());
        }

        let json = serde_json::json!({
            "cpu_core_usage": cpus,
        });
        let cpu_info_json = serde_json::to_string_pretty(&json).unwrap();

        //// ------------------------------
        //// sending sysinfo to the channel
        //// ------------------------------
        if let Err(why) = msg.channel_id.send_message(&ctx.http, |m|{
            m.embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                e.title("server sysinfo stats");
                e.description(cpu_info_json.as_str());
                return e;
            });
            m
        }).await{
            error!("can't send message embedding because {:#?}", why);
        }

        Ok(())

    }
}


