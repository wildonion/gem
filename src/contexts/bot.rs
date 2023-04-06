



// --- bot link --- 
// https://discord.com/api/oauth2/authorize?client_id=1092048595605270589&permissions=274877974528&scope=bot




pub mod wwu_bot{

    use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt, CpuExt};
    use log::{info, error};
    use std::{sync::Arc, collections::HashSet};
    use once_cell::sync::Lazy;
    use futures::executor::block_on;
    use openai::{ //// openai crate is using the reqwest lib under the hood
        chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole}
    }; 
    use serenity::{async_trait, model::prelude::{MessageId, UserId}, framework::standard::{macros::help, HelpOptions, help_commands, CommandGroup}};
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
    
    
    pub static GPT: Lazy<Gpt> = Lazy::new(|| {
        block_on(Gpt::new())
    });



    //// ---------------------------------------------
    //// ----------- DISCORD BOT STRUCTURE -----------
    //// ---------------------------------------------
    // https://github.com/serenity-rs/serenity/blob/current/examples/

    #[group] //// grouping the following commands into the AskGPT group
    #[prefix = "gpt"]
    #[commands(news, expand, stats)]
    struct AskGPT; //// this can be accessible by GENERAL_GROUP inside the main.rs
    
    pub struct ShardManagerContainer;
    impl TypeMapKey for ShardManagerContainer {
        type Value = Arc<Mutex<ShardManager>>;
    }

    pub struct Handler; //// the discord bot commands and events over ws and webhook over http handler 

    //// following we're implementing the EventHandler trait
    //// for the Handler struct to handle all the bot events
    //// which will be fired or emitted through the discrod ws
    //// server thus in here we're subscribing to those events. 
    #[async_trait]
    impl EventHandler for Handler{

        async fn ready(&self, _: Context, ready: Ready){ //// handling ready events, once the bot shards gets ready 
            if let Some(shard) = ready.shard{ //// shard is an slice array of 2 elements, 8 bytes length each as the shard id
                info!("🔗 {} is connected on shard {}/{}", ready.user.name, shard[0], shard[1]);
            }
        }

        async fn message(&self, ctx: Context, msg: Message){ //// handling the message event
            
        }

        async fn resume(&self, _: Context, _: ResumedEvent){
            info!("▶ Resumed");
        }
    
    }


    //// ----------------------------------------------
    //// ---------------- BOT COMMANDS ----------------
    //// ----------------------------------------------

    #[command] //// news command
    async fn news(ctx: &Context, msg: &Message, _args: Args) -> CommandResult{
        

        //// ---------------------------
        //// setting up the GPT instance
        //// --------------------------- 
        let mut gpt = GPT.clone(); //// we're cloning static GPT instance to get the Gpt instance out of the Lazy structure 
        let mut response = "".to_string();
        let mut gpt_request_command = "".to_string();

        //// ---------------------------------
        //// parsing the bot command arguments 
        //// ---------------------------------
        let message_limit = _args.current().unwrap().parse::<u64>().unwrap_or(50); // → number of messages inside the channel for summerization
        let around_message_id = _args.current().unwrap().parse::<u64>().unwrap_or(0); // → the message id that we want to use it to do a summerization around of it (messages before and after that)
        
        //// ------------------------------------------------------
        //// fetching all channel messages based on above criterias
        //// ------------------------------------------------------ 
        let messages = msg.channel_id.messages(&ctx.http, |gm|{
            if around_message_id != 0{ //// fetching all the messages around the passed in message id (before and after that)
                gm
                    .around(around_message_id)
                    .limit(message_limit)
            } else{ //// fetching all the messages
                gm
                    .limit(message_limit)
            }
        }).await;

        //// -----------------------------------------------------------
        //// concatenating all the channel messages into a single string
        //// -----------------------------------------------------------
        let channel_messages = messages.unwrap_or_else(|_| vec![]);
        let messages = if channel_messages.len() > 1{
            channel_messages
                .into_iter()
                .map(|m|{
                    m.content
                })
                .collect::<Vec<String>>()
                .concat()
        } else{
            "".to_string()
        };

        //// ---------------------------------------------------------------
        //// feed the messages to the chat GPT to do a summerization process
        //// ---------------------------------------------------------------
        gpt_request_command = format!("can you summerize the content inside the bracket like news title as a numbered bullet list? [{}]", messages);
        let req_cmd = gpt_request_command.clone();
        response = gpt.feed(req_cmd.as_str()).await.current_response;
        info!("ChatGPT Response: {:?}", response);

        //// ----------------------------------------------
        //// sending the GPT response to the channel itself 
        //// ----------------------------------------------
        if let Err(why) = msg.channel_id.send_message(&ctx.http, |m|{
            m.embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                e.title("Here is the latest NEWS");
                e.description(response);
                return e;
            });
            m
        }).await{
            error!("can't send message embedding because {:#?}", why);
        }

        Ok(())

    }

    #[command] //// expand the summerization
    async fn expand(ctx: &Context, msg: &Message, _args: Args) -> CommandResult{
        
        //// ---------------------------
        //// setting up the GPT instance
        //// --------------------------- 
        let mut gpt = GPT.clone(); //// we're cloning static GPT instance to get the Gpt instance out of the Lazy structure 
        let mut response = "".to_string();
        let mut gpt_request_command = "".to_string();
        
        //// ---------------------------------
        //// parsing the bot command arguments 
        //// ---------------------------------
        let expand_which = _args.current().unwrap().parse::<u16>().unwrap_or(0); // → number of messages inside the channel for summerization
        
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
        gpt_request_command = format!("can you expand and explain more about the {} bullet list", ordinal);
        let req_cmd = gpt_request_command.clone();
        response = gpt.feed(req_cmd.as_str()).await.current_response;
        info!("ChatGPT Response: {:?}", response);

        //// ----------------------------------------------
        //// sending the GPT response to the channel itself 
        //// ----------------------------------------------
        let title = format!("Here is the expanded version of the {} bullet list", ordinal);
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

        let cpu_info_json = serde_json::to_string_pretty(&cpus).unwrap();

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
    pub struct Gpt<'c>{
        pub messages: Vec<ChatCompletionMessage>,
        pub last_content: &'c [u8], //// utf8 bytes is easier to handle tokenization process later
        pub current_response: String,
    }

    impl<'c> Gpt<'c>{
        
        pub async fn new() -> Gpt<'c>{
            //// this is not owned by the current function 
            //// so we can return it since:
            ////    - it's behind a pointer with a valid lifetime ('c)
            ////    - it's str and is inside either the stack or binary   
            let content = "Hello You're Amazing"; //// starting conversation to feed later tokens to the GPT model for prediction
            Self{
                messages: vec![
                    ChatCompletionMessage{
                        role: ChatCompletionMessageRole::System,
                        content: content.to_string(),
                        name: None,
                    }
                ],
                last_content: content.as_bytes(), //// since content is a string slice which is behind a pointer there is no need to clone it
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
        pub async fn feed(&mut self, content: &'c str) -> Gpt<'c>{
            
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
            messages.push(ChatCompletionMessage{
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
            Self{
                messages: self.messages.clone(),
                last_content: content.as_bytes(),
                current_response : self.current_response.clone(), //// cloning sicne rust doesn't allow to move the current_response into new scopes (where it has been called) since self is behind a pointer
            }
        }

    }    
    
}