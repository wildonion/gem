

use actix::{Message, Handler};
use crate::models::users::UserData;
use crate::*;
use actix::MessageResponse;



#[derive(Message)]
#[rtype(result = "()")]
pub struct NotifySystemActorWithRedisSubscription{
    pub new_user: String 
}

#[derive(Clone, Message)]
#[rtype(result = "String")]
pub struct GetNewUser;


#[derive(Clone, Message)]
#[rtype(result = "SystemUsers")]
pub struct GetSystemUsersMap;

#[derive(MessageResponse, Debug)]
pub struct SystemUsers(pub HashMap<i32, UserData>);


#[derive(Clone)]
pub struct SystemActor{
    /* 
        we're using an in memory map based db to store updated user in runtime and realtime
        hence it's fast enough to do read and write operations
    */
    pub updated_users: HashMap<i32, UserData>,
}

impl Actor for SystemActor{
    type Context = actix::Context<Self>;
}

impl Handler<NotifySystemActorWithRedisSubscription> for SystemActor{

    type Result = ();

    fn handle(&mut self, msg: NotifySystemActorWithRedisSubscription, ctx: &mut Self::Context) -> Self::Result {
        
        let NotifySystemActorWithRedisSubscription{new_user} = msg;
        let decoded_user = serde_json::from_str::<UserData>(&new_user).unwrap();
        info!("payload in system actor -> {:?}", decoded_user);
        
        self.updated_users.insert(decoded_user.id, decoded_user);
        info!("updated users map in system actor -> {:?}", self.updated_users);

    }
}

/* 
    other parts of the app can communicate with this actor to get the 
    updated records also behind message handlers of each actor are mpsc 
    jobq channel which allows other parts of the app to send data using 
    the sender and receive the response using receiver


    other parts of the app can do this:

    let resp = system_actor
        .send(GetSystemUsersMap)
        .await;

*/
impl Handler<GetSystemUsersMap> for SystemActor{

    type Result = SystemUsers;

    fn handle(&mut self, msg: GetSystemUsersMap, ctx: &mut Self::Context) -> Self::Result {

        let new_user = self.updated_users.clone();
        SystemUsers(new_user)

    }
}
