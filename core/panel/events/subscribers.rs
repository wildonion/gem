




/* 
    >----------------------------------------------------------------------------------------------
    | subscribers are hook/event/stream/message listeners and handlers in which they must 
    | handle the specific incoming stream of events through a channel (tcp, rpc, mpsc) or 
    | messages in realtime by subscribing to the related topics to show the event as a 
    | notification to different parts of the app or other services, GTK (good to know) that 
    | actors have pubsub patterns and event handlers in their own structure, each session actor 
    | can be used to communicate using pre defined mesaages to send message to other actors 
    | also server actor contains all session actor in which we can use the message sending 
    | pattern for pubsub push notif subscribing to an specific topic using redis in server actor, 
    | like if we want to notify all sessions about a new session since the server actor has all 
    | sessions we should send a new session income message to the server actor once a new session 
    | gets connected to the socket and server can send to all session actors the new income message 


    streamer or event handler or trigger channels to notify app about 
    some data changes using lazy static global mutex shared state 
    data can be done with interval loop{} and: 
          - tokio tcp listener
          - tonic rpc listener
          - actix ws stream handler and actor message handler 
          - libp2p gossipsub 
          - redis pubsub
          - tokio::mpsc,select,spawn,mutex,arc
          - actix http webhook

*/

pub mod handlers;