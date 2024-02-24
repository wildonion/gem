


pub use super::*;


#[post("/clp/register")]
#[passport(user)]
pub(self) async fn register_clp_event(
    req: HttpRequest,
    register_clp_event_request: web::Json<RegisterUserClpEventRequest>,
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

                    let identifier_key = format!("{}-register-clp-event", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "register_clp_event");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                        
                        let get_clp_event = ClpEvent::find_by_id(register_clp_event_request.clp_event_id, connection).await;
                        let Ok(clp_event) = get_clp_event else{
                            let err_resp = get_clp_event.unwrap_err();
                            return err_resp;
                        };

                        let get_current_users_in_this_event = UserClp::get_all_users_in_clp_event(register_clp_event_request.clp_event_id, connection).await;
                        let Ok(current_users_in_this_event) = get_current_users_in_this_event else{
                            let err_resp = get_current_users_in_this_event.unwrap_err();
                            return err_resp;
                        };

                        /** ------------------------------------------------------------------ */
                        /** the is_locked field will be updated by clp subscriber actor worker */ 
                        /** ------------------------------------------------------------------ */ 
                        // means it has been started already and can't register any more
                        if clp_event.is_locked{
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                CLP_EVENT_IS_LOCKED, //// response message
                                StatusCode::NOT_ACCEPTABLE, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }
                        }

                        /** ------------------------------------------------------------------ */
                        /** the expire_at field will be updated by clp subscriber actor worker */ 
                        /** ------------------------------------------------------------------ */
                        // means it has been expired already and can't register any more
                        if chrono::Local::now().timestamp() > clp_event.expire_at{
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                CLP_EVENT_IS_EXPIRED, //// response message
                                StatusCode::NOT_ACCEPTABLE, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }
                        }

                        if clp_event.max_supply as usize == current_users_in_this_event.len(){
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                CLP_EVENT_IS_FULL, //// response message
                                StatusCode::NOT_ACCEPTABLE, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }
                        }

                        let register_clp_event_request = register_clp_event_request.to_owned();
                        
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
                            &register_clp_event_request.participant_cid, 
                            &register_clp_event_request.tx_signature, 
                            &register_clp_event_request.hash_data, 
                            Some(clp_event.mint_price),
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        let get_user_clp_event = UserClp::find_by_participant_and_event_id(_id, clp_event.id, connection).await;
                        if get_user_clp_event.is_ok(){
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                USER_CLP_EVENT_ALREADY_REGISTERED, //// response message
                                StatusCode::FOUND, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }
                        }
                        
                        match UserClp::insert(
                            clp_event.mint_price, 
                            _id, 
                            clp_event.id, 
                            redis_client.clone(),
                            redis_actix_actor,
                            connection
                        ).await{
                            Ok(user_clp_data) => {

                                resp!{
                                    UserClp, //// the data type
                                    user_clp_data, //// response data
                                    CREATED, //// response message
                                    StatusCode::OK, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            }, 
                            Err(resp) => {
                                resp
                            }
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

#[post("/clp/cancel")]
#[passport(user)]
pub(self) async fn cancel_clp_event(
    req: HttpRequest,
    cancel_clp_event_request: web::Json<CancelUserClpEventRequest>,
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

                    let identifier_key = format!("{}-cancel-clp-event", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "cancel_clp_event");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                        
                        let cancel_clp_event_request = cancel_clp_event_request.to_owned();
                        
                        let get_clp_event = ClpEvent::find_by_id(cancel_clp_event_request.clp_event_id, connection).await;
                        let Ok(clp_event) = get_clp_event else{
                            let err_resp = get_clp_event.unwrap_err();
                            return err_resp;
                        };

                        
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
                            &cancel_clp_event_request.participant_cid, 
                            &cancel_clp_event_request.tx_signature, 
                            &cancel_clp_event_request.hash_data, 
                            Some(clp_event.mint_price),
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };
                        

                        let get_user_clp_event = UserClp::find_by_participant_and_event_id(_id, clp_event.id, connection).await;
                        if get_user_clp_event.is_ok(){
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                USER_CLP_EVENT_ALREADY_REGISTERED, //// response message
                                StatusCode::FOUND, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }
                        }
                        
                        match UserClp::cancel_reservation(
                            _id, 
                            clp_event.id, 
                            redis_client.clone(),
                            redis_actix_actor,
                            connection
                        ).await{
                            Ok(user_data) => {

                                resp!{
                                    UserData, //// the data type
                                    user_data, //// response data
                                    CREATED, //// response message
                                    StatusCode::OK, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            }, 
                            Err(resp) => {
                                resp
                            }
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

#[get("/clp/get/new")]
#[passport(user)]
pub(self) async fn get_new_clp_event_info(
    req: HttpRequest,
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
                
                    match ClpEvent::get_latest(connection).await{
                        Ok(clp_event_data) => {

                            resp!{
                                ClpEventData, //// the data type
                                clp_event_data, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => resp
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

#[get("/clp/get/all/")]
#[passport(user)]
pub(self) async fn get_all_user_clp_events_info(
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

                    let get_all_my_events = UserClp::get_all_user_events(limit, _id, connection).await;
                    let Ok(all_my_events) = get_all_my_events else{
                        let err_resp = get_all_my_events.unwrap_err();
                        return err_resp;
                    };
                    
                    resp!{
                        Vec<ClpEventData>, //// the data type
                        all_my_events, //// response data
                        FETCHED, //// response message
                        StatusCode::OK, //// status code
                        None::<Cookie<'_>>, //// cookie
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
    pub use super::get_new_clp_event_info;
    pub use super::register_clp_event;
    pub use super::cancel_clp_event;
    pub use super::get_all_user_clp_events_info;
}