




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
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    account::ReadableAccount, program_pack::Pack,
    pubkey::Pubkey,
    commitment_config::CommitmentConfig,
    system_transaction,
    signature::Keypair,
    signer::Signer,
    signature::Signature,
};









// -------------------------------- add user to whitelist controller
// ➝ Return : Hyper Response Body or Hyper Error
// ----------------------------------------------------------------------------------

pub async fn upsert(req: Request<Body>) -> ConseResult<hyper::Response<Body>, hyper::Error>{

    use routerify::prelude::*; //// to build the response object
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    // let sol_net = env::var("SOLANA_DEVNET").expect("⚠️ no solana net vairiable set");
    let sol_net = env::var("SOLANA_MAINNET").expect("⚠️ no solana net vairiable set");
    let db = &req.data::<Client>().unwrap().to_owned();
    let rpc_client = RpcClient::new_with_commitment::<String>(sol_net.into(), CommitmentConfig::confirmed()); //// the generic that must be passed to the new_with_commitment method must be String since the address is of type String 



    ///// ==============================================================================
    ////                              LOAD NFT MINT ADDRESSES
    ///// ==============================================================================
    //// from_reader() accepts a buffer path or tcp stream
    //// then it returns a serde Value which can be converted
    //// into the json string that can be mapped into a structure.
    let file = std::fs::File::open("nfts.json").unwrap(); //// the file must be inside where we run the `cargo run` command or the root dir
    let nfts_value: serde_json::Value = serde_json::from_reader(file).unwrap();
    let nfts_json_string = serde_json::to_string(&nfts_value).unwrap(); //// reader in serde_json::from_reader can be a tokio tcp stream, a file or a buffer that contains the u8 bytes
    let nft = serde_json::from_str::<schemas::whitelist::Nft>(&nfts_json_string).unwrap(); 
    let snapshot_nfts = nft.mint_addrs;




    //// ============ NOTE ============
    //// frontend must have the following
    //// hash in a config file since this
    //// hash will be decoded that must be
    //// the one inside the .env file.
    // let api_key_hardcoded = "$argon2i$v=19$m=4096,t=3,p=1$Y29uc2UtaW5zZWN1cmUtOTgwbzM3XiEzZnUpa3pibzV6KGtybTJzXl5ibzFuKi1udnkoNis4MiklNjB5cGRtLXU$xyuPmb2pZQ4P2atgLPwc3ocE5VrEamWBkOxE9SBrdrE";
    
    ///// ==============================================================================
    ////                                API KEY VALIDATION
    ///// ==============================================================================
    let Some(header_value_api_key) = req.headers().get("API_KEY") else{
        let response_body = ctx::app::Response::<ctx::app::Nill>{
            data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
            message: HTTP_HEADER_ERR,
            status: 500,
        };
        let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
        return Ok(
            res
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                .unwrap() 
        );
    };

    let Ok(api_key) = header_value_api_key.to_str() else{ //// hased api key from the client
        let response_body = ctx::app::Response::<ctx::app::Nill>{
            data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
            message: NO_API_KEY,
            status: 403,
        };
        let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
        return Ok(
            res
                .status(StatusCode::FORBIDDEN)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                .unwrap() 
        );
    }; 


    let whitelist_secret_key = env::var("WHITELIST_SECRET_KEY").expect("⚠️ no whitelist secret key variable set");
    let whitelist_secret_key_bytes = whitelist_secret_key.as_bytes();
    let dev = match argon2::verify_encoded(api_key, whitelist_secret_key_bytes){
        Ok(is_dev) => {
            is_dev
        }, 
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e.to_string(), //// passing a reference to the underlying string of the Error type 
                status: 500,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            return Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            );
        }
    };

    if !dev{
        let response_body = ctx::app::Response::<ctx::app::Nill>{
            data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
            message: ACCESS_DENIED,
            status: 403,
        };
        let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
        return Ok(
            res
                .status(StatusCode::FORBIDDEN)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                .unwrap() 
        );
    }
    ///// ==============================================================================


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
                                    let mint_addrs = wl_info.mint_addrs.clone(); //// cloning to prevent ownership moving - the pda calculated from the nft burn tx hash address and the nft owner after burning
                                    let name = wl_info.name.clone(); //// cloning to prevent ownership moving

                                    // ================= NFT VERIFICATION ===================
                                    //// one of the NFT mint addr must be inside the snapshot
                                    //// otherwise the NFT is not verified and is not owned
                                    //// by the owner
                                    // ======================================================
                                    let vefified_nft_mint_addresses = mint_addrs.iter().all(|mint_addr| snapshot_nfts.contains(mint_addr));
                                    if !vefified_nft_mint_addresses{
                                        let response_body = ctx::app::Response::<ctx::app::Nill>{
                                            data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                                            message: NOT_VERIFIED_NFT, //// one of the NFT is not verified since it's not inside the snapshot
                                            status: 406,
                                        };
                                        let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                                        return Ok(
                                            res
                                                .status(StatusCode::NOT_ACCEPTABLE)
                                                .header(header::CONTENT_TYPE, "application/json")
                                                .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                                .unwrap() 
                                        );   
                                    }

                                    // ================ OWNER VERIFICATION ================
                                    //// verify that the passed in mint addresses
                                    //// belongs to the passed in owner by sending the 
                                    //// request to the solana json rpc endpoint
                                    // ====================================================
                                    if !schemas::whitelist::verify_owner(owner.clone(), &mint_addrs, &rpc_client).await{
                                        let response_body = ctx::app::Response::<ctx::app::Nill>{
                                            data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                                            message: NOT_VERIFIED_OWNER, //// mint addresses doesn't belong to the owner
                                            status: 406,
                                        };
                                        let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                                        return Ok(
                                            res
                                                .status(StatusCode::NOT_ACCEPTABLE)
                                                .header(header::CONTENT_TYPE, "application/json")
                                                .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                                .unwrap() 
                                        );   
                                    }



                                    ////////////////////////////////// DB Ops

                                    let update_option = FindOneAndUpdateOptions::builder().return_document(Some(ReturnDocument::After)).build();
                                    let whitelist = db.clone().database(&db_name).collection::<schemas::whitelist::WhitelistInfo>("whitelist");
                                    ///// ==============================================================================
                                    ////                         UNIQUE OWNER MINT ADDRESS VALIDATION
                                    ///// ==============================================================================
                                    match whitelist.find_one(doc!{"owners.mint_addrs": mint_addrs.clone()}, None).await.unwrap(){
                                        Some(wl_doc) => {
                                            let response_body = ctx::app::Response::<schemas::whitelist::WhitelistInfo>{ //// we have to specify a generic type for data field in Response struct which in our case is WhitelistInfo struct
                                                data: Some(wl_doc),
                                                message: ALREADY_BURNED, //// already burned nft since the passed in mint addresses are belong to an owner that has already burned them 
                                                status: 302,
                                            };
                                            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                                            return Ok(
                                                res
                                                    .status(StatusCode::FOUND)
                                                    .header(header::CONTENT_TYPE, "application/json")
                                                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                                    .unwrap() 
                                            );
                                        }
                                        None => {}
                                    }
                                    ///// ==============================================================================
                                    match whitelist.find_one(doc!{"name": name.clone(), "owners.owner": owner.clone()}, None).await.unwrap(){
                                        Some(mut wl_doc) => { //// we must declare the wl_doc as mutable since we want to mutate it later
                                            let is_owner_exists = wl_doc.owners.clone().into_iter().position(|od| od.owner == owner.clone());
                                            let owner_index = is_owner_exists.unwrap(); //// we're sure that we have an owner definitely since the find_one() query has executed correctly if we're here :)
                                            //// we found the passed in owner inside the whitelist
                                            //// then we have to update the list with the passed in pda
                                            if let Some(mint_addrs) = wl_doc.add_mint_addrs(mint_addrs.clone(), owner_index).await{
                                                //// means we have an updated pda 
                                                //// then we need to update the collection
                                                wl_doc.owners[owner_index].mint_addrs = mint_addrs;
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
                                                    message: ALREADY_BURNED, //// already burned nft since one of the addresses inside the mint_addrs is already burned by this owner 
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
                                                owners: vec![schemas::whitelist::OwnerData{
                                                    mint_addrs, //// vector of the passed in mint_addrs from the client
                                                    owner, //// owner of the burned nft
                                                    requested_at: Some(now)
                                                }],
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
                            .status(StatusCode::FORBIDDEN)
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

}