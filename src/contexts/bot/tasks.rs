




/*
    
    ┏—————————————————————┓
        BOT SLASH TASKS
    ┗—————————————————————┛

*/


use crate::*;





/* —————————————————————
        WRAPUP TASK
————————————————————————*/

pub async fn wrapup(ctx: &Context, hours_ago: u32, channel_id: ChannelId, init_cmd: Timestamp, command_message_id: u64, user_id: u64, guild_id: u64) -> (String, String, String){
    
    //// ---------------------------
    //// setting up the GPT instance
    //// ---------------------------
    //// we should avoid any blocking operation inside the command in order not to get 
    //// the discord timeout response, thus we're creating the GPT instance per each 
    //// request for summarization process. since locking on the mutex is a 
    //// blocking process thus if we reaches the discord rate limit the locking process
    //// might gets halted inside the thread and other requests won't be able to use 
    //// the gpt_bot data since as long as the thread is locking the mutex other threads 
    //// can't mutate it thus the bot will be halted. 
    // let mut gpt_bot = gpt::chat::Gpt::new(None).await; //// passing none since we want to start a new catchup per each request

    //// data inside the bot client must be safe to be shared between event and command handlers'
    //// threads thus they must be of type Arc<RwLock<TypeMapKey>> in which TypeMapKey is a trait 
    //// that has implemented for the underlying data which is of type Arc<Mutex<Data>>
    //// acquiring a write lock will block other event and command handlers which don't allow 
    //// them to use the data until the lock is released.
    let mut data = ctx.data.write().await; //// write lock returns a mutable reference to the underlying Gpt instance also data is of type Arc<RwLock<TypeMapKey>>
    let gpt_data = match data.get_mut::<handlers::GptBot>(){ //// getting a mutable reference to the underlying data of the Arc<RwLock<TypeMapKey>> which is GptBot
        Some(gpt) => gpt,
        None => {
            let response = (format!("ChatGPT is not online :("), format!("📨 WrapUp requested at: {}", chrono::Local::now()), "".to_string());
            return response;
        },
    };
    
    let mut gpt_bot = gpt_data.lock().await;
    let mut gpt_response = "".to_string();
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
        let footer = format!("📨 WrapUp requested at: {} \n 🧩 WrappedUp from: {} \n 🕰️ timezone: {:#?}", command_time_naive_local.to_string(), start_fetching_from_string, command_time_offset);
        let title = "".to_string();
        let response = (format!("**Nothing to WrapUp in the past {} hours ago**", hours_ago), footer, title);
        return response;
    }
    
    //// --------------------------------------------------------------------
    //// feed the messages to the chat GPT to do a long summarization process
    //// --------------------------------------------------------------------
    gpt_request_command = format!("Summarize each member's contribution to the discussion. Then put it in a numbered list so its easy to read. Also there is a user called JOE, do not add JOE's contributions to your summary.

    Here is how you should format your summaries: 
    
    1.  user1: summarize everything user 1 contributed to the discussion. 
    2. user2: summarize everything user 2 contributed to the discussion.\n
    
                                    
                                    {}", messages);
    let req_cmd = gpt_request_command.clone();
    let feed_result = gpt_bot.feed(req_cmd.as_str()).await;
    if feed_result.is_rate_limit{
        let title = format!("");
        let description = "GPT rate limit".to_string().clone();
        let footer = format!("");
        let response = (description, footer, title);
        return response;
    }

    gpt_response = feed_result.current_response;
    let title = format!("Here is your WrapUp from {} hour(s) ago", hours_ago);
    let description = gpt_response.clone();
    let footer = format!("📨 /wrapup requested at: {} \n 🧩 WrappedUp from: {} \n 🕰️ timezone: {:#?}", command_time_naive_local.to_string(), start_fetching_from_string, command_time_offset);
    let response = (description, footer, title);
    

    /*
        ┏—————————————————————————————┓
           STORING CATCHUP DATA IN DB 
        ┗—————————————————————————————┛
        
        DON'T UNCOMMENT THE FOLLOWING TO MANY LOCKS INSIDE 
        THE COMMAND WILL FACE US DISCORD RATE LIMIT 
    
    */

    // let mut data = ctx.data.write().await; //// writing safely to the Storage instance also write lock returns a mutable reference to the underlying mongodb::Client instance also data is of type Arc<RwLock<TypeMapKey>>
    // let db_data = match data.get_mut::<handlers::Storage>(){ //// getting a mutable reference to the underlying data of the Arc<RwLock<TypeMapKey>> the Storage structure
    //     Some(db) => db,
    //     None => {
    //         let resp = format!("Storage is not accessible :(");
    //         if let Err(why) = channel_id.send_message(&ctx.http, |m|{
    //             m.content("Storage issue :(")
    //         }).await{
    //             error!("can't send message {:#?}", why);
    //         }
    //         return resp;
    //     },
    // };
    // let mut db = db_data.lock().await;

    //// we're building a new client everytime that a user request 
    //// a wrapup, we can't use shared data pattern to bring the db 
    //// in here since we must lock on it to get the underlying connection 
    //// which will face us discord ratelimit and timeout issue
    let db_host = env::var("DB_HOST").expect("⚠️ no db host variable set");
    let db_port = env::var("DB_PORT").expect("⚠️ no db port variable set");
    let db_username = env::var("DB_USERNAME").expect("⚠️ no db username variable set");
    let db_password = env::var("DB_PASSWORD").expect("⚠️ no db password variable set");
    let db_engine = env::var("DB_ENGINE").expect("⚠️ no db engine variable set");
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db_addr = format!("{}://{}:{}", db_engine, db_host, db_port);
    
    tokio::spawn(async move{
        let db = mongodb::Client::with_uri_str(db_addr.as_str()).await.unwrap();
        let catchup_data = db.database(&db_name).collection::<schemas::CatchUpDoc>("catchup_data");
        let catchup_document = schemas::CatchUpDoc{
            user_id,
            channel_id: channel_id.0,
            catchup_request_at: command_time_naive_local.to_string(),
            catchup_from: start_fetching_from_string.clone(),
            guild_id,
            gpt_response: gpt_response.clone()
        };
        match catchup_data.insert_one(catchup_document, None).await{ //// serializing the user doc which is of type RegisterRequest into the BSON to insert into the mongodb
            Ok(insert_result) => info!("inserted into the db with id {:#?}", insert_result.inserted_id.as_str()),
            Err(e) => error!("can't insert catchup data into db since {:#?}", e),
        };        
    });
    
    // -----------------------------------------------------------------------
    // -----------------------------------------------------------------------
    // -----------------------------------------------------------------------
    
    //// no need to update the ctx.data with the updated gpt_bot field 
    //// since we're already modifying it directly through the 
    //// write lock on the RwLock
    //// ...
    
    return response;


}


/* —————————————————————
        EXPAND TASK
————————————————————————*/

pub async fn expand(ctx: &Context, expand_which: u32, channel_id: ChannelId, init_cmd: Timestamp) -> (String, String, String){
    

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
    let gpt_data = match data.get_mut::<handlers::GptBot>(){ //// getting a mutable reference to the underlying data of the Arc<RwLock<TypeMapKey>> which is GptBot
        Some(gpt) => gpt,
        None => {
            let response = (format!("ChatGPT is not online :("), format!("📨 WrapUp requested at: {}", chrono::Local::now()), "".to_string());
            return response;
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

    gpt_request_command = format!("can you expand and explain more about the {} bullet list in the summarization discussion", ordinal);
    let req_cmd = gpt_request_command.clone();
    let feed_result = gpt_bot.feed(req_cmd.as_str()).await;
    if feed_result.is_rate_limit{
        let title = format!("");
        let description = "GPT rate limit".to_string().clone();
        let footer = format!("");
        let response = (description, footer, title);
        return response ;
    }

    response = feed_result.current_response;
    info!("ChatGPT Response: {:?}", response);
    let gpt_bot_messages = &gpt_bot.messages; //// since messages is a vector of String which doesn't implement the Copy trait we must borrow it in here 
    let messages_json_response = serde_json::to_string_pretty(&gpt_bot_messages).unwrap(); //// all the chat GPT messages  
    
    //// ----------------------------------------------
    //// sending the GPT response to the channel itself 
    //// ----------------------------------------------
    
    let title = format!("Here is the {} bullet list expanded from your WrapUp", ordinal);
    let description = response;
    let footer = format!("📨 /expand requested at: {}", init_cmd.naive_local().to_string());
    let response = (description, footer, title);
    return response;

    //// no need to update the ctx.data with the updated gpt_bot field 
    //// since we're already modifying it directly through the 
    //// write lock on the RwLock
    //// ...

}



/* —————————————————————
        STATS TASK
————————————————————————*/

pub async fn stats(ctx: &Context, channel_id: ChannelId, init_cmd: Timestamp, command_message_id: u64) -> (String, String, String){


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
    let description = cpu_info_json;
    let footer = format!("📨 /stats requested at: {}", init_cmd.naive_local().to_string());
    let response = (description, footer, title);
    return response;

}