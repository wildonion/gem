


use crate::*;
use actix::prelude::*;
use s3req::Storage;


/* 
    actions data can be:
    invitation_requests from, friend requests from 
    like, comment, create nft and collection,
    list and buy nft, unclaimed gift cards from
*/
#[derive(Clone)]
pub struct UserActionActor{
    pub actioner_screen_cid: String,
    pub seen: bool,
    pub action_type: String,
    pub action_data: serde_json::Value,
    pub app_storage: Option<Arc<Storage>>
}


impl Actor for UserActionActor{
    
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context){
        
    }
    
}


// step1
// publish user_actions notif into redis pubsub channel when a user
// likes, commnts, creates nft and collection or even unclaimed gifts then 
// we'll start this actor where the server is being started to subscribe 
// to the incoming notif from redis pubsub channel like pg.rs

// step2
// there must be an http api call to be called from the frontend in an interval 
// once it gets hooked in the server we'll send a message to this actor to 
// get user related actions so we can send them back as a response to the
// caller and eventually frontend can show the related notifs, note that 
// the caller of the http api method must be either the action owner or a friend 
// of the user whose wants to see his actions 