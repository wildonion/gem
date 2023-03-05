



use crate::controllers::whitelist;
use crate::utils;
use crate::middlewares;
use crate::contexts as ctx;
use crate::schemas;
use crate::constants::*;
use chrono::Utc;
use futures::{executor::block_on, TryFutureExt, TryStreamExt}; //// futures is used for reading and writing streams asyncly from and into buffer using its traits and based on orphan rule TryStreamExt trait is required to use try_next() method on the future object which is solved by .await - try_next() is used on futures stream or chunks to get the next future IO stream and returns an Option in which the chunk might be either some value or none
use bytes::Buf; //// it'll be needed to call the reader() method on the whole_body buffer and is used for manipulating coming network bytes from the socket
use hyper::{header, StatusCode, Body, Response, Request};
use mongodb::bson::{self, oid::ObjectId, doc}; //// self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use mongodb::Client;
use log::info;
use mongodb::options::FindOneAndUpdateOptions;
use mongodb::options::ReturnDocument;
use std::env;









// -------------------------------- add user to whitelist controller
// ➝ Return : Hyper Response Body or Hyper Error
// ----------------------------------------------------------------------------------

pub async fn upsert(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{

    use routerify::prelude::*; //// to build the response object
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
                if access_level == DEV_ACCESS{ // NOTE - only dev can handle this route
                    let whole_body_bytes = hyper::body::to_bytes(req.into_body()).await?; //// to read the full body we have to use body::to_bytes or body::aggregate to collect all tcp IO stream of future chunk bytes or chunks which is of type utf8 bytes to concatenate the buffers from a body into a single Bytes asynchronously
                    match serde_json::from_reader(whole_body_bytes.reader()){ //// read the bytes of the filled buffer with hyper incoming body from the client by calling the reader() method from the Buf trait
                        Ok(value) => { //// making a serde value from the buffer which is a future IO stream coming from the client
                            let data: serde_json::Value = value;
                            let json = serde_json::to_string(&data).unwrap(); //// converting data into a json string
                            match serde_json::from_str::<schemas::whitelist::InsertWhitelistRequest>(&json){ //// the generic type of from_str() method is InsertWhitelistRequest struct - mapping (deserializing) the json string into the InsertWhitelistRequest struct
                                Ok(wl_info) => {


                                    let owner = wl_info.owner.clone(); //// cloning to prevent ownership moving
                                    let pda = wl_info.pda.clone(); //// cloning to prevent ownership moving - the pda calculated from the nft mint address and the nft owner after burning
                                    let name = wl_info.owner.clone(); //// cloning to prevent ownership moving

                                    ////////////////////////////////// DB Ops

                                    let update_option = FindOneAndUpdateOptions::builder().return_document(Some(ReturnDocument::After)).build();
                                    let whitelist = db.clone().database(&db_name).collection::<schemas::whitelist::WhitelistInfo>("whitelist");
                                    match whitelist.find_one(doc!{"name": name.clone(), "owners.owner": owner.clone()}, None).await.unwrap(){
                                        Some(mut wl_doc) => { //// we must declare the wl_doc as mutable since we want to mutate it later
                                            let is_owner_exists = wl_doc.owners.clone().into_iter().position(|od| od.owner == owner.clone());
                                            let owner_index = is_owner_exists.unwrap(); //// we're sure that we have an owner definitely since the find_one() query has executed correctly if we're here :)
                                            //// we found the passed in owner inside the whitelist
                                            //// then we have to update the list with the passed in pda
                                            if let Some(pdas) = wl_doc.add_pda(pda.clone(), owner_index).await{
                                                //// means we have an updated pda 
                                                //// then we need to update the collection
                                                wl_doc.owners[owner_index].pdas = pdas;
                                                let serialized_owners = bson::to_bson(&wl_doc.owners).unwrap(); //// we have to serialize the owners to BSON Document object in order to update the owners field inside the collection
                                                match whitelist.find_one_and_update(doc!{"name": name}, doc!{"$set": {"owners": serialized_owners, "updated_at": Some(Utc::now().timestamp())}}, Some(update_option)).await.unwrap(){
                                                    Some(wl_info) => { //// deserializing BSON into the EventInfo struct
                                                        let response_body = ctx::app::Response::<schemas::whitelist::WhitelistInfo>{ //// we have to specify a generic type for data field in Response struct which in our case is WhitelistInfo struct
                                                            data: Some(wl_info),
                                                            message: UPDATED, //// collection found in conse database
                                                            status: 200,
                                                        };
                                                        let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                                                        Ok(
                                                            res
                                                                .status(StatusCode::OK)
                                                                .header(header::CONTENT_TYPE, "application/json")
                                                                .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                                                .unwrap() 
                                                        )
                                                    }, 
                                                    None => { //// means we didn't find any document related to this title and we have to tell the user to create a new event
                                                        let response_body = ctx::app::Response::<ctx::app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                                                            data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                                                            message: NOT_FOUND_DOCUMENT,
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
                                                
                                                
                                            } else{
                                                let response_body = ctx::app::Response::<schemas::whitelist::WhitelistInfo>{ //// we have to specify a generic type for data field in Response struct which in our case is WhitelistInfo struct
                                                    data: Some(wl_doc),
                                                    message: FOUND_DOCUMENT,
                                                    status: 302,
                                                };
                                                let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                                                Ok(
                                                    res
                                                        .status(StatusCode::FOUND)
                                                        .header(header::CONTENT_TYPE, "application/json")
                                                        .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                                        .unwrap() 
                                                )
                                            }   
                                        }, 
                                        None => { //// no document found with this name thus we must insert a new whitelist into the databse which has no owners yet
                                            let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nano to sec 
                                            let whitelist = db.clone().database(&db_name).collection::<schemas::whitelist::AddWhitelistInfo>("whitelist"); //// using AddWhitelistInfo struct to insert a whitelist info into whitelist collection 
                                            let whitelist_doc = schemas::whitelist::AddWhitelistInfo{
                                                name,
                                                owners: vec![],
                                                created_at: Some(now),
                                                updated_at: Some(now),
                                            };
                                            match whitelist.insert_one(whitelist_doc, None).await{
                                                Ok(insert_result) => {
                                                    let response_body = ctx::app::Response::<ObjectId>{ //// we have to specify a generic type for data field in Response struct which in our case is ObjectId struct
                                                        data: Some(insert_result.inserted_id.as_object_id().unwrap()),
                                                        message: INSERTED,
                                                        status: 201,
                                                    };
                                                    let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                                                    Ok(
                                                        res
                                                            .status(StatusCode::CREATED)
                                                            .header(header::CONTENT_TYPE, "application/json")
                                                            .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                                            .unwrap() 
                                                    )
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
                                            .unwrap_or(hyper::Response::default()) 
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