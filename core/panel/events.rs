


/* ------------------------------- */
/* publisher and subscriber actors */
/* ------------------------------- */
/* 
    look at apis/erm.rs and apis/clp.rs routes for http ws streamers
    and https://github.com/wildonion/zoomate for tcp, rpc streamers 
    
    streamer or event handler channels to notify app in realtime about 
    some data changes by sharing lazy static global mutex shared state 
    between different parts, route and threads of the app can be done 
    with interval loop{}, while let Some()... and: 
          - tokio tcp listener as publisher
          - tonic rpc pubsub
          - actix ws stream handler and actor message handler as subscriber
          - libp2p gossipsub 
          - redis pubsub
          - actix borker pubsub
          - tokio::mpsc,select,spawn,mutex,arc
          - actix http webhook with stream: Payload and Multipart codec
          - codecs with protobuf and serde, Payload, Multipart
*/

pub mod publishers;
pub mod subscribers;