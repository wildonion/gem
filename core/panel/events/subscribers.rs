




/* 
    >----------------------------------------------------------------------------------------------
    | subscribers are hook/event/stream/message listeners and handlers in which they must 
    | handle the specific incoming stream of events or messages through a channel (tcp, rpc, mpsc) 
    | in realtime by subscribing to the related topics to show the event as a notification to 
    | different parts of the app by sending it through mpsc channels or waiting to receive new 
    | message request from other parts of the app to response them with the received event, GTK 
    | (good to know) that actors have pubsub patterns and event handlers in their own structure, 
    | each session actor can be used to communicate using pre defined mesaages to send message 
    | to other actors also server actor contains all session actor in which we can use the message 
    | sending pattern for pubsub push notif subscribing to an specific topic using redis in server 
    | actor, like if we want to notify all sessions about a new session since the server actor has 
    | all sessions we should send a new session income message to the server actor once a new 
    | session gets connected to the socket and server can send to all session actors the new 
    | income message 

*/

pub mod handlers; // handler ws and notif actors