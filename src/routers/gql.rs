




// gql subs is a listener for subscribing to emitted events and published topics using redis, zmq publisher socket or ws server setup in routers/ws.rs 
// push notif handler other realtime streaming like realtime game monitoring and chatapp using gql subscriptions also see README.md strategies
// https://graphql-rust.github.io/juniper/master/advanced/subscriptions.html

// step 0) setup redis routes in routers/redis.rs to register a redis pubsub topic so the client can see in browser in realtime using gql subscriptions
// step 1) gql subscription routes over hyper to registered redis topics which is done by using hyper request  
// step 2) use schemas::gql to build the gql realtiming and subscription schemas
// step 3) setup ws routers in routers/ws.rs for chatapp routes with gql subscriptions