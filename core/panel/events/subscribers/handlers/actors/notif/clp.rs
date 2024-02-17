




use actix::{AsyncContext, Context};
use s3req::Storage;
use crate::*;
use self::{constants::WS_SUBSCRIPTION_INTERVAL, models::clp_events::ClpEvent};


#[derive(Clone)]
pub struct ClpEventSchedulerActor{
    pub app_storage: Option<Arc<Storage>>,
}


impl Actor for ClpEventSchedulerActor{
    
    // actors run within a specific execution context Context<A>
    // the context object is available only during execution or ctx 
    // each actor has a separate execution context the execution 
    // context also controls the lifecycle of an actor.
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        
        info!("ClpEventSchedulerActor -> started cron scheduler interval");

        // running a 5 seconds interval to check the current event status
        // and if it was expired or finished we would have to start the 
        // process of summarizing users chats, generating and minting 
        //openai pictures
        ctx.run_interval(WS_SUBSCRIPTION_INTERVAL, |actor, ctx|{
            
            let storage = actor.app_storage.clone();
            let this = actor.clone();
            tokio::spawn(async move{ // executing in the background asyncly without having any disruption in other async method order of executions
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
        
        let get_latest_clp_event = ClpEvent::get_latest_without_actix_response(connection).await;
        if get_latest_clp_event.is_err(){
            error!("error in getting latest clp event info due to: {:?}", get_latest_clp_event.as_ref().unwrap_err());
        }

        let latest_clp_event_info = get_latest_clp_event.unwrap();
        if chrono::Local::now().timestamp() > latest_clp_event_info.expire_at{ // event is expired so let's start generating :)
            
            // start generating titles, images and mint them
            // in this case:
            //     1 - summerize users' chats and generate n titles
            //     2 - generate nft based images for those titles
            //     3 - generate a mapping between titles and images using ai
            //     4 - store all generated nfts + metadata on ipfs, then update collection base_uri finally store nfts in db 
            //     5 - mint ai generated pictures to users screen_cids inside the chat by calling contract ABI
            // ... 
        }

        if chrono::Local::now().timestamp() > latest_clp_event_info.start_at{

            // lock the event so users can't register for the event
            let updated_clp_event = ClpEvent::lock_event(latest_clp_event_info.id, connection).await;
            if let Err(why) = updated_clp_event{
                error!("can't lock the clp event due to {:?}", why);
            }
        }

    }

}