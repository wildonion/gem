


/* ------------------------------- */
/* publisher and subscriber actors */
/* ------------------------------- */
// tools => tokio,actix,redis,libp2p,ipfs,mpsc,macro dsl
/* 

    • generally pubsub realtime monitoring, streaming and push notification over an mpsc receiver/redis subscriber/tcp listener 
        can be done inside actor worker interval inside tokio::spawn() using while let some syntax
    • actor based pubsub workers in server/client (like tcp,tonic,http) for realtime streaming over receiver/subscriber and monitoring like grafana
    • start actors globally in a place when the server is being built
    • share the started actor between threads as an app data state in this case the data must be Arc<Mutex<Actor>> + Send + Sync + 'static
    • initialize a global in memory map based db using static Lazy<Arc<Mutex<Actor>>> send sync 'static
    • local pubsub pattern (using actix actor worker and the broker crate with mpsc channel)
        publisher actor  ➙ publish/fire/emit/trigger event data using actix broker 
        subscriber actor ➙ stream/subscribe over/to incoming message data from publisher in an interval in tokio::spawn while let some and mpsc
    • redis pubsub pattern
        publisher actor  ➙ publish/fire/emit/trigger event data using redis actor in an interval then break once a subscriber receives it
        subscriber actor ➙ stream/subscribe over/to incoming stringified data from redis in an interval in tokio::spawn while let some and mpsc
    • tokio tcp streaming pattern
        publisher actor  ➙ publish/fire/emit/trigger event data using tokio tcp client actor
        subscriber actor ➙ stream/subscribe over/to incoming utf8 data from client in an interval in tokio::spawn while let some and mpsc
    • actix ws http streaming pattern
        publisher actor  ➙ publish/fire/emit/trigger event data using ws client actor
        subscriber actor ➙ stream/subscribe over/to incoming stream: Payload, payload: Multiaprt data from client in an interval in tokio::spawn while let some and mpsc
    • http api must be triggered by frontend every 5 seconds in which we send message to subscriber actor worker to 
        get all user notifications from redis and send it as the json response back to the caller


    look at apis/erm.rs and apis/clp.rs routes for http ws streamers
    and https://github.com/wildonion/zoomate for tcp, rpc streamer
    handlers, each, ran in a separate threadpool to start listening
    
    >_ note: 
        streamer or event handler channels to notify app in realtime about 
        some data changes by sharing global static lazy arced mutex data
        between different parts, route and threads of the app can be done 
        with actor workers in their interval loop{} with while let Some()... 
        inside tokio::spawn to start subscribing and listening to incoming 
        packets from the following tlp sources: 
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
            client can listen to streams over server responses like redis and rpc client listener
        in our case publishers are server actors contains session or client actors that can:
        communicate with them using message sending pattern publish new topics on data changes 
        to a channel, a subscriber on the other hand, is basically an stream of events or messages 
        handler which can be treated like a push notif handler by subsribing and streaming to
        incoming notifs, events and data changes from server or publisher, in case of tcp server 
        the server is a listener which will stream over incoming bytes from clients
    
    >_ note:
        server/client actor can stream over client/server across requests/responses  
        inside tokio spawn using while let some also in subscription actor we can call 
        update hook apis once we received the notification from the source channel like 
        redis and in publisher actor we can send a message from different parts of the 
        app to tell the actor publish the data or notification to the source 
        channel like redis
          
*/

pub mod publishers;
pub mod subscribers; 