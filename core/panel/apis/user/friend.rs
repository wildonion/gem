


pub use super::*;

#[get("/fan/get/unaccepted/friend-requests/")]
#[passport(user)]
pub(self) async fn get_user_unaccepted_friend_requests(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserFan::get_user_unaccepted_friend_requests(
                        &user.screen_cid.unwrap(),
                        limit, connection).await{
                        
                        Ok(unaccepted_requests) => {

                            resp!{
                                Vec<Option<FriendData>>, //// the data type
                                unaccepted_requests, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    
                    }
                    
                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[get("/fan/get/relations/for/{who}/")]
#[passport(user)]
pub(self) async fn get_all_user_relations(
    req: HttpRequest,
    who_screen_cid: web::Path<String>,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserFan::get_user_relations(
                        &who_screen_cid.to_owned(), 
                        limit, connection).await{
                        
                        Ok(user_relations) => {

                            resp!{
                                UserRelations, //// the data type
                                user_relations, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    
                    }
                    
                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[post("/fan/accept/friend-request")]
#[passport(user)]
pub(self) async fn accept_friend_request(
    req: HttpRequest,
    accept_friend_request: web::Json<AcceptFriendRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let accept_friend_request = accept_friend_request.to_owned();
                    
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*   -=-=-=-=-=- USER MUST BE KYCED -=-=-=-=-=-  */
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*
                        followings are the param 
                        must be passed to do the 
                        kyc process on request data
                        @params:
                            - _id              : user id
                            - from_cid         : user crypto id
                            - tx_signature     : tx signature signed
                            - hash_data        : sha256 hash of data generated in client app
                            - deposited_amount : the amount of token must be deposited for this call
                    */
                    let is_request_verified = helpers::kyced::verify_request(
                        _id, 
                        &accept_friend_request.owner_cid, 
                        &accept_friend_request.tx_signature, 
                        &accept_friend_request.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserFan::accept_friend_request(accept_friend_request, redis_actix_actor, connection).await{
                        Ok(user_fan_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fan_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    }
                    

                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[post("/fan/send/friend-request")]
#[passport(user)]
pub(self) async fn send_friend_request_to(
    req: HttpRequest,
    send_friend_request_to: web::Json<SendFriendRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let send_friend_request_to = send_friend_request_to.to_owned();
                    
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*   -=-=-=-=-=- USER MUST BE KYCED -=-=-=-=-=-  */
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*
                        followings are the param 
                        must be passed to do the 
                        kyc process on request data
                        @params:
                            - _id              : user id
                            - from_cid         : user crypto id
                            - tx_signature     : tx signature signed
                            - hash_data        : sha256 hash of data generated in client app
                            - deposited_amount : the amount of token must be deposited for this call
                    */
                    let is_request_verified = helpers::kyced::verify_request(
                        _id, 
                        &send_friend_request_to.owner_cid, 
                        &send_friend_request_to.tx_signature, 
                        &send_friend_request_to.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserFan::send_friend_request_to(send_friend_request_to, redis_actix_actor, connection).await{
                        Ok(user_fan_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fan_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    }
                    

                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[post("/fan/remove/follower")]
#[passport(user)]
pub(self) async fn remove_user_from_follower(
    req: HttpRequest,
    remove_follower_request: web::Json<RemoveFollower>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let remove_follower_request = remove_follower_request.to_owned();
                    
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*   -=-=-=-=-=- USER MUST BE KYCED -=-=-=-=-=-  */
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*
                        followings are the param 
                        must be passed to do the 
                        kyc process on request data
                        @params:
                            - _id              : user id
                            - from_cid         : user crypto id
                            - tx_signature     : tx signature signed
                            - hash_data        : sha256 hash of data generated in client app
                            - deposited_amount : the amount of token must be deposited for this call
                    */
                    let is_request_verified = helpers::kyced::verify_request(
                        _id, 
                        &remove_follower_request.owner_cid, 
                        &remove_follower_request.tx_signature, 
                        &remove_follower_request.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserFan::remove_follower(remove_follower_request, connection).await{
                        Ok(user_fan_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fan_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    }
                    

                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[post("/fan/remove/friend")]
#[passport(user)]
pub(self) async fn remove_user_from_friend(
    req: HttpRequest,
    remove_friend_request: web::Json<RemoveFriend>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let remove_friend_request = remove_friend_request.to_owned();
                    
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*   -=-=-=-=-=- USER MUST BE KYCED -=-=-=-=-=-  */
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*
                        followings are the param 
                        must be passed to do the 
                        kyc process on request data
                        @params:
                            - _id              : user id
                            - from_cid         : user crypto id
                            - tx_signature     : tx signature signed
                            - hash_data        : sha256 hash of data generated in client app
                            - deposited_amount : the amount of token must be deposited for this call
                    */
                    let is_request_verified = helpers::kyced::verify_request(
                        _id, 
                        &remove_friend_request.owner_cid, 
                        &remove_friend_request.tx_signature, 
                        &remove_friend_request.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserFan::remove_friend(remove_friend_request, redis_client.clone(), redis_actix_actor.clone(), connection).await{
                        Ok(user_fan_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fan_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    }
                    

                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[post("/fan/remove/following")]
#[passport(user)]
pub(self) async fn remove_user_from_following(
    req: HttpRequest,
    remove_following_request: web::Json<RemoveFollowing>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let remove_following_request = remove_following_request.to_owned();
                    
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*   -=-=-=-=-=- USER MUST BE KYCED -=-=-=-=-=-  */
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*
                        followings are the param 
                        must be passed to do the 
                        kyc process on request data
                        @params:
                            - _id              : user id
                            - from_cid         : user crypto id
                            - tx_signature     : tx signature signed
                            - hash_data        : sha256 hash of data generated in client app
                            - deposited_amount : the amount of token must be deposited for this call
                    */
                    let is_request_verified = helpers::kyced::verify_request(
                        _id, 
                        &remove_following_request.owner_cid, 
                        &remove_following_request.tx_signature, 
                        &remove_following_request.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserFan::remove_following(remove_following_request, redis_client.clone(), redis_actix_actor.clone(), connection).await{
                        Ok(user_fan_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fan_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    }
                    

                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[get("/fan/get/all/friends/")]
#[passport(user)]
pub(self) async fn get_all_my_friends(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserFan::get_all_my_friends(
                        &user.screen_cid.unwrap(),
                        limit, connection).await{
                        Ok(user_fans_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fans_data, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    }
                    

                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[get("/fan/get/all/followings/")]
#[passport(user)]
pub(self) async fn get_all_my_followings(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserFan::get_all_my_followings(
                        &user.screen_cid.unwrap(),
                        limit, connection).await{
                        Ok(followings) => {

                            resp!{
                                Vec<UserFanDataWithWalletInfo>, //// the data type
                                followings, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    }
                    

                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[get("/fan/get/all/followers/")]
#[passport(user)]
pub(self) async fn get_all_my_followers(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserFan::get_all_my_followers(
                        &user.screen_cid.unwrap(),
                        limit, connection).await{
                        Ok(user_fans_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fans_data, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    }
                    

                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[get("/fan/get/suggestions/for/")]
#[passport(user)]
pub(self) async fn get_friend_suggestions_for_owner(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match User::suggest_user_to_owner(
                        limit, 
                        &user.screen_cid.unwrap(),
                        connection).await{
                        Ok(suggestions) => {

                            resp!{
                                Vec<UserWalletInfoResponseForUserSuggestions>, //// the data type
                                suggestions, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    }
                    

                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[get("/fan/friends/search/for/{who_screen_cid}/")]
#[passport(user)]
pub(self) async fn search_in_friends(
    req: HttpRequest,
    query: web::Query<UnlimitedSearch>,
    who_screen_cid: web::Path<String>,
    app_state: web::Data<AppState>,
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    
    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    let search_q = &query.q;
                    let get_friends = UserFan::get_all_my_friends_without_limit(&who_screen_cid.to_owned(), connection).await;
                    let Ok(fan_data) = get_friends else{
                        let err_resp = get_friends.unwrap_err();
                        return err_resp;
                    };

                    let friends_data = fan_data.friends;
                    let mut decoded_friends_data = if friends_data.is_some(){
                        serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
                    } else{
                        vec![]
                    };

                    let mut match_friends = vec![];
                    for friend in decoded_friends_data{
                        
                        if friend.screen_cid.contains(search_q) || 
                            friend.username.contains(search_q){
                                match_friends.push(
                                    friend
                                )
                            }
                        }


                    resp!{
                        UserFanData,
                        UserFanData{ 
                            id: fan_data.id, 
                            user_screen_cid: fan_data.user_screen_cid,
                            friends: {
                                Some(serde_json::to_value(&match_friends).unwrap())
                            }, 
                            invitation_requests: fan_data.invitation_requests, 
                            created_at: fan_data.created_at, 
                            updated_at: fan_data.updated_at 
                        },
                        FETCHED,
                        StatusCode::OK,
                        None::<Cookie<'_>>,
                    }
                    

                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[get("/fan/followings/search/for/{who_screen_cid}/")]
#[passport(user)]
pub(self) async fn search_in_followings(
    req: HttpRequest,
    who_screen_cid: web::Path<String>,
    query: web::Query<UnlimitedSearch>,
    app_state: web::Data<AppState>,
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    
    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    let search_q = &query.q;
                    let get_friends = UserFan::get_all_my_followings_without_limit(&who_screen_cid.to_owned(), connection).await;
                    let Ok(followings_data) = get_friends else{
                        let err_resp = get_friends.unwrap_err();
                        return err_resp;
                    };

                    let mut match_followings = vec![];
                    for following in followings_data{
                        
                        let following_screen_cid = following.clone().user_wallet_info.screen_cid.unwrap();
                        let following_username = following.clone().user_wallet_info.username;

                        if following_screen_cid.contains(search_q) || 
                            following_username.contains(search_q){
                                match_followings.push(
                                    following
                                )
                            }
                        }

                    match_followings.sort_by(|uw1, uw2|{
                            
                        let uw1_created_at = NaiveDateTime
                            ::parse_from_str(&uw1.created_at, "%Y-%m-%d %H:%M:%S%.f")
                            .unwrap();

                        let uw2_created_at = NaiveDateTime
                            ::parse_from_str(&uw2.created_at, "%Y-%m-%d %H:%M:%S%.f")
                            .unwrap();

                        uw2_created_at.cmp(&uw1_created_at)

                    });


                    resp!{
                        Vec<UserFanDataWithWalletInfo>,
                        match_followings,
                        FETCHED,
                        StatusCode::OK,
                        None::<Cookie<'_>>,
                    }
                    

                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }
    
}

#[get("/fan/search/relations/for/{who}/")]
#[passport(user)]
pub(self) async fn search_in_all_user_relations(
    req: HttpRequest,
    who_screen_cid: web::Path<String>,
    query: web::Query<UnlimitedSearch>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserFan::get_user_relations_without_limit(
                        &who_screen_cid.to_owned(), 
                        connection).await{
                        
                        Ok(mut user_relations) => {

                            let search_q = &query.q;
                            let followers = user_relations.clone().followers;
                            let friends = user_relations.clone().friends;
                            // taking a mutable reference to the underlying followings data which updates
                            // followings field inside the user_relations if any of the following info 
                            // inside the followings vector gets updated
                            let followings = &mut user_relations.followings; 

                            let mut match_friends = vec![];
                            let mut match_followers = vec![];
                            
                            // following is a mutable pointer so if we update one of its field
                            // then the followings vector gets updated too with the latest changes
                            // of this element
                            for following in followings{ 
                                let user_friends_data_followings = following.clone().friends;
                                let mut decoded_friends_data_followings = if user_friends_data_followings.is_some(){
                                    serde_json::from_value::<Vec<FriendData>>(user_friends_data_followings.unwrap()).unwrap()
                                } else{
                                    vec![]
                                }; 

                                let mut match_followings = vec![];
                                for frd in decoded_friends_data_followings{
                                    if frd.username.contains(search_q) || 
                                        frd.screen_cid.contains(search_q){
                                            match_followings.push(frd);
                                        }
                                }

                                following.friends = {
                                    Some(serde_json::to_value(&match_followings).unwrap())
                                };
                            }
                                
                            let user_friends_data_followers = followers.construct_friends_data(connection);
                            let mut decoded_friends_data_followers = if user_friends_data_followers.is_some(){
                                serde_json::from_value::<Vec<FriendData>>(user_friends_data_followers.unwrap()).unwrap()
                            } else{
                                vec![]
                            }; 

                            let user_friends_data_friends = friends.construct_friends_data(connection);
                            let mut decoded_friends_data_friends = if user_friends_data_friends.is_some(){
                                serde_json::from_value::<Vec<FriendData>>(user_friends_data_friends.unwrap()).unwrap()
                            } else{
                                vec![]
                            }; 

                            for frd in decoded_friends_data_followers{
                                if frd.username.contains(search_q) || 
                                    frd.screen_cid.contains(search_q){
                                        match_followers.push(frd);
                                    }
                            }

                            for frd in decoded_friends_data_friends{
                                if frd.username.contains(search_q) || 
                                    frd.screen_cid.contains(search_q){
                                        match_friends.push(frd);
                                    }
                            }

                            user_relations.followers.friends = {
                                Some(serde_json::to_value(&match_followers).unwrap())
                            };

                            user_relations.friends.friends = {
                                Some(serde_json::to_value(&match_friends).unwrap())
                            };

                            resp!{
                                UserRelations, //// the data type
                                user_relations, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }


                        },
                        Err(resp) => {
                            resp
                        }
                    
                    }
                    
                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[get("/fan/search/suggestions/for/")]
#[passport(user)]
pub(self) async fn search_in_friend_suggestions_for_owner(
    req: HttpRequest,
    query: web::Query<UnlimitedSearch>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match User::suggest_user_to_owner_without_limit(
                        &user.screen_cid.unwrap(),
                        connection).await{
                        Ok(suggestions) => {

                            let search_q = &query.q;
                            let mut match_suggestions = vec![];
                            for uw in suggestions{
                                if uw.username.contains(search_q) || 
                                    uw.screen_cid.as_ref().unwrap().contains(search_q){
                                        match_suggestions.push(uw);
                                    }
                            }

                            resp!{
                                Vec<UserWalletInfoResponseForUserSuggestions>, //// the data type
                                match_suggestions, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    }
                    

                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[get("/fan/search/unaccepted/friend-requests/")]
#[passport(user)]
pub(self) async fn search_in_unaccepted_friend_request(
    req: HttpRequest,
    query: web::Query<UnlimitedSearch>,
    app_state: web::Data<AppState>,
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserFan::get_user_unaccepted_friend_requests_without_limit(
                        &user.screen_cid.unwrap(),
                        connection).await{
                        
                        Ok(mut unaccepted_requests) => {

                            let search_q = &query.q;
                            unaccepted_requests.retain(|frd| frd.is_some()); // remove none ones
                            let mut match_unaccepted_frd_reqs = vec![];
                            for frd in unaccepted_requests{
                                if frd.as_ref().unwrap().username.contains(search_q) || 
                                    frd.as_ref().unwrap().screen_cid.contains(search_q){
                                        match_unaccepted_frd_reqs.push(frd);
                                    }
                            }

                            resp!{
                                Vec<Option<FriendData>>, //// the data type
                                match_unaccepted_frd_reqs, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    
                    }
                    
                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }
    
}

#[get("/fan/followers/search/for/{who_screen_cid}/")]
#[passport(user)]
pub(self) async fn search_in_followers(
    req: HttpRequest,
    query: web::Query<UnlimitedSearch>,
    who_screen_cid: web::Path<String>,
    app_state: web::Data<AppState>,
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    
    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }


                    let search_q = &query.q;
                    let get_friends = UserFan::get_all_my_followers_without_limit(&who_screen_cid.to_owned(), connection).await;
                    let Ok(fan_data) = get_friends else{
                        let err_resp = get_friends.unwrap_err();
                        return err_resp;
                    };

                    let friends_data = fan_data.friends;
                    let mut decoded_friends_data = if friends_data.is_some(){
                        serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
                    } else{
                        vec![]
                    };

                    let mut match_followers = vec![];
                    for friend in decoded_friends_data{
                        
                        if friend.screen_cid.contains(search_q) || 
                            friend.username.contains(search_q){
                                match_followers.push(
                                    friend
                                )
                            }
                        }


                    resp!{
                        UserFanData,
                        UserFanData{ 
                            id: fan_data.id, 
                            user_screen_cid: fan_data.user_screen_cid,
                            friends: {
                                Some(serde_json::to_value(&match_followers).unwrap())
                            }, 
                            invitation_requests: fan_data.invitation_requests, 
                            created_at: fan_data.created_at, 
                            updated_at: fan_data.updated_at 
                        },
                        FETCHED,
                        StatusCode::OK,
                        None::<Cookie<'_>>,
                    }
                    

                },
                Err(resp) => {
                
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }
    
}

pub mod exports{
    pub use super::get_user_unaccepted_friend_requests;
    pub use super::get_all_my_friends;
    pub use super::get_all_my_followers;
    pub use super::get_all_my_followings;
    pub use super::get_all_user_relations;
    pub use super::get_friend_suggestions_for_owner;
    pub use super::send_friend_request_to;
    pub use super::remove_user_from_follower;
    pub use super::remove_user_from_friend;
    pub use super::remove_user_from_following;
    pub use super::accept_friend_request;
    pub use super::search_in_followers;
    pub use super::search_in_followings;
    pub use super::search_in_friends;
    pub use super::search_in_unaccepted_friend_request;
    pub use super::search_in_friend_suggestions_for_owner;
    pub use super::search_in_all_user_relations;
}