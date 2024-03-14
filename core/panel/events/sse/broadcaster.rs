

use crate::*;


// -0--0--0--0--0--0--0--0--0--0-
//    sse broadcaster struct 
// -0--0--0--0--0--0--0--0--0--0-

// add broadcaster struct to app state so we can share it between threads
// broadcast new clp event

#[derive(Debug, Clone, Default)]
pub struct Broadcaster{
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Event{

}

impl Broadcaster{

    pub fn new() -> Self{

        todo!()
        
    }

    pub async fn get_clients(){

    }

    pub async fn add_client(){

    }

    pub async fn broadcast(topic: &str, event: Event){

    } 

    pub async fn get_clp_event(){

    }

    pub async fn get_event_future() -> std::pin::Pin<Box<dyn futures::Future<Output=Event>>>{
        
        todo!()
    
    }
    
}