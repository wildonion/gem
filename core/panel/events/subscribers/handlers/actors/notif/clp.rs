




use actix::{AsyncContext, Context};
use s3req::Storage;
use crate::{constants::WS_CLP_EVENT_SUBSCRIPTION_INTERVAL, *};
use self::{constants::WS_SUBSCRIPTION_INTERVAL, models::clp_events::{ClpEvent, ClpEventData}};


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
        ctx.run_interval(WS_CLP_EVENT_SUBSCRIPTION_INTERVAL, |actor, ctx|{
            
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

        // note that some ClpEvent methods doesn't return actix response in their
        // error part since we're using tokio::spawn() and can't transfer actix 
        // http response between tokio::spawn since it's not Send and Sync
        tokio::spawn(async move{
            let pg_pool = app_storage.get_pgdb().await.unwrap();
            let connection = &mut pg_pool.get().unwrap();
            
            let get_latest_clp_event = ClpEvent::get_latest_without_actix_response(connection).await;
            if get_latest_clp_event.is_err(){
                error!("error in getting latest clp event info due to: {:?}", get_latest_clp_event.as_ref().unwrap_err());
            }
            
            /* -------------------------------------------------------------------------------- */
            /* ---------------------- check that event is expired or not ---------------------- */
            /* -------------------------------------------------------------------------------- */
            let latest_clp_event_info = get_latest_clp_event.unwrap_or(ClpEventData::default());
            if chrono::Local::now().timestamp() > latest_clp_event_info.expire_at{ // event is expired so let's rewarding :)
                
                // start summarizing chats, generating titles and images and finally mint them
                // to all participant in-app evm wallet inside the event
                let reward_them = ClpEvent::distribute_rewards(latest_clp_event_info.id, app_storage).await;
                if let Err(why) = reward_them{
                    error!("can't reward participants due to {:?}", why);
                }
            }
            
            /* -------------------------------------------------------------------------------- */
            /* ---------------------- check that event is started or not ---------------------- */
            /* -------------------------------------------------------------------------------- */
            if chrono::Local::now().timestamp() > latest_clp_event_info.start_at{
    
                // lock the event so users can't register for the event
                let updated_clp_event = ClpEvent::lock_event(latest_clp_event_info.id, connection).await;
                if let Err(why) = updated_clp_event{
                    error!("can't lock the clp event due to {:?}", why);
                }
            }
        });

    }

}