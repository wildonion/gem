



/* 
    >-----------------------------------------------------------------------------------
    notif handlers can be actors that can be started in the background once the server 
    gets started, they can start subscribing either to redis topics or pg notif trigger 
    in an interval, then we can communicate with them by sending message and call their 
    methods directly to get the data constantly also the reason why we're using actor 
    instead of simple worker is because they have mailbox, multithreaded based async 
    message handlers and pubsub pattern in their own structure, simply we can subscribe 
    to an incoming events from a tcp source using while let Some()... syntax inside a 
    tokio::spawn(async move) then send the incoming Arc<Mutex<Data to an mpsc channel 
    so other parts of the app can receive it, with actor this will be done my message 
    sending pattern (which uses mpsc behind the scence) and asyncly in the background

    note that in order to share a data for mutation between threads it must be 
    Arc<Mutex<Data>> + Send + Syn + 'static and if we want to have some global
    like data to be initialized once we can wrap Arc<Mutex<Data>> in Lazy like 
    STORAGE inside constants.rs
*/
pub mod clp;
pub mod user;
pub mod action;
pub mod system;