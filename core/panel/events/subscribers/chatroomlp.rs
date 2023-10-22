



use crate::*;


// chatroom launchpad peer subscribers 
// https://github.com/actix/examples/tree/master/websockets
// stream/wh/event handler/loop using actix-ws-web/tokio stuffs/rpc/libp2p

// chat session actor: 
//      an actor that will handle ws streams from client
//      coming from the client
// chat server actor: 
//      an actor that session actors will communicate 
//      with and send message to it to do log
//      to do code logic between different parts of the app
// tokio tcp based server runs in a tokio::spawn()
// actix web http+ws server runs in the same thread that actix has ran
// client based server 
// actor broker

pub async fn start_tcp_server(){

    tokio::spawn(async move{

        let listener = tokio::net::TcpListener::bind("0.0.0.0:2324").await.unwrap();
            
    });

}