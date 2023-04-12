





use crate::*;



pub async fn wrapup(ctx: &Context, hours_ago: u32, channel_id: ChannelId, init_cmd: Timestamp) -> String{

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
    let gpt_data = match data.get_mut::<ctx::bot::handler::GptBot>(){ //// getting a mutable reference to the underlying data of the Arc<RwLock<TypeMapKey>> which is GptBot
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

    
    //// ------------------------------------------------------
    //// fetching all channel messages based on above criterias
    //// ------------------------------------------------------ 
    // example:
    // get 10 hours ago messages from the inital command
    // inital command is     : 2023-10-4 16:24:00
    // until 10 hours ago is : 2023-10-4 06:24:00
    // start fetching from   : 2023-10-4 06:24:00
    
    let command_time_offset = init_cmd.offset();
    let command_time_naive_local = init_cmd.naive_local(); //// initial command message datetime
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

    let messages = channel_id    
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
    
    let typing = channel_id.start_typing(&ctx.http).unwrap();
    
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
    if let Err(why) = channel_id.send_message(&ctx.http, |m|{
        m.embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
            e.color(Colour::from_rgb(235, 204, 120));
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
        return format!("can't send message embedding because {:#?}", why);
    } else{
        return format!(""); //// embedding has sent
    }

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
    let gpt_data = match data.get_mut::<ctx::bot::handler::GptBot>(){ //// getting a mutable reference to the underlying data of the Arc<RwLock<TypeMapKey>> which is GptBot
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
    let messages_json_response = serde_json::to_string_pretty(&gpt_bot_messages).unwrap();
    let title = format!("Here is the expanded version of the {} bullet list of the last warp up", ordinal);
    if let Err(why) = channel_id.send_message(&ctx.http, |m|{
        m.embed(|e|{ //// param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
            e.color(Colour::from_rgb(235, 204, 120));
            e.title(title.as_str());
            e.description(response);
            e.footer(|f|{ //// since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                let content = format!("🪶 requested at: {}", init_cmd.naive_local().to_string());
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