




use actix::{AsyncContext, Context};
use s3req::Storage;
use crate::*;
use self::constants::WS_SUBSCRIPTION_INTERVAL;


#[derive(Clone)]
pub struct ClpEventSchedulerActor{
    pub app_storage: Option<Arc<Storage>>,
}


impl Actor for ClpEventSchedulerActor{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        
        info!("ClpEventSchedulerActor -> started cron scheduler interval");

        // running a 5 seconds interval to check the current event status
        // and if it was expired or finished start the process of summarizing 
        // users chats, generating and minting ai pictures
        ctx.run_interval(WS_SUBSCRIPTION_INTERVAL, |actor, ctx|{
            
            let storage = actor.app_storage.clone();
            let this = actor.clone();
            tokio::spawn(async move{
                this.check_event_status(storage.unwrap()).await;
            });

        });
        
    }

}

impl ClpEventSchedulerActor{

    pub fn new(app_storage: Option<Arc<Storage>>) -> Self{

        ClpEventSchedulerActor{
            app_storage,
        }
    }

    /* ------------------------------------------------------------ */
    /* --------------- EVENT ACTOR WROKER SCHEDULER --------------- */
    /* ------------------------------------------------------------ */
    // once the actor gets started we'll execute this methiod 
    // every 5 seconds constantly to check the followings:
    pub async fn check_event_status(&self, app_storage: Arc<Storage>){

        let pg_pool = app_storage.get_pgdb().await.unwrap();
        let connection = &mut pg_pool.get().unwrap();

        tokio::spawn(async move{
            
            /*     
                1 - check that the current and latest event is expired or not if now > clp_event.expire_at then start generating titles, images and mint them
                    in this case:
                        1 - summerize users' chats and generate n titles
                        2 - generate nft based images for those titles
                        3 - generate a mapping between titles and images using ai
                        4 - store all generated nfts + metadata on ipfs, then update collection base_uri finally store nfts in db 
                        5 - mint ai generated pictures to users screen_cids inside the chat by calling contract ABI
                2 - lock the event if now > clp_event.start_at then lock the event so they can't register for the event 
            */

            // ...

        });

    }

}