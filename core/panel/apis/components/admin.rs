

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


#[derive(Clone)]
pub enum ComponentState{
    Halted,
    Executed,
}

// fn pointer method
type Method = fn(HttpRequest, AppState) -> Box<dyn futures::Future<Output = PanelHttpResponse>>;
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

}

// message handlers to run an api of this component
// send back the result to the communicator
// ...