




/*


--- sources ---
https://blog.logrocket.com/building-rust-discord-bot-shuttle-serenity/
https://github.com/serenity-rs/serenity/tree/current/examples/
https://github.com/serenity-rs/serenity/blob/current/examples/e05_command_framework/src/main.rs

--- bot link example --- 
https://discord.com/api/oauth2/authorize?client_id=1092048595605270589&permissions=277025483776&scope=bot
https://discord.com/oauth2/authorize?client_id=1092048595605270589&scope=applications.commands

get token from : https://discord.com/developers/applications/1092048595605270589/bot


command examples:

    → show the help message
        !help conse

    → feed the chat GPT all the messages before the passed in hours ago (4 hours ago in this case) for summarization
        !conse wrapup 4
    
    → feed the chat GPT the selected bullet list to exapnd it
        !conse expand 2  


*/




pub mod wwu_bot{

    use chrono::{TimeZone, Timelike, Datelike, Utc}; //// this trait is rquired to be imported here to call the with_ymd_and_hms() method on a Utc object since every Utc object must be able to call the with_ymd_and_hms() method 
    use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt, CpuExt};
    use log::{info, error};
    use std::{sync::Arc, collections::HashSet};
    use openai::{ //// openai crate is using the reqwest lib under the hood
        chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole}
    };
    use serenity::{async_trait, model::prelude::{MessageId, UserId, ChannelId, 
                    interaction::application_command::{CommandDataOption, CommandDataOptionValue}}, 
                    framework::standard::{macros::{help, hook}, 
                    HelpOptions, help_commands, CommandGroup}
                };
    use serenity::builder;
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
    
    */



    //// ---------------------------------------------
    //// ----------- DISCORD BOT STRUCTURE -----------
    //// ---------------------------------------------
    // https://github.com/serenity-rs/serenity/blob/current/examples/

    #[group] //// grouping the following commands into the AskGPT group
    #[prefix = "conse"]
    #[commands(wrapup, expand, stats)]
    struct AskGPT; //// this can be accessible by GENERAL_GROUP inside the main.rs
    

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
        type Value = Arc<Mutex<Gpt>>;
    }

    pub struct Handler; //// the discord bot commands and events listener over ws and webhook over http 

    //// following we're implementing the EventHandler trait
    //// for the Handler struct to handle all the bot events
    //// which will be fired or emitted through the discrod ws
    //// server thus in here we're subscribing to those events. 
    #[async_trait]
    impl EventHandler for Handler{

        async fn ready(&self, ctx: Context, ready: Ready){ //// handling ready events, once the bot shards gets ready 
            if let Some(shard) = ready.shard{ //// shard is an slice array of 2 elements, 8 bytes length each as the shard id
                info!("🔗 {} bot is connected on shard id {}/{}", ready.user.name, shard[0], shard[1]);
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
        let mut data = ctx.data.write().await; //// write lock returns a mutable reference to the underlying Gpt instance also data is of type Arc<RwLock<TypeMapKey>>
        let gpt_data = match data.get_mut::<GptBot>(){ //// getting a mutable reference to the underlying data of the Arc<RwLock<TypeMapKey>> which is GptBot
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
        let gpt_data = match data.get_mut::<GptBot>(){ //// getting a mutable reference to the underlying data of the Arc<RwLock<TypeMapKey>> which is GptBot
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


    //// ----------------------------------------------
    //// -------------- GPT STRUCTURE -----------------
    //// ----------------------------------------------

    #[derive(Clone, Debug)]
    pub struct Gpt{
        pub messages: Vec<ChatCompletionMessage>,
        pub last_content: String, //// utf8 bytes is easier to handle tokenization process later
        pub current_response: String,
    }

    impl Gpt{
        
        pub async fn new() -> Gpt{
            let content = "Hello,"; //// starting conversation to feed later tokens to the GPT model for prediction
            Self{
                messages: vec![
                    ChatCompletionMessage{
                        role: ChatCompletionMessageRole::System,
                        content: content.to_string(),
                        name: None,
                    }
                ],
                last_content: content.to_string(),
                current_response: "".to_string()
            }
        }
        
        //→ if the content was String we couldn't return its &str since this is 
        //  owned by the function and its lifetime will be dropped once the function 
        //  gets executed thus we can't return a &str or a pointer to its utf8 bytes 
        //  because its pointer might be a dangling one in the caller space since 
        //  we don't have that String anymore inside the function! this is different
        //  about the &str in the first place cause we're cool with returning them
        //  since they are behind a pointer and kinda stack data types which by
        //  passing them to other scopes their lifetime won't be dropped since they
        //  will be copied bit by bit instead moving the entire underlying data.
        //→ also if the self wasn't behind a reference by calling the first method on 
        //  the Gpt instance the instance will be moved and we can't call other methods.
        //
        //// if we want to mutate the pointer it must be defined as mutable also its underlying data must be mutable 
        //// we borrow since copy trait doesn't implement for the type also we can clone it too to pass to other scopes 
        //// dereferencing and clone method will return the Self and can't deref if the underlying data doesn't implement Copy trait 
        pub async fn feed(&mut self, content: &str) -> Gpt{
            
            //→ based on borrowing and ownership rules in rust we can't move a type into new scope when there
            //  is a borrow or a pointer of that type exists, rust moves heap data types by default since it 
            //  has no gc rules means if the type doesn't implement Copy trait by moving it its lifetime will 
            //  be dropped from the memory and if the type is behind a pointer rust doesn't allow the type to 
            //  be moved, the reason is, by moving the type into new scopes its lifetime will be dropped 
            //  accordingly its pointer will be a dangling one in the past scope, to solve this we must either 
            //  pass its clone or its borrow to other scopes. in this case self is behind a mutable reference 
            //  thus by moving every field of self which doesn't implement Copy trait we'll lose the ownership 
            //  of that field and since it's behin a pointer rust won't let us do that in the first place which 
            //  forces us to pass either its borrow or clone to other scopes.   
            
            let mut messages = self.messages.clone(); //// clone messages vector since we can't move a type if it's behind a pointer 
            messages.push(ChatCompletionMessage{ //// pushing the current token to the vector so the GPT can be able to predict the next tokens based on the previous ones 
                role: ChatCompletionMessageRole::User,
                content: content.to_string(),
                name: None,
            });
            let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
                                                                        .create()
                                                                        .await
                                                                        .unwrap()
                                                                        .unwrap();
            let returned_message = chat_completion.choices.first().unwrap().message.clone();
            self.current_response = returned_message.content.to_string();
            messages.push(ChatCompletionMessage{ //// we must also push the response of the chat GPT to the messages in order he's able to predict the next tokens based on what he just saied :)  
                role: ChatCompletionMessageRole::Assistant,
                content: self.current_response.clone(),
                name: None
            });
            self.messages = messages.clone(); //// finally update the message field inside the Gpt instance 
            Self{
                messages,
                last_content: content.to_string(),
                current_response : self.current_response.clone(), //// cloning sicne rust doesn't allow to move the current_response into new scopes (where it has been called) since self is behind a pointer
            }
        }

    }    
    
}
