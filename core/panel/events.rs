


/* ------------------------------- */
/* publisher and subscriber actors */
/* ------------------------------- */
/* 
    look at apis/erm.rs and apis/clp.rs routes for http ws streamers
    and https://github.com/wildonion/zoomate for tcp, rpc streamer
    handlers run in a separate threadpool
    
    streamer or event handler channels to notify app in realtime about 
    some data changes by sharing lazy static global mutex shared state 
    between different parts, route and threads of the app can be done 
    with interval loop{}, while let Some()... and actor workers like: 
          - tokio tcp listener as publisher
          - tonic rpc pubsub
          - actix ws stream handler and actor message handler as subscriber
          - libp2p gossipsub 
          - redis pubsub
          - actix borker pubsub
          - tokio::mpsc,select,spawn,mutex,arc
          - actix http webhook with stream: Payload and Multipart codec
          - codecs with protobuf and serde, Payload, Multipart
    
    >_ note: 
        we can run each publisher and subscriber actor in a separate 
        threadpool like tokio::spawn(async move{}) and start subscribing 
        to data changes topics using while let Some()... syntax in realtime 
        as well as publishing new topics using publisher
    
    >_ note:
        actors are threadpool workers that can be used to create concurrent and pubsub 
        based stream handlers for both server and client models like rpc as well as 
        communicate with different parts of the app and each other by sending async messages
        hence in actor based pubsub pattern server and client both can be listener, publisher, 
        subscriber and stream handlre actors at the same time:
            server can listen to streams over client requests like tokio tcp listener or stream: web::Payload in actix http
            client can listen to streams over server responses like redis client listener
        in our case publishers are server actors contains session or client actors that can:
            communicate with them using message sending pattern
            publish new topics on data changes to a channel
        on the other hand a subscriber is basically an stream of events or messages handler 
        which can be treated like a push notif handler by subsribing and streaming to incoming 
        notifs, events and data changes from server or publisher, in case of tcp server the 
        server is a listener which will stream over incoming bytes from clients
          
*/

pub mod publishers;
pub mod subscribers; 