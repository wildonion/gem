


/*  > ---------------------------------------------------------
    | user notif listener actor to subscribe to new user notif  
    | ---------------------------------------------------------
    | contains: message structures and their handlers
    | usage: front can get new notif data from this actor by calling /profile/notifs/get api
    |
*/

use crate::{*, constants::{WS_SUBSCRIPTION_INTERVAL, STORAGE_IO_ERROR_CODE}, events::publishers::user::{UserNotif, NotifExt, NotifData, SingleUserNotif}, models::users::User};
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
    
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context){
        
        info!("UserActionActor -> started subscription interval");

        /* start subscription interval in tokio::spawn() using while let Some()... syntax */
        // ctx.run_interval(WS_SUBSCRIPTION_INTERVAL, |actor, ctx|{

            let mut this = self.clone();
            let app_storage = self.app_storage.clone().unwrap();
            let redis_pubsub_async = app_storage.get_async_redis_pubsub_conn_sync().unwrap();

            // start subscribing to redis `user_actions` topic
            tokio::spawn(async move{

                this.redis_subscribe(app_storage, redis_pubsub_async).await;

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

    pub async fn redis_subscribe(&mut self, app_storage: Arc<Storage>,
        redis_async_pubsubconn: Arc<PubsubConnection>){

        let pool_conn = app_storage.get_pgdb().await;
        let connection = &mut pool_conn.unwrap().get().unwrap();
        let redis_async_pubsubconn = redis_async_pubsubconn.clone();
        let mut users_notifs_data = self.users_notifs.clone();

        let (user_notif_sender, mut user_notif_receiver) = 
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

                subscribing to on_user_update pubsub channel asyncly and constantly using async redis
                by streaming over the incoming future tasks topics to decode the published topics
            
            !!! 🚨 */
            let get_stream_messages = redis_async_pubsubconn
                .subscribe("on_user_action")
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
                let decoded_user_notif = serde_json::from_str::<SingleUserNotif>(&stringified_update_user).unwrap();

                info!("got user notif payload on `on_user_action` channel at time {}", chrono::Local::now().timestamp());
    
                if let Err(why) = user_notif_sender.send(decoded_user_notif).await{
                    error!("can't send user notif sender due to: {}", why.to_string());
                }

            }

        });

        while let Some(notif_data) = user_notif_receiver.recv().await{

            let user_id = {
                let scid = notif_data.clone().wallet_info.screen_cid.unwrap_or(String::from(""));
                let get_user = User::find_by_screen_cid(&scid, connection).await;
                let user = get_user.unwrap_or(User::default());
                user.id
            };

            let mut user_notif = UserNotif::default();
            let updated_user_notif = user_notif
                .set_user_notif(notif_data.notif)
                .set_user_wallet_info(notif_data.wallet_info);
            if users_notifs_data.is_some(){
                users_notifs_data.clone().unwrap().insert(
                    user_id,
                    updated_user_notif
                );
            }

        }

        // update users_notifs data of this actor with the latest notif
        self.users_notifs = users_notifs_data;
        
    }

}

/* 
    this handler will be used to send message to this actor from /profile/notifs/get api 
    to receive the user notifs
*/
impl Handler<GetUsersNotifsMap> for UserActionActor{
    
    type Result = UsersNotifs;

    fn handle(&mut self, msg: GetUsersNotifsMap, ctx: &mut Self::Context) -> Self::Result {
        
        let users_notifs = self.users_notifs.clone();
        UsersNotifs(users_notifs)
    }
}