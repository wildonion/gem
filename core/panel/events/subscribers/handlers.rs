



/* 
    actors have threadpool worker, async task stream handler, mpsc channel,
    pubsub borker and interval loop in their own structure so we can create 
    each handler as an actor which will start subscribing to incoming events 
    from different sources like database or websocket in realtime
*/
pub mod actors;