

/*  > -----------------------------------------------------------------------------------------------------
    | pg listener actor to subscribe to tables changes notifs and communicate with other parts of the app 
    | -----------------------------------------------------------------------------------------------------
    | contains: message structures and their handlers
    | 
    | remember to enable a trigger function to notify 
    | listeners of changes to table content 
    |
    |
*/

use crate::*;
use crate::constants::{WS_CLIENT_TIMEOUT, WS_SUBSCRIPTION_INTERVAL, STORAGE_IO_ERROR_CODE};
use crate::events::subscribers::handlers::actors::notif::system::NotifySystemActorWithRedisSubscription;
use crate::misc::*;
use crate::models::users::{User, UserData};
use redis_async::resp::FromResp;
use s3req::Storage;
use sqlx::postgres::PgListener;
use crate::*;
use actix::prelude::*;
use sqlx::Executor;
use super::system::{SystemActor, GetNewUser, GetSystemUsersMap, SystemUsers};





/* 
    pg notif actor is a ds that will start subscribing to postgres event in 
    its interval loop using while let Some()... syntax in the background and realtime
    once it gets started, to notify other parts about tables changes by sending 
    the received event from redis/pg through mpsc channels.
*/
#[derive(Clone)]
pub struct PgListenerActor{
    pub updated_users: HashMap<i32, UserData>,
    pub app_storage: Option<Arc<Storage>>,
    pub system_actor: Addr<SystemActor>,
}

impl Actor for PgListenerActor{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        info!("PgListenerActor -> started subscription interval");
        
        ctx.run_interval(WS_SUBSCRIPTION_INTERVAL, |actor, ctx|{
            
            let mut this = actor.clone();
            let app_storage = actor.app_storage.as_ref().unwrap();
            let get_sqlx_pg_listener = app_storage.get_sqlx_pg_listener_none_async().unwrap();
            let redis_async_pubsub = app_storage.get_async_redis_pubsub_conn_sync().unwrap();

            tokio::spawn(async move{
               
                // this.sqlx_subscribe(get_sqlx_pg_listener).await;
                this.redis_subscribe(redis_async_pubsub).await;
               
            });

        });
    }
}

impl PgListenerActor{

    pub fn new(app_storage: Option<Arc<Storage>>, system_actor: Addr<SystemActor>) -> Self{

        PgListenerActor{
            updated_users: HashMap::new(),
            app_storage,
            system_actor
        }
    }

    /* 
        pg streaming of events handler by subscribing to the related topic in an interval loop using 
        while let Some()... syntax and redis/pg, in order to get new changes by sending GetUpdatedRecord 
        message from different parts of the app to this actor to get the latest table update as a response 
        of this actor, this can be done by starting the actor in place where we're starting the server 
        then share the actor as a shared state data like Arc<Mutex< between actix routers threads so we 
        can extract it from the app_data in each api and send the GetUpdatedRecord message to fetch new 
        updated record of the passed in table name
    */
    pub async fn sqlx_subscribe(&mut self, sqlx_pg_listener: Arc<tokio::sync::Mutex<PgListener>>){

        /* start subscribing inside a separate threadpool */
        tokio::spawn(async move{
            loop{
    
                let mut sqlx_pg_listener = sqlx_pg_listener.lock().await;
                info!("inside the notif listener loop");
                
                /* start listening to all channels */
                while let Some(notification) = sqlx_pg_listener.try_recv().await.unwrap(){
                    let payload = notification.payload();
                    info!("got notification payload on {:?} channel : {:?}", payload, notification.channel());
                }
            }
        });

    }

    pub async fn redis_subscribe(&mut self, redis_async_pubsubconn: Arc<PubsubConnection>){

        let redis_async_pubsubconn = redis_async_pubsubconn.clone();
        let mut this = self.clone();
        let system_actor = self.system_actor.clone();

        /* 
            on user update subscription process is done using the redis async subscriber inside a tokio 
            threadpool which subscribes asyncly to the incoming future io object streams 
            from the passed in channel contains updated data over users tables
        */
        tokio::spawn(async move{

            /* 🚨 !!! 
                we must receive asyncly from the redis subscription streaming 
                channel otherwise actor gets halted in here since using sync 
                redis and actor redis cause the current thread gets halted
                because they'll receive in a blocking manner, thus we must 
                use tokio::spawn() to do so.

                subscribing to on_user_update pubsub channel asyncly and constantly using async redis
                by streaming over the incoming future tasks topics to decode the published topics
            
            !!! 🚨 */
            let get_stream_messages = redis_async_pubsubconn
                .subscribe("on_user_update")
                .await;
            
            let Ok(mut pubsubstreamer) = get_stream_messages else{

                use error::{ErrorKind, StorageError::RedisAsync, PanelError};
                let e = get_stream_messages.unwrap_err();
                let error_content = e.to_string().as_bytes().to_vec();  
                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(RedisAsync(e)), "PgListenerActor::redis_subscribe");
                let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                /*
                    since we should constantly keep subscribing to the event name 
                    thus there is no break in the loop and if there was an error 
                    in receiving from the pubsub streaming channel we must return;
                    from the method 
                */
                return (); 

            };
        
            /* 
                iterating through the pubsubstreamer to extract the Some part of each 
                message future object stream as they're coming to the stream channel,
            */
            while let Some(message) = pubsubstreamer.next().await{ 

                let resp_val = message.unwrap();
                let stringified_update_user = String::from_resp(resp_val).unwrap();
                
                info!("got user update notification payload on `on_user_update` channel at time {}", chrono::Local::now().timestamp());    
                system_actor
                    .send(NotifySystemActorWithRedisSubscription{
                        new_user: stringified_update_user
                    })
                    .await;

            }

        });

    }

}
