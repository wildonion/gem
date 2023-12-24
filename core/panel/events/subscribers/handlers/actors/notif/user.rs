


/* 

    actor based pubsub workers in server/client (like tcp,tonic,http) for realtime streaming and monitoring like grafana
    - start actors globally in a place when the server gets built (static Lazy<Arc<Mutex<Actor>>> send sync 'static)
    - local pubsub pattern (using actix actor worker and the broker crate)
        publisher actor  -> publish/fire/emit/trigger event data using actix broker 
        subscriber actor -> subscribe to incoming data from publisher in the interval in tokio::spawn while let some and mpsc
    - redis pubsub pattern
        publisher actor  -> publish/fire/emit/trigger event data using redis actor in the interval then break once a subscriber receives it
        subscriber actor -> subscribe to incoming data from redis in the interval in tokio::spawn while let some and mpsc
    - http api must be triggered by frontend every 5 seconds in which we send message to subscriber actor worker to 
      get the notifications from redis and send it as the json response back to the caller

    step1
    publish user_actions notif into redis pubsub channel when a user
    send fan requests, likes, commnts, creates and updates nft and 
    collection or even unclaimed gifts then 
    we'll start this actor where the server is being started to subscribe 
    to the incoming notif from redis pubsub channel like pg.rs

    step2
    there must be an http api call to be called from the frontend in an interval 
    once it gets hooked in the server we'll send a message to this actor worker to 
    get user related actions and notifications so we can send them back as a response 
    to the caller and eventually frontend can show the related notifs, note that 
    the caller of the http api method must be either the action owner or a friend 
    of the user whose wants to see his actions 

*/

use crate::{*, constants::WS_SUBSCRIPTION_INTERVAL};
use actix::prelude::*;
use s3req::Storage;


#[derive(Clone)]
pub struct UserActionActor{
    pub app_storage: Option<Arc<Storage>>
}


impl Actor for UserActionActor{
    
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context){
        
        info!("UserActionActor -> started subscription interval");

        /* start subscription interval in tokio::spawn() using while let Some()... syntax */
        ctx.run_interval(WS_SUBSCRIPTION_INTERVAL, |actor, ctx|{

            let mut this = actor.clone();
            let app_storage = actor.app_storage.clone().unwrap();
            let redis_pubsub_async = app_storage.get_async_redis_pubsub_conn_sync().unwrap();

            // start subscribing to redis `user_actions` topic
            tokio::spawn(async move{

                this.redis_subscribe(app_storage, redis_pubsub_async).await;

            });

        });

    }
    
}

impl UserActionActor{

    pub async fn redis_subscribe(&mut self, app_storage: Arc<Storage>,
        redis_async_pubsubconn: Arc<PubsubConnection>){


        // start subscribing to the user_actions topic
        // inside tokio::spawn() using while let Some 
        // send the received data to mpsc channel to 
        // receive it outside of the tokio::spawn()
        // ...
        
    }

}