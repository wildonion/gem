


use crate::*;

// all user nft, collection, gallery, like, comment, dislike, ... related notifs
// user unclaimed gifts notifs

// step1
// publish user_actions notif into redis pubsub channel once a user has 
// liked, commnted, created nft and collection or even unclaimed gifts then 
// we'll start this actor to subscribe to the incoming notif from redis 
// pubsub channel like pg.rs

// step2
// there must be an http api call to be called from the frontend in an interval 
// once it gets hooked in the server we'll send a message to this actor to 
// get user related actions so we can send them back as a response to the
// api call and eventually frontend can show the related notifs