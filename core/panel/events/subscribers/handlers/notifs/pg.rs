


use crate::*;

/* 
    event handler and listener for pg push notif system to notify app 
    about tables changes so we can always use the latest data updates
    in other parts of the app, the process took place by starting a 
    subscription loop or interval using while let Some()... syntax 
    in the background to notify the app once we received a new data 
    from the channel
*/
pub struct PgListener{}

impl PgListener{

    pub async fn subscribe(){
        
        // use redis pubsub pattern
        // use Postgres' NOTIFY/LISTEN to notify app on users table update
        // with worker patterns like mpsc, tokio::select, tokio::spawn(), 
        // to fetch latest data from db every 5 seconds with a global 
        // mutexed shared state data to gets mutated during the checking process
        // and accessible inside other actix routes threads

    }
}
