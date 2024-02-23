

/*  > -----------------------------------------------------------------------
    | deploy actor is used to deploy the app by sending command to this actor
    | -----------------------------------------------------------------------
    | contains: message structures and their handlers
    |
*/

use crate::{*, constants::{WS_SUBSCRIPTION_INTERVAL, STORAGE_IO_ERROR_CODE}, events::publishers::action::{UserNotif, NotifExt, NotifData, SingleUserNotif}, models::users::User};
use actix::prelude::*;
use s3req::Storage;
use crate::events::subscribers::handlers::actors::notif::system::SystemActor;
use redis_async::resp::FromResp;
use actix::*;

pub struct DeployAgentActor{
    pub port: u16,
    pub path: std::path::PathBuf // use to store the path of deploy service script
}

impl Actor for DeployAgentActor{

    type Context = Context<Self>; // ctx contains the whole actor instance and its lifecycle execution

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("DeployAgentActor is started");
    }

}


impl DeployAgentActor{

    pub fn new(port: u16, path: std::path::PathBuf) -> Self{
        Self { port, path}
    }
}