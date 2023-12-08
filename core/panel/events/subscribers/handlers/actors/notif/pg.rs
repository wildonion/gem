

/*  > -----------------------------------------------------------------------------------------------------
    | pg listener actor to subscribe to tables changes notifs and communicate with other parts of the app 
    | -----------------------------------------------------------------------------------------------------
    | contains: message structures and their handlers
    | 
    |
    |
*/

use crate::*;
use crate::constants::{WS_CLIENT_TIMEOUT, WS_SUBSCRIPTION_INTERVAL};
use crate::misc::*;
use s3req::Storage;
use crate::*;
use actix::prelude::*;



#[derive(Clone, Message)]
#[rtype(String)]
pub struct GetLatestChanges {
    pub table_name: String
}

#[derive(Clone, Default)]
pub struct TableInfo{
    pub latest_record: String,
    pub table_name: String
}

/* 
    pg notif actor is a ds that will start subscribing to postgres event in 
    its interval loop using while let Some()... syntax in the background and realtime
    once it gets started, to notify other parts about tables changes by sending 
    the received event from redis through mpsc channels.
*/
#[derive(Clone, Default)]
pub struct PgListenerActor{
    pub tables: Vec<TableInfo>,
    pub app_storage: Option<Arc<Storage>>,
}

impl Actor for PgListenerActor{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        ctx.run_interval(WS_SUBSCRIPTION_INTERVAL, |actor, ctx|{
            
            let mut this = actor.clone();
            tokio::spawn(async move{
                this.subscribe().await;
            });
        });
    }
}

impl PgListenerActor{

    /* 
        pg streaming of events handler by subscribing to the related topic in an interval loop using 
        while let Some()... syntax and redis, in order to get new changes by sending GetLatestChanges 
        message from different parts of the app to this actor to get the latest table update as a response 
        of this actor, this can be done by starting the actor in place where we're starting the server 
        then share the actor as a shared state data like Arc<Mutex< between actix routers threads so we 
        can extract it from the app_data in each api and send the GetLatestChanges message to fetch new 
        updated record of the passed in table name
    */
    pub async fn subscribe(&mut self){
        
        tokio::spawn(async move{
            
            // subscribe to users_*_update topic using redis and while let Some()...
            // ...

        }); 

    }

}


/* 
    other parts of the app can communicate with this actor to get the 
    latest record update of the passed in table name, behind message 
    handlers of each actor are mpsc jobq channel which allows other 
    parts of the app to send data using the sender and receive the 
    response using receiver
*/
impl Handler<GetLatestChanges> for PgListenerActor{
    
    type Result = String;

    fn handle(&mut self, msg: GetLatestChanges, ctx: &mut Self::Context) -> Self::Result{

        let GetLatestChanges{ table_name } = msg;
        let tables = self.tables.clone();

        let mut found_tabel = TableInfo::default();
        if tables
            .into_iter()
            .any(|t|{
                
                if t.table_name == table_name{
                    found_tabel = t;
                    true
                } else{
                    false
                }

            }){

                found_tabel.latest_record
            } else{
                String::from("")
            }
        
    }

}