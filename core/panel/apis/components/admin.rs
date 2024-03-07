

/* 
     ------------------------------------------
    | --------- ADMIN COMPONENT ACTOR --------- 
    | ------------------------------------------
    | contains api structures and message handlers
    |
*/

use crate::*;
use actix::Actor;
use actix::prelude::*;
use self::constants::AppState;

// fn pointer method, futures must be pinned at a fixed position on the heap 
// to avoid getting invalidated pointers even after moving the type
type Method = fn(HttpRequest, AppState) -> std::pin::Pin<Box<dyn futures::Future<Output = PanelHttpResponse>>>;


#[derive(Clone, Message)]
#[rtype(result = "ApiResponse")]
pub struct ExecuteApi{ // execute an api available from the list of all registered apis
    pub route: String,
}

#[derive(MessageResponse)]
pub struct ApiResponse(pub PanelHttpResponse);

#[derive(Clone)]
pub enum ComponentState{
    Halted,
    Executed,
}

#[derive(Clone)]
pub struct Api{
    pub route: String, 
    pub method: Method,
    pub last_response: Option<serde_json::Value> // last response json value caught throughout the api calling
}

#[derive(Clone)]
pub struct AdminComponentActor{
    pub state: Option<ComponentState>,
    pub apis: Vec<Api>
}

impl Actor for AdminComponentActor{

    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        
        info!("AdminComponentActor -> started");
    }
}

impl AdminComponentActor{

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

    }

    // redis subscription process to subscribe to an specific channel 
    // contians emitted data from other apis publisher
    pub async fn subscribe(){

    }

}

impl Handler<ExecuteApi> for AdminComponentActor{
    type Result = ApiResponse;

    /*  
        1) other apis can send ExecuteApi message to this actor to execute an specific route 
        2) once the api gets executed its http response will back to the caller
        3) the response of the executed api will be cached inside the api state
    */
    fn handle(&mut self, msg: ExecuteApi, ctx: &mut Self::Context) -> Self::Result {

        todo!()

    }
}