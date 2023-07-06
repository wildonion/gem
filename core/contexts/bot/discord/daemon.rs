



use crate::*;


pub async fn activate_discord_bot(
    discord_token: &str, 
    serenity_shards: u64, 
    gpt: gpt::chat::Gpt, 
    user_ratelimit: HashMap<u64, u64>,
    guild_ratelimit: HashMap<u64, u64>
    ){

    /* 
        each shard is a ws client to the discrod ws server also discord 
        requires that there be at least one shard for every 2500 guilds 
        (discrod servers) that a bot is on.
        
        data of each bot client must be safe to send between other shards' 
        threads means they must be Arc<Mutex<Data>> + Send + Sync + 'static
        or an RwLock type also each shard must be Arced and Mutexed to 
        be shareable between threads.
    */
    let http = http::Http::new(&discord_token);
    let (owners, _bot_id) = match http.get_current_application_info().await{ // fetching bot owner and id, application id is the id of the created http channel
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (Some(owners), Some(bot_id.id)),
                Err(why) => {
                    error!("😖 Could not access the bot id: {:?}", why);
                    (None, None)
                },
            }
        },
        Err(why) => {
            error!("😖 could not access discord bot application info: {:?}", why);
            (None, None)
        },
    };
    if owners.is_some(){

        let bot_buffer_size = env::var("BOT_BUFFER_SIZE").expect("⚠️ no bot buffer size key variable set").parse::<u32>().unwrap();
        let (tx, mut rx) = tokio::sync::mpsc::channel::<(Context, ApplicationCommandInteraction)>(bot_buffer_size as usize);
        let event_handler = handlers::Handler::new(tx.clone()).await;

        let framework = StandardFramework::new()
                                                .configure(|c| 
                                                    c
                                                        .on_mention(_bot_id)
                                                        .owners(owners.unwrap())
                                                );
        // gateway intents are predefined ws event gateways
        /* ---later on we must apply for each of intents--- */
        let intents = GatewayIntents::all(); // all the gateway intents must be on inside the https://discord.com/developers/applications/1092048595605270589/bot the privileged gateway intents section
        let mut bot_client = BotClient::builder(discord_token, intents)
                                                        .framework(framework)
                                                        .event_handler(event_handler)
                                                        .await
                                                        .expect("😖 in creating discord bot client");
        {   

            // building a shreable guild_rate_limit data structure to share 
            // it between shards' thread for handling user slash 
            // commands rate limit
            let guild_rate_limit = Arc::new(async_std::sync::Mutex::new(guild_ratelimit.clone()));
            // building a shreable user_ratelimit data structure to share 
            // it between shards' thread for handling user slash 
            // commands rate limit
            let user_rate_limit = Arc::new(async_std::sync::Mutex::new(user_ratelimit.clone()));
            // building a new chat GPT instance for our summerization process
            // it must be Send to be shared and Sync or safe to move it between 
            // shards' and command handlers' threads 
            let gpt_instance_cloned_mutexed = Arc::new(async_std::sync::Mutex::new(gpt.clone())); // Mutex is inside the tokio::sync
            // since we want to borrow the bot_client as immutable we must define 
            // a new scope to do this because if a mutable pointer exists 
            // an immutable one can't be there otherwise we get this Error:
            // cannot borrow `bot_client` as mutable because it is also borrowed as immutable
            let mut data = bot_client.data.write().await; // data of the bot client is of type RwLock which can be written safely in other threads
            data.insert::<handlers::GuildRateLimit>(guild_rate_limit.clone()); // writing the guild_rate_limit data structure into the data variable of the bot client to pass it between shards' threads
            data.insert::<handlers::RateLimit>(user_rate_limit.clone()); // writing the user_rate_limit data structure into the data variable of the bot client to pass it between shards' threads
            data.insert::<handlers::GptBot>(gpt_instance_cloned_mutexed.clone()); // writing the GPT bot instance into the data variable of the bot client to pass it between shards' threads 
            data.insert::<handlers::ShardManagerContainer>(bot_client.shard_manager.clone()); // writing a cloned shard manager inside the bot client data
        }
        // moving the shreable shard (Arc<Mutex<ShardManager>>) 
        // into tokio green threadpool to check all the shards status
        let shard_manager = bot_client.shard_manager.clone(); // each shard is an Arced Mutexed data that can be shared between other threads safely
        tokio::spawn(async move{
            tokio::signal::ctrl_c().await.expect("😖 install the plugin CTRL+C signal to the server");
            shard_manager.lock().await.shutdown_all().await; // once we received the ctrl + c we'll shutdown all shards or ws clients 
            // we'll print the current statuses of the shards to the 
            // terminal every 30 seconds. This includes the ID of the shard, 
            // the current connection stage, (e.g. "Connecting" or "Connected"), 
            // and the approximate WebSocket latency (time between when a heartbeat 
            // is sent to discord and when a heartbeat acknowledgement is received),
            // note that it may take a minute or more for a latency to be recorded or to
            // update, depending on how often Discord tells the client to send a heartbeat.
            loop{ // here we're logging the shard status every 30 seconds
                tokio::time::sleep(Duration::from_secs(30)).await; // wait for 30 seconds heartbeat of course it depends on the discord ws server of the heartbeat response
                let lock = shard_manager.lock().await;
                let shard_runners = lock.runners.lock().await;
                for (id, runner) in shard_runners.iter(){
                    info!(
                        "🧩 shard with ID {} is {} with a latency of {:?}",
                        id, runner.stage, runner.latency,
                    );
                }
            }
        });


        /*
            it's better no to put the db ops inside tokio::task::spawn(async move{}) or tokio::spawn(async move{}) since  
            async block may outlive the current function, but it might borrows a type which is owned by the current function
            and types which are owned by the function will be dropped from the ram once the function gets executed
            also async blocks are not executed immediately and must either take a reference or ownership of outside 
            variables they use since async blocks will be pinned to the ram to be solved later which must have kinda a static 
            lifetime, in our case in tokio::spawn(async move{}) the type will be moved between tokio green threads which force us 
            to move the type into the tokio::spawn(async move{}) using `move` keyword which takes the ownership of the type or
            take a reference to it like using Arc to share the data safely
        */
        // handling each interaction command as a separate task 
        // coming from the receiver inside the tokio green threadpool
        // in a concurrent manner
        tokio::spawn(async move{
            handlers::Handler::handle_interaction_command(rx).await;
        });

        // start the bot client with specified shards or ws clients for listening
        // to events, there is an ~5 second ratelimit period between when one shard 
        // can start after another, also Discord recommends one shard per 
        // 1,000 to 2,000 servers
        if let Err(why) = bot_client.start_shards(serenity_shards).await{
            error!("😖 discord bot client error: {:?}", why);
        }
    }

}