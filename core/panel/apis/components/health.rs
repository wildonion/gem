

/* 
     ------------------------------------------
    | --------- HEALTH COMPONENT ACTOR --------- 
    | ------------------------------------------
    | contains api structures and message handlers
    | to communicate locally
    |
*/

use crate::*;
use actix::Actor;
use actix::prelude::*;
use s3req::Storage;
use self::constants::AppState;
use crate::apis::health::{ComponentState, HealthComponentActor, Api};



#[derive(Clone, Message)]
#[rtype(result = "ApiResponse")]
pub struct ExecuteApi{ // execute an api available from the list of all registered apis
    pub route: String,
}

// -----------------------------------
// messages used for communication
// -----------------------------------
#[derive(MessageResponse)]
pub struct ApiResponse(pub PanelHttpResponse);

impl Actor for HealthComponentActor{

    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        
        info!("HealthComponentActor -> started");
    }
}

// -----------------------------------
// health component implementations
// -----------------------------------
impl HealthComponentActor{

    pub fn new(apis: Vec<Api>, app_storage: Option<Arc<Storage>>) -> Self{
        Self{
            state: None,
            apis,
            app_storage
        }
    }

    pub fn get_self(&self) -> Self{
        Self { state: self.clone().state, apis: self.clone().apis, app_storage: self.clone().app_storage }
    }

    pub fn set_state(&mut self, new_state: ComponentState) -> Self{
        self.state = Some(new_state);
        self.get_self()
    }

    pub fn add_api(&mut self, api: Api) -> Self{

        self.apis.push(
            api
        );

        self.get_self()

    }

    // emit/fire/publish an event into a redis pubsub channel 
    // so other apis can subscribe to it
    pub async fn emit(&mut self){
        // redis and actix broker publication
        // ...
    }

    // redis/actixborker subscription process to subscribe to an specific channel 
    // contians emitted data from other apis publisher
    pub async fn subscribe(&mut self){
        // redis and actix broker subscription
        // ...
    }

}


// -----------------------------------
// local and remote message handlers
// -----------------------------------
impl Handler<ExecuteApi> for HealthComponentActor{
    type Result = ApiResponse;

    /*  
        1) other apis can send ExecuteApi message to this actor to execute an specific route 
        2) once the api gets executed its http response will back to the caller
        3) the response of the executed api will be cached inside the api state


        let res = HealthComponentActor.send(
            ExecuteApi{
                route: "/nft/get"
            }
        ).await;

    */
    fn handle(&mut self, msg: ExecuteApi, ctx: &mut Self::Context) -> Self::Result {

        todo!()

    }
}