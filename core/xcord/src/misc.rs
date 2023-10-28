


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

    pub async fn new_task(){

    }

}


pub mod daemon{

    use super::*;

    pub async fn activate_bot(
            discord_token: &str,
            serenity_shards: u64,
            new_task_receiver: tokio::sync::mpsc::Receiver<NewTask>
        ){

    }

}