



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
            • actor based pubsub workers in server/client (like tcp,tonic,http) for realtime streaming over receiver/subscriber and monitoring like grafana
            • start actors globally in a place when the server is being built
            • shared the started actor between threads as an app data state in this case the data must be Arc<Mutex<Actor>>
            • initialize a global in memory map based db using static Lazy<Arc<Mutex<Actor>>> send sync 'static
            • local pubsub pattern (using actix actor worker and the broker crate)
                publisher actor  ➙ publish/fire/emit/trigger event data using actix broker 
                subscriber actor ➙ subscribe to incoming data from publisher in the interval in tokio::spawn while let some and mpsc
            • redis pubsub pattern
                publisher actor  ➙ publish/fire/emit/trigger event data using redis actor in the interval then break once a subscriber receives it
                subscriber actor ➙ subscribe to incoming data from redis in the interval in tokio::spawn while let some and mpsc
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