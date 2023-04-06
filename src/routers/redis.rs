


/*
    --------------------------------------------------------------------------
   |                      REGISTER HANDLER FOR REDIS ROUTER
   |--------------------------------------------------------------------------
   |
   |    job    : the following registers a router requested by the client
   |    return : a Router of type either hyper response body or error response
   |
   |
   |
   | we don't need to have one response object for each router and we can build
   | a new one inside the body of each router since rust doesn't support garbage
   | collection rule and each response object will be dropped once each router 
   | router body scope gets ended.
   | 

*/



use std::env;
use mongodb::Client;
use routerify::{Router, Middleware};
use crate::middlewares;
use crate::contexts as ctx;
use crate::constants::*;
use hyper::{header, Body, Response, StatusCode};
use crate::controllers::redis::{_404::main as not_found, 
                                register::register_notif as register_new_topic
                            };





pub async fn register() -> Router<Body, hyper::Error>{  

    /////////////////////////////////////////////////////////////////////////////////////
    // let db_host = env::var("MONGODB_HOST").expect("⚠️ no db host variable set");
    // let db_port = env::var("MONGODB_PORT").expect("⚠️ no db port variable set");
    // let db_engine = env::var("DB_ENGINE").expect("⚠️ no db engine variable set");
    // let db_addr = format!("{}://{}:{}", db_engine, db_host, db_port);
    // let app_storage = Client::with_uri_str(&db_addr).await.unwrap();
    /////////////////////////////////////////////////////////////////////////////////////

    ////////
    // NOTE - all the passed in handlers in every method of the Router::builder() is of type fn() or function pointer which must have a request object as their first param
    // NOTE - above operation is so expensive due to this fact : https://mongodb.github.io/mongo-rust-driver/manual/performance.html#lifetime
    // NOTE - only the request object must be passed through each handler
    // NOTE - shared state which will be available to every route handlers is the app_storage which must be Send + Syn + 'static to share between threads
    // NOTE - currently we're sharing the db instance between routers' threads from the main.rs instead of inside event router
    ////////`   

    Router::builder()
        // .data(app_storage) //// sharing the initialized app_storage between routers' threads
        .get("/page", |req| async move{
            let res = Response::builder(); //// creating a new response cause we didn't find any available route
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                message: WELCOME,
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                status: 200,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket
                    .unwrap()
            )
        })
        .post("/register-topic", register_new_topic)
        .any(not_found) //// handling 404 request
        .build()
        .unwrap()



}