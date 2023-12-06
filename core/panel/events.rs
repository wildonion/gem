


/* ------------------------------- */
/* publisher and subscriber actors */
/* ------------------------------- */
/* 
    look at apis/role.rs and apis/clp.rs routes for http ws streamers
    and https://github.com/wildonion/zoomate for tcp, rpc streamers 
    
    streamer or event handler channels to notify app about in realtime
    some data changes using lazy static global mutex shared state 
    data can be done with interval loop{}, while let Some()... and: 
          - tokio tcp listener as publisher
          - tonic rpc listener as publisher
          - actix ws stream handler and actor message handler as subscriber
          - libp2p gossipsub 
          - redis pubsub
          - tokio::mpsc,select,spawn,mutex,arc
          - actix http webhook with stream: Payload codec
*/

pub mod publishers;
pub mod subscribers;