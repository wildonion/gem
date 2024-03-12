

/* 
     ------------------------------------------
    | --------- HEALTH COMPONENT ACTOR --------- 
    | ------------------------------------------
    | contains api structures and message handlers
    | to communicate locally and remotely
    |
*/

use crate::*;
use actix::Actor;
use actix::prelude::*;
use self::constants::AppState;
use crate::apis::health::{ComponentState, HealthComponentActor, Api};

// fn pointer method, futures must be pinned at a fixed position on the heap 
// to avoid getting invalidated pointers even after moving the type
type Method = fn(HttpRequest, AppState) -> std::pin::Pin<Box<dyn futures::Future<Output = PanelHttpResponse>>>;


// -----------------------------------
// messages used for communication
// -----------------------------------
#[derive(Clone, Message)]
#[rtype(result = "ApiResponse")]
pub struct ExecuteApi{ // execute an api available from the list of all registered apis
    pub route: String,
}

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

    pub fn new(apis: Vec<Api>) -> Self{
        Self{
            state: None,
            apis
        }
    }

    pub fn get_self(&self) -> Self{
        Self { state: self.clone().state, apis: self.clone().apis }
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
    pub async fn emit(){
        // redis and actix broker publication
        // ...
    }

    // redis/actixborker subscription process to subscribe to an specific channel 
    // contians emitted data from other apis publisher
    pub async fn subscribe(){
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

        // use actix telepathy to send message to other actors remotely
        // https://docs.rs/actix-telepathy/latest/actix_telepathy/#:~:text=To%20send%20messages%20between%20remote,RemoteAddr%20that%20the%20ClusterListener%20receives.&text=Now%2C%20every%20new%20member%20receives,every%20ClusterListener%20in%20the%20cluster.
        // ...

        todo!()

    }
}