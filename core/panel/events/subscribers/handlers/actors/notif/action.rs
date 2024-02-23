


/*  > ---------------------------------------------------------
    | user notif listener actor to subscribe to new user notif  
    | ---------------------------------------------------------
    | contains: message structures and their handlers
    | usage: front can get new notif data from this actor by calling /profile/notifs/get api
    |
*/

use crate::{*, constants::{WS_SUBSCRIPTION_INTERVAL, STORAGE_IO_ERROR_CODE}, events::publishers::action::{UserNotif, NotifExt, NotifData, SingleUserNotif}, models::users::User};
use actix::prelude::*;
use s3req::Storage;
use crate::events::subscribers::handlers::actors::notif::system::SystemActor;
use redis_async::resp::FromResp;


#[derive(Clone, Message)]
#[rtype(result = "UsersNotifs")]
pub struct GetUsersNotifsMap;

#[derive(MessageResponse)]
pub struct UsersNotifs(pub Option<HashMap<i32, UserNotif>>);


// -------------------------------------
/* user notif subscriber actor worker */
// -------------------------------------
#[derive(Clone)]
pub struct UserActionActor{
    pub users_notifs: Option<HashMap<i32, UserNotif>>,
    pub app_storage: Option<Arc<Storage>>,
    pub system_actor: Addr<SystemActor>,
}


impl Actor for UserActionActor{
    
    // actors run within a specific execution context Context<A>
    // the context object is available only during execution or ctx 
    // each actor has a separate execution context the execution 
    // context also controls the lifecycle of an actor.
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context){
        
        info!("UserActionActor -> started subscription interval");

        /* start subscription interval in tokio::spawn() using while let Some()... syntax */
        // ctx.run_interval(WS_SUBSCRIPTION_INTERVAL, |actor, ctx|{

            // self must be cloned to prevent self from moving 
            // since it's going to be moved into tokio::spawn() scope
            let mut this = self.clone(); 
            let app_storage = self.app_storage.clone().unwrap();
            let redis_pubsub_async = app_storage.clone().get_async_redis_pubsub_conn_sync().unwrap();

            // start subscribing to redis `user_actions` topic
            tokio::spawn(async move{

                this.redis_subscribe(app_storage, 
                    redis_pubsub_async
                ).await;

            });

        // });

    }
    
}

impl UserActionActor{

    pub fn new(app_storage: Option<Arc<Storage>>, system_actor: Addr<SystemActor>) -> Self{

        UserActionActor{
            users_notifs: None,
            app_storage,
            system_actor
        }
    }

    /* 
        none-actor approach for subscription: 
        following method is not necessary to be executed inside the started() method,
        actually there is no need to use an actor for this, the subscription can simply 
        be started inside the tokio::spawn() an let it be there in the background in 
        an async function the we could have executed the method inside another tokio::spawn
        where the server is being started but the convenient way is to use actor worker 
        for this cause actors are great structures to deal with executing and scheduling 
        tasks asyncly and concurrently in different parts of the app, hopefully the have 
        some mailbox receiver for communicating with other actors.

        pub async fn redis_subscribe(){

            tokio::spawn(async move{

                // subscription logic
                // ...

            });

        }

        // put this where the server is being started
        tokio::spawn(async move{
            redis_subscribe().await;
        });
    */
    pub async fn redis_subscribe(&mut self, app_storage: Arc<Storage>,
        redis_async_pubsubconn: Arc<PubsubConnection>){

        let pool_conn = app_storage.get_pgdb().await;
        let connection = &mut pool_conn.unwrap().get().unwrap();
        let redis_async_pubsubconn = redis_async_pubsubconn.clone();
        let redis_client = app_storage.as_ref().get_redis().await.unwrap();
        let mut redis_conn = redis_client.get_async_connection().await.unwrap();

        let (user_notif_event_sender, mut user_notif_event_receiver) = 
            tokio::sync::mpsc::channel::<SingleUserNotif>(1024);

        /* 
            on user action subscription process is done using the redis async subscriber inside a tokio 
            threadpool which subscribes asyncly to the incoming future io object streams 
            from the passed in channel that contains the action data info
        */
        tokio::spawn(async move{

            /* 🚨 !!! 
                we must receive asyncly from the redis subscription streaming 
                channel otherwise actor gets halted in here since using sync 
                redis and actor redis cause the current thread gets halted
                because they'll receive in a blocking manner, thus we must 
                use tokio::spawn() to do so.

                subscribing to on_user_action pubsub channel asyncly and constantly using async redis
                by streaming over the incoming future tasks topics to decode the published topics
            
            !!! 🚨 */
            let get_stream_messages = redis_async_pubsubconn
                .subscribe("on_user_action")
                .await;

            let Ok(mut pubsubstreamer) = get_stream_messages else{

                use helpers::error::{ErrorKind, StorageError::RedisAsync, PanelError};
                let e = get_stream_messages.unwrap_err();
                let error_content = e.to_string().as_bytes().to_vec();  
                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(RedisAsync(e)), "UserActionActor::redis_subscribe");
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
                let decoded_user_notif = serde_json::from_str::<SingleUserNotif>(&stringified_update_user).unwrap();

                info!("got user notif payload on `on_user_action` channel at time {}", chrono::Local::now().timestamp());
                
                // sending the decoded data into an mpsc jobq channel to update 
                // redis outside of tokio::spawn() threadpool
                if let Err(why) = user_notif_event_sender.send(decoded_user_notif).await{
                    error!("can't send user notif sender due to: {}", why.to_string());
                }

            }

        });

        // streaming over mpsc jobq channel to receive notif data constantly
        // from the sender and then update the redis later 
        while let Some(notif_data) = user_notif_event_receiver.recv().await{

            let user_id = {
                let scid = notif_data.clone().wallet_info.screen_cid.unwrap_or(String::from(""));
                let get_user = User::find_by_screen_cid(&scid, connection).await;
                let user = get_user.unwrap_or(User::default());
                user.id
            };

            let redis_key = format!("user_notif_{}", user_id);

            // reading from redis
            let get_users_notifs: String = redis_client.clone().get(redis_key.clone()).unwrap_or(
                serde_json::to_string_pretty(&UserNotif::default()).unwrap()
            );
            let mut user_notifs = serde_json::from_str::<UserNotif>(&get_users_notifs).unwrap();

            let updated_user_notif = user_notifs
                .set_user_notif(notif_data.notif)
                .set_user_wallet_info(notif_data.wallet_info);
            
            // updating/caching in redis, later on we can read the 
            // entire notif data from redis in notification api
            let stringified_user_notif = serde_json::to_string_pretty(&updated_user_notif).unwrap();
            let ـ : RedisResult<String> = redis_conn.set(redis_key, stringified_user_notif).await;

        }
        
    }

}

impl Handler<GetUsersNotifsMap> for UserActionActor{

    type Result = UsersNotifs;

    fn handle(&mut self, msg: GetUsersNotifsMap, ctx: &mut Self::Context) -> Self::Result {
        
        todo!()

    }
}