


/* >-------------------------------------
   |         SPACETIMEDB METHODS
   |
   | - thread safe app state syncing
   | - pubsub pattern syncing
   | - wasm and smart contract based design pattern
   | - in wasm context we can't async and std crates
   |
*/
use spacetimedb::{spacetimedb, ReducerContext, Identity, Timestamp};

#[spacetimedb(table)]
pub struct User {
    #[primarykey]
    identity: Identity,
    screen_cid: String,
    online: bool,
}

#[spacetimedb(table)]
pub struct Message {
    sender: Identity,
    sent: Timestamp,
    text: String,
}

#[spacetimedb(table)]
pub struct UserMessage {
    sender: Identity,
    message: Identity,
    clp_event_id: i32
}

#[spacetimedb(init)]
pub fn init() {
    // Called when the module is initially published
}

#[spacetimedb(connect)]
pub fn identity_connected(_ctx: ReducerContext) {
    // Called everytime a new client connects
}

#[spacetimedb(disconnect)]
pub fn identity_disconnected(_ctx: ReducerContext) {
    // Called everytime a client disconnects
}

#[spacetimedb(reducer)]
pub fn say_hello() {
    for user in User::iter() {
        log::info!("Hello, {}!", user.screen_cid);
    }
}