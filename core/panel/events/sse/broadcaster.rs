

use crate::*;
use self::models::clp_events::ClpEvent;


// -0--0--0--0--0--0--0--0--0--0-
//    sse broadcaster struct 
// -0--0--0--0--0--0--0--0--0--0-
/* 
    Server-Sent Events is a part of the HTML5 specification that enables 
    servers to push data to web clients over a single, long-lived connection. 
    Unlike WebSocket, which facilitates full-duplex communication, SSE is ideal 
    for scenarios where one-way communication from server to client is required.
    SSE relies on the EventSource API on the client side, which allows the browser 
    to establish a persistent connection to a server endpoint. Once connected, 
    the server can send events to the client as simple text data, typically in a 
    format called "text/event-stream." The client-side JavaScript can then handle 
    these events and update the web page in real-time.
*/

// add broadcaster struct to app state so we can share it between threads
// broadcast new clp event

#[derive(Debug, Clone, Default)]
pub struct Broadcaster{
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Event<E>{
    pub data: E
}
 
impl Broadcaster{

    pub fn new() -> Self where Self: Sized{

        todo!()
        
    }

    pub async fn get_clients(){

    }

    pub async fn add_client(){

    }

    pub async fn broadcast<E>(topic: &str, event: Event<E>){

    } 

    pub async fn get_clp_event(){

    }

    pub async fn get_event_future<E>() -> std::pin::Pin<Box<dyn futures::Future<Output=Event<E>>>>{
        
        todo!()
    
    }
    
}