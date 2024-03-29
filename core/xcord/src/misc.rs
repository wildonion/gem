


use handlers::*;
use crate::*;


/* 
    making a thread safe shareable shard manager data structure 
    to share between serenity shards' threads safely
*/
pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<tokio::sync::RwLock<ShardManager>>;
}

/* 
    an static global mutex must be in RwLock in order to be mutable safely in threadpool 
    since static types can't be mutated since rust doesn't have gc and by mutating an static
    type we might have race conditions in other scopes.
*/
pub static USER_RATELIMIT: Lazy<HashMap<u64, u64>> = Lazy::new(||{
    HashMap::new()
});

pub const TASK_TOPIC_CHANNEL: &str = "XTASK";

/* 
    the new task structure that will be used to map the bytes 
    coming from the redispubsub into this
*/
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct NewTask{
    pub task_name: String,
    pub task_description: Option<String>,
    pub task_score: i32,
    pub task_priority: i32,
    pub hashtag: String,
    pub tweet_content: String,
    pub retweet_id: String,
    pub like_tweet_id: String,
    pub admin_id: i32
}

pub mod broadcast{

    use super::*;
    use serenity::{model::prelude::ChannelId, CacheAndHttp};


    pub async fn new_task(
        mut new_task_receiver: tokio::sync::mpsc::Receiver<NewTask>,
        ctx: std::sync::Arc<CacheAndHttp>){

        let channel_id = std::env::var("XCORD_CHANNEL_ID")
            .unwrap()
            .parse::<u64>()
            .unwrap();

        let target_channel_id = ChannelId(channel_id);
            
        /* ----------------------------------------------------------------------------------------- */
        /* ----------------------- receiving from redis pubsub mpsc receiver ----------------------- */
        /* ----------------------------------------------------------------------------------------- */
        /* 
            we must put the while let Some(..) = ..{} inside the 
            tokio::spawn(async move{}) to receive asyncly 
            to avoid blocking issues 
        */
        tokio::spawn(async move{
            
            while let Some(payload) = new_task_receiver.recv().await{

                /* Display trait is implemented for String types */
                target_channel_id.send_message(&ctx.http, |m|{
                    m
                        .allowed_mentions(|mentions| mentions.replied_user(true))
                        .embed(|e|{ // param type of embed() mehtod is FnOne closure : FnOnce(&mut CreateEmbed) -> &mut CreateEmbed
                            e.color(Colour::from_rgb(0, 152, 219));
                            e.description(payload.clone().task_description.unwrap());
                            e.title(payload.task_name);
                            e.footer(|f|{ // since method takes a param of type FnOnce closure which has a param instance of type CreateEmbedFooter struct
                                f
                                .text(&format!("taks score is: {}", payload.task_score))
                            });
                            return e;
                        });
                        m
                    }) // edit the thinking message with the command response
                    .await
                    .unwrap();
                

            }

        });
        
    }

}


pub mod daemon{

    use super::*;

    pub async fn activate_bot(
            discord_token: &str,
            serenity_shards: u64,
            new_task_receiver: tokio::sync::mpsc::Receiver<NewTask>
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
                        error!("😖 could not access the bot id: {:?}", why);
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

            let channel_id = env::var("TWISCORD_CHANNEL_ID").unwrap().parse::<u64>().unwrap();

            let event_handler = Handler{};

            let framework = StandardFramework::new()
                                                    .configure(|c| 
                                                        c
                                                            .on_mention(_bot_id)
                                                            .owners(owners.unwrap())
                                                    );
            // gateway intents are predefined ws event gateways
            let intents = GatewayIntents::all(); // all the gateway intents must be on inside the https://discord.com/developers/applications/1092048595605270589/bot the privileged gateway intents section
            let mut bot_client = BotClient::builder(discord_token, intents)
                                                            .framework(framework)
                                                            .event_handler(event_handler)
                                                            .await
                                                            .expect("😖 in creating discord bot client");
            {   

                /* 
                    since we want to borrow the bot_client as immutable we must define 
                    a new scope to do this because if a mutable pointer exists 
                    an immutable one can't be there otherwise we get this Error:
                    cannot borrow `bot_client` as mutable because it is also borrowed as immutable
                */
                let mut data = bot_client.data.write().await; // data of the bot client is of type RwLock which can be written safely in other threads
                data.insert::<handlers::ShardManagerContainer>(bot_client.shard_manager.clone()); // writing a cloned shard manager inside the bot client data
        
            }


            // moving the shreable shard (Arc<Mutex<ShardManager>>) 
            // into tokio green threadpool to check all the shards status
            let shard_manager = bot_client.shard_manager.clone(); // each shard is an Arced Mutexed data that can be shared between other threads safely
            let ctx = bot_client.cache_and_http.clone();
            


            broadcast::new_task(
                new_task_receiver, 
                ctx
            ).await;


            // start the bot client with specified shards or ws clients for listening
            // to events, there is an ~5 second ratelimit period between when one shard 
            // can start after another, also Discord recommends one shard per 
            // 1,000 to 2,000 servers
            if let Err(why) = bot_client.start_shards(serenity_shards).await{
                error!("😖 discord bot client error: {:?}", why);
            }
        }
            
    }

}