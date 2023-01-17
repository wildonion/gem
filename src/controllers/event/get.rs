





use crate::middlewares;
use crate::contexts as ctx;
use crate::schemas;
use crate::constants::*;
use crate::utils;
use futures::{executor::block_on, TryFutureExt, TryStreamExt}; //// futures is used for reading and writing streams asyncly from and into buffer using its traits and based on orphan rule TryStreamExt trait is required to use try_next() method on the future object which is solved by .await - try_next() is used on futures stream or chunks to get the next future IO stream and returns an Option in which the chunk might be either some value or none
use bytes::Buf; //// it'll be needed to call the reader() method on the whole_body buffer and is used for manipulating coming network bytes from the socket
use hyper::{header, StatusCode, Body, Response, Request};
use log::info;
use mongodb::Client;
use mongodb::bson::Regex;
use mongodb::bson::{self, oid::ObjectId, doc}; //// self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use hyper::http::Uri;
use mongodb::options::FindOptions;
use std::env;














// -------------------------------- search event controller
// ➝ Return : Hyper Response Body or Hyper Error
// --------------------------------------------------------------------------------------

pub async fn explore_none_expired_events(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{


    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

                    
    let query = format!("{}", req.param("query").unwrap()); //// we must create the url param using format!() since this macro will borrow the req object and doesn't move it so we can access the req object later to handle other incoming data 
    let search_query = format!("[{}.*$]", query); //// https://www.computerhope.com/jargon/r/regex.htm
    let re = Regex{
        pattern: search_query,
        options: String::new(),
    };
    
    ////////////////////////////////// DB Ops
    
    let filter = doc! { "is_expired": false, "title": {"$regex": re} }; //// filtering all none expired events based on the searched query from the client
    let events = db.database(&db_name).collection::<schemas::event::ExploreEventInfo>("events"); //// selecting events collection to fetch and deserialize all event infos or documents from BSON into the ExploreExploreEventInfo struct
    let mut available_events = Vec::<schemas::event::ExploreEventInfo>::new();

    match events.find(filter, None).await{
        Ok(mut cursor) => {
            while let Some(event) = cursor.try_next().await.unwrap(){ //// calling try_next() method on cursor needs the cursor to be mutable - reading while awaiting on try_next() method doesn't return None
                available_events.push(event);
            }
            let res = Response::builder(); //// creating a new response cause we didn't find any available route
            let response_body = ctx::app::Response::<Vec<schemas::event::ExploreEventInfo>>{
                message: FETCHED,
                data: Some(available_events),
                status: 200,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::OK) //// not found route or method not allowed
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket
                    .unwrap()
            )
        },
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 500,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }
    
    //////////////////////////////////
                

                

}












// -------------------------------- get all events for a specific palyer controller
// ➝ Return : Hyper Response Body or Hyper Error
// --------------------------------------------------------------------------------------
pub async fn player_all_expired(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{
    
     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    match middlewares::auth::pass(req).await{
        Ok((token_data, req)) => { //// the decoded token and the request object will be returned from the function call since the Copy and Clone trait is not implemented for the hyper Request and Response object thus we can't have borrow the req object by passing it into the pass() function therefore it'll be moved and we have to return it from the pass() function   
                            
            let _id = token_data.claims._id;
            let username = token_data.claims.username;
            let access_level = token_data.claims.access_level;
    
            
            let db_to_pass = db.clone();
            if middlewares::auth::user::exists(Some(&db_to_pass), _id, username, access_level).await{ //// finding the user with these info extracted from jwt
                if access_level == DEV_ACCESS || access_level == DEFAULT_USER_ACCESS{ // NOTE - only dev and player can handle this route


                    ////////////////////////////////// DB Ops
                    
                    let filter = doc! { "is_expired": true, "players._id": _id.unwrap() }; //// filtering all expired events
                    let events = db.database(&db_name).collection::<schemas::event::EventInfo>("events"); //// selecting events collection to fetch and deserialize all event infos or documents from BSON into the ExploreEventInfo struct
                    let mut all_expired_events = schemas::event::AvailableEvents{
                        events: vec![],
                    };
                    match events.find(filter, None).await{
                        Ok(mut cursor) => {
                            while let Some(event) = cursor.try_next().await.unwrap(){ //// calling try_next() method on cursor needs the cursor to be mutable - reading while awaiting on try_next() method doesn't return None
                                all_expired_events.events.push(event);
                            }
                            let player_events = all_expired_events.events
                                                    .into_iter()
                                                    .map(|event| {
                                                        schemas::event::PlayerEventInfo{
                                                            _id: event._id,
                                                            title: event.title,
                                                            content: event.content,
                                                            deck_id: event.deck_id,
                                                            entry_price: event.entry_price,
                                                            group_info: event.group_info,
                                                            image_path: event.image_path,
                                                            creator_wallet_address: event.creator_wallet_address,
                                                            upvotes: event.upvotes,
                                                            downvotes: event.downvotes,
                                                            voters: event.voters,
                                                            phases: event.phases,
                                                            max_players: event.max_players,
                                                            is_expired: event.is_expired,
                                                            is_locked: event.is_locked,
                                                            started_at: event.started_at,
                                                            expire_at: event.expire_at,
                                                            created_at: event.created_at,
                                                            updated_at: event.updated_at,
                                                        }
                                                    })
                                                    .collect::<Vec<_>>();
                            let response_body = ctx::app::Response::<Vec<schemas::event::PlayerEventInfo>>{
                                message: FETCHED,
                                data: Some(player_events), //// returning all player events without exposing other player infos
                                status: 200,
                            };
                            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                            Ok(
                                res
                                    .status(StatusCode::OK) //// not found route or method not allowed
                                    .header(header::CONTENT_TYPE, "application/json")
                                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket
                                    .unwrap()
                            )
                        },
                        Err(e) => {
                            let response_body = ctx::app::Response::<ctx::app::Nill>{
                                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                                message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                                status: 500,
                            };
                            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                            Ok(
                                res
                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                    .header(header::CONTENT_TYPE, "application/json")
                                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                    .unwrap() 
                            )
                        },
                    }

                    //////////////////////////////////

                        
                } else{ //// access denied for this user with none admin and dev access level
                    let response_body = ctx::app::Response::<ctx::app::Nill>{
                        data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                        message: ACCESS_DENIED,
                        status: 403,
                    };
                    let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                    Ok(
                        res
                            .status(StatusCode::BAD_REQUEST)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                            .unwrap() 
                    )
                }
            } else{ //// user doesn't exist :(
                let response_body = ctx::app::Response::<ctx::app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                    data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                    message: DO_SIGNUP, //// document not found in database and the user must do a signup
                    status: 404,
                };
                let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                Ok(
                    res
                        .status(StatusCode::NOT_FOUND)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                        .unwrap() 
                )
            }
        },
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e, //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 500,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }

}












// -------------------------------- get all events for a specific palyer controller
// ➝ Return : Hyper Response Body or Hyper Error
// --------------------------------------------------------------------------------------
pub async fn player_all_none_expired(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{
    
     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    match middlewares::auth::pass(req).await{
        Ok((token_data, req)) => { //// the decoded token and the request object will be returned from the function call since the Copy and Clone trait is not implemented for the hyper Request and Response object thus we can't have borrow the req object by passing it into the pass() function therefore it'll be moved and we have to return it from the pass() function   
                            
            let _id = token_data.claims._id;
            let username = token_data.claims.username;
            let access_level = token_data.claims.access_level;
    
            
            let db_to_pass = db.clone();
            if middlewares::auth::user::exists(Some(&db_to_pass), _id, username, access_level).await{ //// finding the user with these info extracted from jwt
                if access_level == DEV_ACCESS || access_level == DEFAULT_USER_ACCESS{ // NOTE - only dev and player can handle this route

                    ////////////////////////////////// DB Ops
                    
                    let filter = doc! { "is_expired": false, "players._id": _id.unwrap() }; //// filtering all expired events
                    let events = db.database("ayoub").collection::<schemas::event::EventInfo>("events"); //// selecting events collection to fetch and deserialize all event infos or documents from BSON into the ExploreEventInfo struct
                    let mut all_none_expired_events = schemas::event::AvailableEvents{
                        events: vec![],
                    };
                    match events.find(filter, None).await{
                        Ok(mut cursor) => {
                            while let Some(event) = cursor.try_next().await.unwrap(){ //// calling try_next() method on cursor needs the cursor to be mutable - reading while awaiting on try_next() method doesn't return None
                                all_none_expired_events.events.push(event);
                            }
                            let player_events = all_none_expired_events.events
                                                    .into_iter()
                                                    .map(|event| {
                                                        schemas::event::PlayerEventInfo{
                                                            _id: event._id,
                                                            title: event.title,
                                                            content: event.content,
                                                            deck_id: event.deck_id,
                                                            entry_price: event.entry_price,
                                                            group_info: event.group_info,
                                                            image_path: event.image_path,
                                                            creator_wallet_address: event.creator_wallet_address,
                                                            upvotes: event.upvotes,
                                                            downvotes: event.downvotes,
                                                            voters: event.voters,
                                                            phases: event.phases,
                                                            max_players: event.max_players,
                                                            is_expired: event.is_expired,
                                                            is_locked: event.is_locked,
                                                            started_at: event.started_at,
                                                            expire_at: event.expire_at,
                                                            created_at: event.created_at,
                                                            updated_at: event.updated_at,
                                                        }
                                                    })
                                                    .collect::<Vec<_>>();
                            let response_body = ctx::app::Response::<Vec<schemas::event::PlayerEventInfo>>{
                                message: FETCHED,
                                data: Some(player_events), //// returning all player events without exposing other player infos
                                status: 200,
                            };
                            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                            Ok(
                                res
                                    .status(StatusCode::OK) //// not found route or method not allowed
                                    .header(header::CONTENT_TYPE, "application/json")
                                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket
                                    .unwrap()
                            )
                        },
                        Err(e) => {
                            let response_body = ctx::app::Response::<ctx::app::Nill>{
                                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                                message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                                status: 500,
                            };
                            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                            Ok(
                                res
                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                    .header(header::CONTENT_TYPE, "application/json")
                                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                    .unwrap() 
                            )
                        },
                    }

                    //////////////////////////////////
                            
                            
                } else{ //// access denied for this user with none admin and dev access level
                    let response_body = ctx::app::Response::<ctx::app::Nill>{
                        data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                        message: ACCESS_DENIED,
                        status: 403,
                    };
                    let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                    Ok(
                        res
                            .status(StatusCode::BAD_REQUEST)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                            .unwrap() 
                    )
                }
            } else{ //// user doesn't exist :(
                let response_body = ctx::app::Response::<ctx::app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                    data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                    message: DO_SIGNUP, //// document not found in database and the user must do a signup
                    status: 404,
                };
                let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                Ok(
                    res
                        .status(StatusCode::NOT_FOUND)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                        .unwrap() 
                )
            }
        },
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e, //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 500,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }

}











// -------------------------------- get all events controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------
pub async fn all_none_expired(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{
    
     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    ////////////////////////////////// DB Ops
                    
    let filter = doc! { "is_expired": false }; //// filtering all none expired events
    let events = db.database(&db_name).collection::<schemas::event::PlayerEventInfo>("events"); //// selecting events collection to fetch and deserialize all event infos or documents from BSON into the PlayerEventInfo struct
    let mut available_events = Vec::<schemas::event::PlayerEventInfo>::new();

    match events.find(filter, None).await{
        Ok(mut cursor) => {
            while let Some(event) = cursor.try_next().await.unwrap(){ //// calling try_next() method on cursor needs the cursor to be mutable - reading while awaiting on try_next() method doesn't return None
                available_events.push(event);
            }
            let res = Response::builder(); //// creating a new response cause we didn't find any available route
            let response_body = ctx::app::Response::<Vec<schemas::event::PlayerEventInfo>>{
                message: FETCHED,
                data: Some(available_events),
                status: 200,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::OK) //// not found route or method not allowed
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket
                    .unwrap()
            )
        },
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 500,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }

    //////////////////////////////////
        
}









// -------------------------------- get all events controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------
pub async fn all_expired(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{
    
     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    ////////////////////////////////// DB Ops
                    
    let filter = doc! { "is_expired": true }; //// filtering all none expired events
    let events = db.database(&db_name).collection::<schemas::event::PlayerEventInfo>("events"); //// selecting events collection to fetch and deserialize all event infos or documents from BSON into the PlayerEventInfo struct
    let mut available_events = Vec::<schemas::event::PlayerEventInfo>::new();

    match events.find(filter, None).await{
        Ok(mut cursor) => {
            while let Some(event) = cursor.try_next().await.unwrap(){ //// calling try_next() method on cursor needs the cursor to be mutable - reading while awaiting on try_next() method doesn't return None
                available_events.push(event);
            }
            let res = Response::builder(); //// creating a new response cause we didn't find any available route
            let response_body = ctx::app::Response::<Vec<schemas::event::PlayerEventInfo>>{
                message: FETCHED,
                data: Some(available_events),
                status: 200,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::OK) //// not found route or method not allowed
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket
                    .unwrap()
            )
        },
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 500,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }

    //////////////////////////////////
        
}










// -------------------------------- get all events controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------
pub async fn all(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{
    
     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    ////////////////////////////////// DB Ops
                    
    let events = db.database(&db_name).collection::<schemas::event::PlayerEventInfo>("events"); //// selecting events collection to fetch and deserialize all event infos or documents from BSON into the PlayerEventInfo struct
    let mut available_events = Vec::<schemas::event::PlayerEventInfo>::new();

    match events.find(None, None).await{
        Ok(mut cursor) => {
            while let Some(event) = cursor.try_next().await.unwrap(){ //// calling try_next() method on cursor needs the cursor to be mutable - reading while awaiting on try_next() method doesn't return None
                available_events.push(event);
            }
            let res = Response::builder(); //// creating a new response cause we didn't find any available route
            let response_body = ctx::app::Response::<Vec<schemas::event::PlayerEventInfo>>{
                message: FETCHED,
                data: Some(available_events),
                status: 200,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::OK) //// not found route or method not allowed
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket
                    .unwrap()
            )
        },
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 500,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }

    //////////////////////////////////
        
}










// -------------------------------- get a single event controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------
pub async fn single(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{
    
     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();
    

    let whole_body_bytes = hyper::body::to_bytes(req.into_body()).await?; //// to read the full body we have to use body::to_bytes or body::aggregate to collect all tcp IO stream of future chunk bytes or chunks which is of type utf8 bytes to concatenate the buffers from a body into a single Bytes asynchronously
    match serde_json::from_reader(whole_body_bytes.reader()){ //// read the bytes of the filled buffer with hyper incoming body from the client by calling the reader() method from the Buf trait
        Ok(value) => { //// making a serde value from the buffer which is a future IO stream coming from the client
            let data: serde_json::Value = value;
            let json = serde_json::to_string(&data).unwrap(); //// converting data into a json string
            match serde_json::from_str::<schemas::event::GetEventRequest>(&json){ //// the generic type of from_str() method is GetEventRequest struct - mapping (deserializing) the json string into the GetEventRequest struct
                Ok(event_info) => { //// we got the username and password inside the login route


                    ////////////////////////////////// DB Ops

                    let event_id = ObjectId::parse_str(event_info._id.as_str()).unwrap(); //// generating mongodb object id from the id string
                    let events = db.database(&db_name).collection::<schemas::event::PlayerEventInfo>("events"); //// selecting events collection to fetch and deserialize all event infos or documents from BSON into the ExplorePlayerEventInfo struct
                    match events.find_one(doc! { "_id": event_id }, None).await.unwrap(){
                        Some(event_doc) => {
                            let response_body = ctx::app::Response::<schemas::event::PlayerEventInfo>{
                                message: FETCHED,
                                data: Some(event_doc),
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
                        },
                        None => { //// means we didn't find any document related to this user_id and we have to tell the user do a signup
                            let response_body = ctx::app::Response::<ctx::app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                                message: NOT_FOUND_DOCUMENT, //// document not found in database and the user must do a signup
                                status: 404,
                            };
                            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                            Ok(
                                res
                                    .status(StatusCode::NOT_FOUND)
                                    .header(header::CONTENT_TYPE, "application/json")
                                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                    .unwrap() 
                            )
                        },
                    }

                    //////////////////////////////////

                },
                Err(e) => {
                    let response_body = ctx::app::Response::<ctx::app::Nill>{
                        data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                        message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                        status: 406,
                    };
                    let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                    Ok(
                        res
                            .status(StatusCode::NOT_ACCEPTABLE)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                            .unwrap() 
                    )
                },
            }
        },
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 400,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::BAD_REQUEST)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }        
}

















// -------------------------------- get god single event controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------
pub async fn god_single(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{
    
     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();
    
    match middlewares::auth::pass(req).await{
        Ok((token_data, req)) => { //// the decoded token and the request object will be returned from the function call since the Copy and Clone trait is not implemented for the hyper Request and Response object thus we can't have borrow the req object by passing it into the pass() function therefore it'll be moved and we have to return it from the pass() function   
                            
            let _id = token_data.claims._id;
            let username = token_data.claims.username;
            let access_level = token_data.claims.access_level;
    
            
            let db_to_pass = db.clone();
            if middlewares::auth::user::exists(Some(&db_to_pass), _id, username, access_level).await{ //// finding the user with these info extracted from jwt
                if access_level == ADMIN_ACCESS || access_level == DEV_ACCESS{ // NOTE - only dev and admin (God) can handle this route
                
                    let event_id = format!("{}", req.param("eventId").unwrap()); //// we must create the url param using format!() since this macro will borrow the req object and doesn't move it so we can access the req object later to handle other incoming data 

                    let event_id = ObjectId::parse_str(event_id.as_str()).unwrap(); //// generating mongodb object id from the id string
                    let events = db.database(&db_name).collection::<schemas::event::EventInfo>("events"); //// selecting events collection to fetch and deserialize all event infos or documents from BSON into the EventInfo struct
                    if utils::event_belongs_to_god(_id.unwrap(), event_id, db_to_pass.clone()).await || access_level == DEV_ACCESS{ //// checking that the passed in event id is belongs to the passed in god id or not 

                        ////////////////////////////////// DB Ops
                        
                        match events.find_one(doc! { "_id": event_id }, None).await.unwrap(){
                            Some(event_doc) => {
                                let response_body = ctx::app::Response::<schemas::event::EventInfo>{
                                    message: FETCHED,
                                    data: Some(event_doc),
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
                            },
                            None => { //// means we didn't find any document related to this user_id and we have to tell the user do a signup
                                let response_body = ctx::app::Response::<ctx::app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                                    data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                                    message: NOT_FOUND_DOCUMENT, //// document not found in database and the user must do a signup
                                    status: 404,
                                };
                                let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                                Ok(
                                    res
                                        .status(StatusCode::NOT_FOUND)
                                        .header(header::CONTENT_TYPE, "application/json")
                                        .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                        .unwrap() 
                                )
                            },
                        }
    
                        //////////////////////////////////

                    } else{
                        let response_body = ctx::app::Response::<ctx::app::Nill>{
                            data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                            message: ACCESS_DENIED,
                            status: 403,
                        };
                        let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                        Ok(
                            res
                                .status(StatusCode::BAD_REQUEST)
                                .header(header::CONTENT_TYPE, "application/json")
                                .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                .unwrap() 
                        )
                    }
                } else{ //// access denied for this user with none admin and dev access level
                    let response_body = ctx::app::Response::<ctx::app::Nill>{
                        data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                        message: ACCESS_DENIED,
                        status: 403,
                    };
                    let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                    Ok(
                        res
                            .status(StatusCode::BAD_REQUEST)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                            .unwrap() 
                    )
                }
            } else{ //// user doesn't exist :(
                let response_body = ctx::app::Response::<ctx::app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                    data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                    message: DO_SIGNUP, //// document not found in database and the user must do a signup
                    status: 404,
                };
                let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                Ok(
                    res
                        .status(StatusCode::NOT_FOUND)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                        .unwrap() 
                )
            }
        },
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e, //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 500,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }
}

















// -------------------------------- get god single event controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------
pub async fn god_all(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{
    
     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();
    
    match middlewares::auth::pass(req).await{
        Ok((token_data, req)) => { //// the decoded token and the request object will be returned from the function call since the Copy and Clone trait is not implemented for the hyper Request and Response object thus we can't have borrow the req object by passing it into the pass() function therefore it'll be moved and we have to return it from the pass() function   
                            
            let _id = token_data.claims._id;
            let username = token_data.claims.username;
            let access_level = token_data.claims.access_level;
    
            
            let db_to_pass = db.clone();
            if middlewares::auth::user::exists(Some(&db_to_pass), _id, username, access_level).await{ //// finding the user with these info extracted from jwt
                if access_level == ADMIN_ACCESS || access_level == DEV_ACCESS{ // NOTE - only dev and admin (God) can handle this route
                

                    ////////////////////////////////// DB Ops
                    
                    
                    let events = db.database(&db_name).collection::<schemas::event::EventInfo>("events"); //// selecting events collection to fetch and deserialize all event infos or documents from BSON into the EventInfo struct
                    let mut all_god_events = vec![];
                    let events = db.database(&db_name).collection::<schemas::event::EventInfo>("events"); //// selecting events collection to fetch and deserialize all event infos or documents from BSON into the EventInfo struct
                    let mut events_cursor = events.find(doc!{"group_info.god_id": _id.unwrap().to_string()}, None).await.unwrap(); //// we must define the cursor as mutable since fetching all events is a mutable operation
                    while let Some(event_info) = events_cursor.try_next().await.unwrap(){
                        all_god_events.push(event_info)
                    }
                    if all_god_events.is_empty(){
                        let response_body = ctx::app::Response::<ctx::app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                            data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                            message: NOT_FOUND_DOCUMENT, //// document not found in database and the user must do a signup
                            status: 404,
                        };
                        let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                        Ok(
                            res
                                .status(StatusCode::NOT_FOUND)
                                .header(header::CONTENT_TYPE, "application/json")
                                .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                .unwrap() 
                        )
                    } else{
                        let response_body = ctx::app::Response::<Vec<schemas::event::EventInfo>>{
                            message: FETCHED,
                            data: Some(all_god_events),
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
                    }

                    //////////////////////////////////

                        
                } else{ //// access denied for this user with none admin and dev access level
                    let response_body = ctx::app::Response::<ctx::app::Nill>{
                        data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                        message: ACCESS_DENIED,
                        status: 403,
                    };
                    let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                    Ok(
                        res
                            .status(StatusCode::BAD_REQUEST)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                            .unwrap() 
                    )
                }
            } else{ //// user doesn't exist :(
                let response_body = ctx::app::Response::<ctx::app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                    data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                    message: DO_SIGNUP, //// document not found in database and the user must do a signup
                    status: 404,
                };
                let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                Ok(
                    res
                        .status(StatusCode::NOT_FOUND)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                        .unwrap() 
                )
            }
        },
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e, //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 500,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }
}




















// -------------------------------- get all events for a specific group controller
// ➝ Return : Hyper Response Body or Hyper Error
// ----------------------------------------------------------------------------------------------
pub async fn group_all(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{
    
     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();
    

    let whole_body_bytes = hyper::body::to_bytes(req.into_body()).await?; //// to read the full body we have to use body::to_bytes or body::aggregate to collect all tcp IO stream of future chunk bytes or chunks which is of type utf8 bytes to concatenate the buffers from a body into a single Bytes asynchronously
    match serde_json::from_reader(whole_body_bytes.reader()){ //// read the bytes of the filled buffer with hyper incoming body from the client by calling the reader() method from the Buf trait
        Ok(value) => { //// making a serde value from the buffer which is a future IO stream coming from the client
            let data: serde_json::Value = value;
            let json = serde_json::to_string(&data).unwrap(); //// converting data into a json string
            match serde_json::from_str::<schemas::game::GetGroupRequest>(&json){ //// the generic type of from_str() method is GetEventRequest struct - mapping (deserializing) the json string into the GetEventRequest struct
                Ok(group_info) => { //// we got the username and password inside the login route


                    ////////////////////////////////// DB Ops

                    let mut all_group_events = vec![];
                    let events = db.database(&db_name).collection::<schemas::event::PlayerEventInfo>("events"); //// selecting events collection to fetch and deserialize all event infos or documents from BSON into the ExplorePlayerEventInfo struct
                    let mut events_cursor = events.find(doc!{"group_info._id": group_info._id}, None).await.unwrap(); //// we must define the cursor as mutable since fetching all events is a mutable operation
                    while let Some(event_info) = events_cursor.try_next().await.unwrap(){
                        all_group_events.push(event_info)
                    }
                    if all_group_events.is_empty(){
                        let response_body = ctx::app::Response::<ctx::app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                            data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                            message: NOT_FOUND_DOCUMENT, //// document not found in database and the user must do a signup
                            status: 404,
                        };
                        let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                        Ok(
                            res
                                .status(StatusCode::NOT_FOUND)
                                .header(header::CONTENT_TYPE, "application/json")
                                .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                .unwrap() 
                        )
                    } else{
                        let response_body = ctx::app::Response::<Vec<schemas::event::PlayerEventInfo>>{
                            message: FETCHED,
                            data: Some(all_group_events),
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
                    }
                

                    //////////////////////////////////

                },
                Err(e) => {
                    let response_body = ctx::app::Response::<ctx::app::Nill>{
                        data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                        message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                        status: 406,
                    };
                    let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                    Ok(
                        res
                            .status(StatusCode::NOT_ACCEPTABLE)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                            .unwrap() 
                    )
                },
            }
        },
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 400,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::BAD_REQUEST)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }        
}








