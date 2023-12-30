



use crate::{*, constants::WS_SUBSCRIPTION_INTERVAL, events::publishers::user::{UserNotif, NotifExt, NotifData}};
use actix::prelude::*;
use s3req::Storage;
use crate::events::subscribers::handlers::actors::notif::system::SystemActor;


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

    pub fn new(app_storage: Option<Arc<Storage>>, system_actor: Addr<SystemActor>) -> Self{

        UserActionActor{
            users_notifs: None,
            app_storage,
            system_actor
        }
    }

    pub async fn redis_subscribe(&mut self, app_storage: Arc<Storage>,
        redis_async_pubsubconn: Arc<PubsubConnection>){


        /* 
            ----------------------------------------------------
            ----tokio,actix,redis,libp2p,ipfs,mpsc,macro dsl----
            ----------------------------------------------------
            pubsub realtime monitoring, streaming and push notification over a receiver/subscriber/listener with:
            • actor based pubsub workers in server/client (like tcp,tonic,http) for realtime streaming over receiver/subscriber and monitoring like grafana
            • start actors globally in a place when the server is being built
            • share the started actor between threads as an app data state in this case the data must be Arc<Mutex<Actor>> + Send + Sync + 'static
            • initialize a global in memory map based db using static Lazy<Arc<Mutex<Actor>>> send sync 'static
            • local pubsub pattern (using actix actor worker and the broker crate with mpsc channel)
                publisher actor  ➙ publish/fire/emit/trigger event data using actix broker 
                subscriber actor ➙ stream/subscribe over/to incoming message data from publisher in an interval in tokio::spawn while let some and mpsc
            • redis pubsub pattern
                publisher actor  ➙ publish/fire/emit/trigger event data using redis actor in an interval then break once a subscriber receives it
                subscriber actor ➙ stream/subscribe over/to incoming stringified data from redis in an interval in tokio::spawn while let some and mpsc
            • tokio tcp streaming pattern
                publisher actor  ➙ publish/fire/emit/trigger event data using tokio tcp client actor
                subscriber actor ➙ stream/subscribe over/to incoming utf8 data from client in an interval in tokio::spawn while let some and mpsc
            • actix ws http streaming pattern
                publisher actor  ➙ publish/fire/emit/trigger event data using ws client actor
                subscriber actor ➙ stream/subscribe over/to incoming stream: Payload, payload: Multiaprt data from client in an interval in tokio::spawn while let some and mpsc
            • http api must be triggered by frontend every 5 seconds in which we send message to subscriber actor worker to 
              get all user notifications from redis and send it as the json response back to the caller

        */

        // start subscribing to the user_actions topic
        // inside tokio::spawn() using while let Some 
        // send the received data to mpsc channel to 
        // receive it outside of the tokio::spawn()
        // ...

        // some how resset all user notifs or get those onse 
        // which have been fired recently, sort by fired at

        // update self.users_notifs

        

        let notif_data = NotifData::default();
        let mut user_notif = UserNotif::default();
        user_notif.set_user_notif(notif_data);
        
    }

}

impl Handler<GetUsersNotifsMap> for UserActionActor{
    
    type Result = UsersNotifs;

    fn handle(&mut self, msg: GetUsersNotifsMap, ctx: &mut Self::Context) -> Self::Result {
        
        let users_notifs = self.users_notifs.clone();
        UsersNotifs(users_notifs)
    }
}