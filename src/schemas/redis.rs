



// https://crates.io/crates/redis
// https://crates.io/crates/redis-async

// redis client schemas to cache data on ram then store in mongodb on disk
// register a new pubsub topic in here so client to be able to see it in realtime
// ...

use std::net::SocketAddr;


#[derive(Clone)]
pub struct RedistClient{
    pub server_addr: SocketAddr,
}