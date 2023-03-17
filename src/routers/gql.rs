



// push notif handler and other realtime streaming like chatapp using gql subscriptions also see README.md strategies
// https://graphql-rust.github.io/juniper/master/advanced/subscriptions.html

// step 1) gql subscription routes over hyper to registered redis topics which is done by using hyper request  
// step 2) use schemas::gql to build the gql realtiming and subscription schemas
// step 3) setup ws routers in routers/ws.rs for chatapp routes with gql subscriptions