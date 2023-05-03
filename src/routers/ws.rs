




// https://crates.io/crates/routerify-websocket

// chatapp routes also see README.md strategies

// client --<ws | gql subs>-- hyper ws and gql server, redis + mongodb 

// 🥑 future io objects streaming coding using zmq, rpc capnp, ws and gql subs + redis and mongodb 
//    with tokio jobq channels (Arc, Mutex and RwLock), select event loop, event listener and event driven, 
//    spawn, tcp and udp quic streaming over hyper-tls, noise protocols and tokio-rustls for game coding logic 

// 🥑 there must be an event handler trait that in client can be implemented for any event struct 
//    to subs to events which have been fired through the http request to the server to be scheduled 
//    later for publishing and broadcasting them to ws channels 