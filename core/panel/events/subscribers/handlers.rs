



/* >------------------------------------------------------------------------------------- 
    handlers are actor based threadpool workers which will start in the background
    to start subscribing to incoming streams of events from different sources in 
    realtime they also have threadpool worker, async task stream handler, mpsc channel,
    pubsub borker and interval loop in their own structure which allows us to create 
    each handler as an actor so other parts of the app be able to communicate with 
    them through message passing pattern by sending the related message request to
    get fetch some data from the actor as the actor response.
*/
pub mod actors;