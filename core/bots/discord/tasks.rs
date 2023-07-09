




/*
    
    ┏—————————————————————┓
        BOT SLASH TASKS
    ┗—————————————————————┛


    📤  fix the rate limit issue of discord in /catchup command by queuing all the incoming interaction 
        request to the bot to handle them separately inside a threadpool (tokio definitely 🚀 ) 
    📤 fix the rate limit issue of chat GPT by handling each request to the openai server or the slash command as a 
        separate async task inside a threadpool, this task is basically the async version of the slash command requests 
        since every catchup slash commands is a separate api calling to the openai server 
    📤 remove /expand command to avoid dead lock and blocking situation since expanding a bullet 
        point needs the updated instance of the GPT structure in which it can only be acquired through 
        the mutex locking which leads us to block the current thread thus the discord rate limit issue.
    📤 remove the whole db setup from the code since its IO load was too heavy which causes 
        the bot got stuck in the halting mode.   



*/


use std::io::SeekFrom;
use chrono::NaiveDate;
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncWriteExt, AsyncSeekExt};
use crate::*;





/* —————————————————————
       CATCHUP TASK
————————————————————————*/

pub async fn catchup(ctx: &Context, hours_ago: u32, channel_id: ChannelId, init_cmd: Timestamp, command_message_id: u64, user_id: u64, guild_id: u64) -> (String, String, String){
    
    // -------------------------------------------------------------------------------
    // fetching all channel messages before the initialized /catchup command timestamp
    // -------------------------------------------------------------------------------
    
    let command_time_offset = init_cmd.offset();
    let command_time_naive_local = init_cmd.naive_local(); // initial command message datetime
    let date = command_time_naive_local.date();
    let time = command_time_naive_local.time();

    let mut start_fetching_year = date.year();
    let mut start_fetching_day = date.day();
    let mut start_fetching_month = date.month();
    let start_fetching_mins = time.minute();
    let start_fetching_secs = time.second();

    // ----------------------------------------------
    // ----------- TIME CALCULATION LOGIC -----------
    // ----------------------------------------------
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
    
    fn get_days_from_month(year: i32, month: u32) -> u32 {
        NaiveDate::from_ymd_opt(
            match month {
                12 => year + 1,
                _ => year,
            },
            match month {
                12 => 1,
                _ => month + 1,
            },
            1,
        )
        .unwrap()
        .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .num_days() as u32
    }
    
    // if the requested time was smaller than the passed 
    // in hours ago means we must fetch all the 
    // messages from a day ago at the calculated 
    // correct time (see the time calculation logic).
    let ago = time.hour() as i32 - hours_ago as i32; 
    start_fetching_day = if ago < 0{ // a day ago 
        start_fetching_day = if date.day() - 1 == 0{
            let prev_month_num = start_fetching_month - 1;
            let current_day;
            if prev_month_num == 0{ // means it's first month of the year and we have to fetch the last year 
                start_fetching_year -= 1;
                start_fetching_month = 12; // last month is december 
                current_day = 31; // december has 31 days
            } else{
                start_fetching_month = prev_month_num;
                let day = get_days_from_month(start_fetching_year, prev_month_num);
                current_day = day;
            }
            current_day
        } else{
            date.day() - 1
        };
        start_fetching_day as u32
    } else{
        start_fetching_day as u32 
    };

    // if the requested time was greater than the 
    // passed in hours ago time simply the start time
    // will be the hours ago of the requested time.
    let start_fetching_hours = if time.hour() > hours_ago{
        time.hour() - hours_ago
    } 
    // if the requested time was smaller than the 
    // passed in hours ago time simply the start time
    // will be the hours ago of the requested time + 24
    // since the hours ago is greater than the requested time
    // we have to add 24 hours to the requested time.
    else if time.hour() < hours_ago{
        (time.hour() + 24) - hours_ago
    } 
    // if the requested time was equal to the 
    // passed in hours ago time simply the start time
    // will be the hours ago of the requested time 
    // which will be 00 time or 12 late night.
    else{
        // this can be 00
        time.hour() - hours_ago 
    };
    // ----------------------------------------------
    // ----------------------------------------------
    // ----------------------------------------------

    let d = match chrono::NaiveDate::from_ymd_opt(start_fetching_year, start_fetching_month, start_fetching_day){
        Some(d) => {
            d
        },
        None => {
            let log_name = format!("[{}]", chrono::Local::now());
            let filepath = format!("logs/error-kind/{}-inappropriate-date.log", log_name);
            let log_content = format!("year:{}|month:{}|day:{}", start_fetching_year, start_fetching_month, start_fetching_day);
            let mut error_kind_log = tokio::fs::File::create(filepath.as_str()).await.unwrap();
            error_kind_log.write_all(log_content.as_bytes()).await.unwrap();
            let footer = format!("");
            let title = "".to_string();
            let response = (format!("**I lost dates**"), footer, title);
            return response;
        }
    };
    
    let t = match chrono::NaiveTime::from_hms_opt(start_fetching_hours, start_fetching_mins, start_fetching_secs){
        Some(t) => {
            t
        },
        None => {        
            let log_name = format!("[{}]", chrono::Local::now());
            let filepath = format!("logs/error-kind/{}-inappropriate-time.log", log_name);
            let log_content = format!("hours:{}|mins:{}|secs:{}", start_fetching_hours, start_fetching_mins, start_fetching_secs);
            let mut error_kind_log = tokio::fs::File::create(filepath.as_str()).await.unwrap();
            error_kind_log.write_all(log_content.as_bytes()).await.unwrap();
            let footer = format!("");
            let title = "".to_string();
            let response = (format!("**I lost times**"), footer, title);
            return response;
        }
    };

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
                // we can convert the message id of the interaction response message to timestamp 
                // using https://snowsta.mp/?l=en-us&z=g&f=axciv6sznf-9zc for example for 1096776315631312936 
                // its timestamp would be 1681562250 or 2023-04-15T12:37:30.765Z
                .before(command_message_id) // fetch all messages before the initialized command timestamp
    }).await;

    // -----------------------------------------------------------
    // concatenating all the channel messages into a single string
    // -----------------------------------------------------------
    let bot_username = env::var("BOT_USERNAME").expect("⚠️ no bot username variable set");
    let channel_messages = messages.unwrap_or_else(|_| vec![]);
    let messages = if channel_messages.len() > 1{
        let mut hours_ago_messages = vec![]; 
        let mut messages_iterator = channel_messages.into_iter();
        while let Some(m) = messages_iterator.next(){
            if (m.timestamp.timestamp() as u64) > start_fetching_from_timestamp{ // only those messages that their timestamp is greater than the calculated starting timestamp are the ones that are n hours ago
                /*
                    in fetching messages from the discord server all type of messages will be fetched including 
                    the embeddings and since bot is sending response back in an embedding format they have no 
                    content inside of themselves they are just embeddings can't get their content simply
                    and because those messages that fetched contains the bot user, their content are empty 
                    thus GPT in some how attached the actual messages by other user which contains the content 
                    to the bot empty embedding and thus we'll just ignore what the bot said in total and go to other massage  
                */
                if m.author.name == bot_username{
                    continue;
                }
                let user_message = format!("{}:{}, ", m.author.name, m.content);
                hours_ago_messages.push(user_message);
            } else{
                continue;
            }
        }
        hours_ago_messages.reverse(); // reverse the fetched messages so GPT can summarize from the recent one to the latest one
        hours_ago_messages.concat()
    } else{
        "".to_string()
    };

    if messages.is_empty(){
        let footer = format!("📨 CatchUp requested at: {} \n 🧩 CaughtUp from: {} \n 🕰️ timezone: {:#?}", command_time_naive_local.to_string(), start_fetching_from_string, command_time_offset);
        let title = "".to_string();
        let response = (format!("**Nothing to CatchUp in the past {} hours**", hours_ago), footer, title);
        return response;
    }

    // --------------------------------------------------------------------
    // feed the messages to the chat GPT to do a long summarization process
    // --------------------------------------------------------------------
    let mut gpt_request_command = "".to_string();
    gpt_request_command = format!("{}:{}", gpt::chat::GPT_PROMPT, messages);

    /*
        ┏—————————————————————————————————————————————————┓
            LOCKING ON GPT INSTANCE TO ACQUIRE THE MUTEX  
        ┗—————————————————————————————————————————————————┛
        
        DON'T UNCOMMENT THE FOLLOWING, TOO MANY LOCKS INSIDE 
        THE COMMAND WILL FACE US DISCORD RATE LIMIT SINCE 
        IT'LL HALT IN THE LOCKING PROCESS BECAUSE OTHER THREADS
        WANT TO MUTATE THE DATA BUT THE FIRST THREAD IS NOT DONE
        WITH THE ACQUIRED DATA.  
    
        we should avoid any blocking operation inside the command in order not to get 
        the discord timeout response, thus we're creating the GPT instance per each 
        request for summarization process. since locking on the mutex is a blocking 
        process thus if we reaches the discord rate limit the locking process might 
        gets halted inside the thread and other requests won't be able to use the gpt_bot 
        data since as long as the thread is locking the mutex other threads can't mutate 
        it thus the bot will be halted. 
        data inside the bot client must be safe to be shared between event and command handlers'
        threads thus they must be of type Arc<RwLock<TypeMapKey>> in which TypeMapKey is a trait 
        that has implemented for the underlying data which is of type Arc<Mutex<Data>>
        acquiring a write lock will block other event and command handlers which don't allow 
        them to use the data until the lock is released.
    */
    // let mut data = ctx.data.write().await; // write lock returns a mutable reference to the underlying Gpt instance also data is of type Arc<RwLock<TypeMapKey>>
    // let gpt_data = match data.get_mut::<handlers::GptBot>(){ // getting a mutable reference to the underlying data of the Arc<RwLock<TypeMapKey>> which is GptBot
    //     Some(gpt) => gpt,
    //     None => {
    //         let response = (format!("ChatGPT is not online"), format!("📨 CatchUp requested at: {}", chrono::Local::now()), "".to_string());
    //         return response;
    //     },
    // };
    // let mut gpt_bot = gpt_data.lock().await;
    
    let mut gpt_bot = gpt::chat::Gpt::new(None).await; // passing none since we want to start a new catchup chat history per each request
    let mut gpt_response = "".to_string();
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
    let title = format!("Here is your CatchUp from {} hour(s) ago", hours_ago);
    let description = gpt_response.clone();
    let footer = format!("📨 /catchup requested at: {} \n 🧩 CaughtUp from: {} \n 🕰️ timezone: {:#?}", command_time_naive_local.to_string(), start_fetching_from_string, command_time_offset);
    let response = (description, footer, title);
    

    /*
        ┏—————————————————————————————————┓
           STORING CATCHUP DATA IN A FILE 
        ┗—————————————————————————————————┛

         when a new thread is spawned, the provided closure can 
         only borrow items with a static lifetime. In other words, 
         the borrowed values must be alive for the full program lifetime.
          
    */

    // since we're doing IO we must put the task inside the
    // tokio green threadpool to avoid rate limit and halting 
    // issues since tokio::spawn(async move{}) handle async 
    // tasks concurrently
    tokio::spawn(async move{
        
        let mut gpt_ok = false;
        if !gpt_response.is_empty(){
            gpt_ok = true;
        }

        let log_content = format!("[{}] - userId:{}|channelId:{}|catchupRequestedAt:{}|catchupFrom:{}|guildId:{}|gptResponseOk:{}\n", chrono::Local::now(), user_id, channel_id.0, command_time_naive_local.to_string(), start_fetching_from_string.clone(), guild_id, gpt_ok);
        let filepath = format!("logs/gpt-logs/requests.log");
        let mut gpt_log;

        match fs::metadata("logs/gpt-logs/requests.log").await {
            Ok(_) => {
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(filepath.as_str())
                    .await.unwrap();
                file.write_all(log_content.as_bytes()).await.unwrap(); // Write the data to the file
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                gpt_log = tokio::fs::File::create(filepath.as_str()).await.unwrap();
                gpt_log.write_all(log_content.as_bytes()).await.unwrap();
            },
            Err(e) => {
                let log_name = format!("[{}]", chrono::Local::now());
                let filepath = format!("logs/error-kind/{}-gpt-reading-log-file.log", log_name);
                let mut error_kind_log = tokio::fs::File::create(filepath.as_str()).await.unwrap();
                error_kind_log.write_all(e.to_string().as_bytes()).await.unwrap();
            }
        }

    });
    
    // no need to update the ctx.data with the updated gpt_bot field 
    // since we're already modifying it directly through the 
    // write lock on the RwLock
    // ...
    
    return response;


}


/* —————————————————————
        STATS TASK
————————————————————————*/

pub async fn stats(ctx: &Context, channel_id: ChannelId, init_cmd: Timestamp, command_message_id: u64) -> (String, String, String){

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