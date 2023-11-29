




/* 
    ws servers and peers contains redis async subscribers to 
    subscribe to redis topics and notifs 

    session actor: 
         an actor that will handle ws streams from client
         also it has messages to communicate with server actor
    server actor: 
         an actor that session actors will communicate 
         with and send message to it to do code logic 
         between different parts of the app
*/

pub mod notifs;
pub mod session;
pub mod chatroomlp;
pub mod sessionlp;