


// https://crates.io/crates/routerify-websocket

// realtiming chatapp and push notification

// client --<gql subs + tokio tls/noise>-- server 
// client --<hyper>-- server + register redis pubsub using its client 
// client -----gql/ws/webrtc.rs [p2p]----- server actors 
//                                             tokio tls, ws server, rcp capnp pubsub server, gql pubsub server, redis pubsub client, 
//                                             zmq pubsub server, mongodb (libp2p stacks(muxer, noise, quic, tcp, gossipsub, kademlia, ws, webrtc))

// while let Ok((stream, addr)) = listener.accept().await{
//     tokio::spawn(async move{
//         streaming of IO future objects through redis, hyper, ricker, 
//         tokio tcp and udp and quic and muxer, libp2p stacks, zmq, rpc, ws and gql 
//         for realtiming pubsub streaming like push notif and chatapp
//     });
// }

// ➙ tokio tcp, udp streaming future IO objects and select eventloop, spawn, scheduler, channels (Arc<Mutex<Data>> + Send + Sync + 'static) to build multi worker and thread based proxy and server like nginx and hyper 
// ➙ public key digital signature ed25519 verification for updating app and verification 
// ➙ tokio tcp and udp streams (the one configured inside the coiniXerr) with hyper ws and redis based on mongodb for chatapp with docker setup