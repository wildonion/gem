




/*
    --------------------------------------------------------------------------
   |                      REGISTER HANDLER FOR EVENT ROUTER
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
use crate::constants::*;
use crate::misc;
use hyper::{header, Body, Response, StatusCode};
use crate::controllers::event::{
                                add::{main as add_event, upload_img}, 
                                get::{all as all_events, 
                                      all_none_expired as get_all_none_expired_events,
                                      all_expired as get_all_expired_events,
                                      player_all_expired as get_all_player_expired_events, 
                                      player_all_for_dev as get_all_player_events_for_dev,
                                      player_all_none_expired as get_all_player_none_expired_events, 
                                      god_all_for_dev as get_all_god_events_for_dev,
                                      single as get_single_event, 
                                      group_all as get_all_group_events,
                                      explore_none_expired_events,
                                      god_single as get_god_single_event,
                                      god_all as get_all_god_events
                                    }, 
                                vote::main as cast_vote_event, 
                                expire::main as expire_event,
                                lock::main as lock_event,
                                cancel::main as cancel_event,
                                _404::main as not_found, 
                                phase::insert as insert_phase,
                                reserve::{process_payment_request, mock_reservation},
                                reveal::role,
                                simd::main as simd_ops
                            };





pub async fn register() -> Router<Body, hyper::Error>{  

    ///////////////////////////////////////////
    // let db_host = env::var("MONGODB_HOST").expect("⚠️ no db host variable set");
    // let db_port = env::var("MONGODB_PORT").expect("⚠️ no db port variable set");
    // let db_engine = env::var("DB_ENGINE").expect("⚠️ no db engine variable set");
    // let db_addr = format!("{}://{}:{}", db_engine, db_host, db_port);
    // let app_storage = Client::with_uri_str(&db_addr).await.unwrap();
    ///////////////////////////////////////////

    ////
    // NOTE - all the passed in handlers in every method of the Router::builder() is of type fn() or function pointer which must have a request object as their first param
    // NOTE - above operation is so expensive due to this fact : https://mongodb.github.io/mongo-rust-driver/manual/performance.html#lifetime
    // NOTE - only the request object must be passed through each handler
    // NOTE - shared state which will be available to every route handlers is the app_storage which must be Send + Syn + 'static to share between threads
    // NOTE - currently we're sharing the db instance between routers' threads from the main.rs instead of inside event router 
    ////`   

    Router::builder()
        // .data(app_storage) // sharing the initialized app_storage between routers' threads
        .get("/page", |req| async move{
            let res = Response::builder(); // creating a new response cause we didn't find any available route
            let response_body = misc::app::Response::<misc::app::Nill>{
                message: WELCOME,
                data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                status: 200,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket
                    .unwrap()
            )
        })
        .post("/add", add_event)
        .get("/explore/:query", explore_none_expired_events)
        .get("/get/all/in-going", get_all_none_expired_events)
        .get("/get/all/done", get_all_expired_events)
        .get("/get/all/god", get_all_god_events)
        .get("/get/all/god-with-id/:godId", get_all_god_events_for_dev)
        .post("/get/all/player/in-going",get_all_player_none_expired_events)
        .post("/get/all/player/done",get_all_player_expired_events)
        .get("/get/all/player/:playerId",get_all_player_events_for_dev)
        .post("/get/all/group", get_all_group_events)
        .get("/get/all", all_events)
        .post("/get/single/:eventId/god", get_god_single_event)
        .post("/get/single", get_single_event)
        .post("/cast-vote", cast_vote_event)
        .post("/set-expire", expire_event)
        .post("/set-lock", lock_event)
        .post("/cancel", cancel_event)
        .post("/update/phases/add", insert_phase)
        .post("/reserve/mock", mock_reservation)
        .post("/reveal/roles", role)
        .post("/update/:eventId/image", upload_img)
        .post("/simd", simd_ops)
        .any(not_found) // handling 404 request
        .build()
        .unwrap()



}