

/*  > -----------------------------------------------------------------------------------------------------
    | pg listener actor to subscribe to tables changes notifs and communicate with other parts of the app 
    | -----------------------------------------------------------------------------------------------------
    | contains: message structures and their handlers
    | 
    |
*/

use crate::*;
use crate::constants::{WS_CLIENT_TIMEOUT, WS_SUBSCRIPTION_INTERVAL, STORAGE_IO_ERROR_CODE};
use crate::events::subscribers::handlers::actors::notif::system::NotifySystemActorWithRedisSubscription;
use crate::misc::*;
use crate::models::users::{User, UserData};
use crate::models::users_fans::UserFan;
use crate::models::users_nfts::UserNft;
use redis_async::resp::FromResp;
use s3req::Storage;
use sqlx::postgres::PgListener;
use crate::*;
use actix::prelude::*;
use sqlx::Executor;
use super::system::{SystemActor, GetNewUser, GetSystemUsersMap, SystemUsers};





/* 
    user notif actor is a ds that will start subscribing to postgres event in 
    its interval loop using while let Some()... syntax in the background and realtime
    once it gets started, to notify other parts about tables changes by sending 
    the received event from redis/pg through mpsc channels.
*/
#[derive(Clone)]
pub struct UserListenerActor{
    /* 
        we're using an in memory map based db to store updated user in runtime and realtime
        hence it's fast enough to do read and write operations
    */
    pub updated_users: HashMap<i32, UserData>,
    pub app_storage: Option<Arc<Storage>>,
    pub system_actor: Addr<SystemActor>,
}

impl Actor for UserListenerActor{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        info!("UserListenerActor -> started subscription interval");

        // no need to run the subscription process in actor interval
        // cause we're subscribing in the loop using while let Some 
        // syntax with redis async
        // ctx.run_interval(WS_SUBSCRIPTION_INTERVAL, |actor, ctx|{
            
            let mut this = self.clone();
            let app_storage = self.app_storage.clone().unwrap(); // don't borrow it by using as_ref() cause app_storage is going to be moved out of closure and we can't move if it gets borrowed using as_ref()
            let get_sqlx_pg_listener = app_storage.get_sqlx_pg_listener_none_async().unwrap();
            let redis_async_pubsub = app_storage.get_async_redis_pubsub_conn_sync().unwrap();

            tokio::spawn(async move{
               
                // this.sqlx_subscribe(get_sqlx_pg_listener).await;
                this.redis_subscribe(app_storage, redis_async_pubsub).await;
               
            });

        // });
    }
}

impl UserListenerActor{

    pub fn new(app_storage: Option<Arc<Storage>>, system_actor: Addr<SystemActor>) -> Self{

        UserListenerActor{
            updated_users: HashMap::new(),
            app_storage,
            system_actor
        }
    }

    /* >_ ------------------ redis and sqlx subscription process ------------------
        pg streaming of events handler by subscribing to the related topic in an interval loop using 
        while let Some()... syntax and redis/pg, in order to get new changes by sending GetUpdatedRecord 
        message from different parts of the app to this actor to get the latest table update as a response 
        of this actor, this can be done by starting the actor in place where we're starting the server 
        then share the actor as a shared state data like Arc<Mutex< between actix routers threads so we 
        can extract it from the app_data in each api and send the GetUpdatedRecord message to fetch new 
        updated record of the passed in table name

        remember to enable a trigger function to notify 
        listeners of changes to table content 
        https://hackmd.io/@wtflink515/BksvOw2Hw

        create EXTENSION tcn;
        create trigger users_tcn_trigger
        after insert or update or delete on users
        for each row execute function triggered_change_notification();

    */
    pub async fn sqlx_subscribe(&mut self, sqlx_pg_listener: Arc<tokio::sync::Mutex<PgListener>>){

        /* start subscribing inside a separate threadpool */
        tokio::spawn(async move{

            let mut sqlx_pg_listener = sqlx_pg_listener.lock().await;
            info!("inside the notif listener loop");
            
            /* start listening to all channels */
            while let Some(notification) = sqlx_pg_listener.try_recv().await.unwrap(){
                let payload = notification.payload();
                info!("got notification payload on {:?} channel : {:?}", payload, notification.channel());
            }
        });

    }

    pub async fn redis_subscribe(&mut self, app_storage: Arc<Storage>,
        redis_async_pubsubconn: Arc<PubsubConnection>){

        let cloned_app_storage = app_storage.clone(); // we're gonna move this into the tokio::spawn
        let redis_async_pubsubconn = redis_async_pubsubconn.clone();
        let mut this = self.clone();
        let system_actor = self.system_actor.clone();

        /* 
            on user update subscription process is done using the redis async subscriber inside a tokio 
            threadpool which subscribes asyncly to the incoming future io object streams 
            from the passed in channel that contains updated data over users tables
        */
        tokio::spawn(async move{

            /* 
                we should get and extract the pg pool in tokio spawn not outside of it 
                cause pg pool doesn't implement the Clone trait
            */
            let pool_conn = cloned_app_storage.get_pgdb().await;
            let connection = &mut pool_conn.unwrap().get().unwrap();

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
                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(RedisAsync(e)), "UserListenerActor::redis_subscribe");
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
                let decoded_user = serde_json::from_str::<UserData>(&stringified_update_user).unwrap();

                info!("got user update notification payload on `on_user_update` channel at time {}", chrono::Local::now().timestamp());    
                system_actor
                    .send(NotifySystemActorWithRedisSubscription{
                        new_user: stringified_update_user
                    })
                    .await;

                //----------------------------------------------
                //------------- users_nfts and users_fans hooks
                //----------------------------------------------
                // we'll update following tables once we received 
                // new update of a user from redis pubsub channel

                info!("updateing `users_nfts` and `users_fans` tables with new user data");
                
                // trigger the update process of user nfts likes and comments with this user
                if let Err(why) = UserNft::update_nft_reactions_with_this_user(decoded_user.clone(), connection).await{
                    error!("can't update `users_nfts` table due to: {}", why);
                };

                // trigger the update process of user fans friends and invitation requests with this user
                if let Err(why) = UserFan::update_user_fans_data_with_this_user(decoded_user.clone(), connection).await{
                    error!("can't update `users_fans` table due to: {}", why);
                };
                
                info!("finished updating `users_nfts` and `users_fans` tables");
                //----------------------------------------------------------------------
                //----------------------------------------------------------------------
                //----------------------------------------------------------------------

            }

        });

    }

}
