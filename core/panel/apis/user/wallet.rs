


pub use super::*;

#[get("/get-token-value/{tokens}")]
#[passport(user)]
pub(self) async fn get_token_value(
        req: HttpRequest,  
        tokens: web::Path<i64>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let async_redis_client = storage.as_ref().clone().unwrap().get_async_redis_pubsub_conn().await.unwrap();

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

                    /* 
                        Note: The usd field in response need to gets divided by 10000000 to extract 
                        the exact amount of token based in USD.
                    */
                    let value = gastracker::calculate_token_value(tokens.to_owned(), redis_client.clone()).await;
                    
                    if let Err(resp) = TokenStatInfo::save(
                        TokenStatInfoRequest{
                            user_id: _id,
                            requested_tokens: tokens.to_owned(),
                            usd_token_price: value.0,
                        },
                        connection).await{
                            error!("can't store token stat info into db, check the log file (TokenStatInfo::save)");
                        }
                    
                    resp!{
                        GetTokenValueResponse, // the data type
                        GetTokenValueResponse{
                            usd: value.0,
                            irr: value.1,
                        }, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
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

#[get("/get-gas-fee")]
#[passport(user)]
pub(self) async fn get_gas_fee(
        req: HttpRequest,  
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let async_redis_client = storage.as_ref().clone().unwrap().get_async_redis_pubsub_conn().await.unwrap();

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

                    let get_gas_fee = gastracker::calculate_gas_in_token(redis_client.clone()).await;
                    let Ok(gas) = get_gas_fee else{
                        let resp_err = get_gas_fee.unwrap_err();
                        return resp_err;
                    };

                    resp!{
                        i64, // the data type
                        gas, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
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

#[post("/cid/wallet/stripe/charge")]
#[passport(user)]
pub(self) async fn charge_wallet_request(
    req: HttpRequest,
    charge_wallet_request: web::Json<ChargeWalletRequest>,
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

                    let identifier_key = format!("{}-charge-wallet-request", _id);

                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "charge_wallet_request");
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

                        let charge_wallet_request_object = charge_wallet_request.to_owned();
                        
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
                            &charge_wallet_request_object.buyer_cid, 
                            &charge_wallet_request_object.tx_signature, 
                            &charge_wallet_request_object.hash_data, 
                            None, /* no need to charge the user for this call */
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        if charge_wallet_request_object.tokens < 5{

                                resp!{
                                    i32, // the data type
                                    _id, // response data
                                    INVALID_TOKEN_AMOUNT, // response message
                                    StatusCode::NOT_ACCEPTABLE, // status code
                                    None::<Cookie<'_>>, // cookie
                                }

                            }

                        if user.region.is_none(){

                            resp!{
                                i32, // the data type
                                _id, // response data
                                REGION_IS_NONE, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        }

                        let u_region = user.region.as_ref().unwrap();
                        let token_price = gastracker::calculate_token_value(charge_wallet_request_object.tokens, redis_client.clone()).await;

                        match u_region.as_str(){
                            "ir" => {

                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    NOT_IMPLEMENTED, // response message
                                    StatusCode::NOT_IMPLEMENTED, // status code
                                    None::<Cookie<'_>>, // cookie
                                }

                            },
                            /** -------------------------------------------------------------- */
                            /** --------------------- use stripe gateway --------------------- */
                            /** -------------------------------------------------------------- */
                            _ => {

                                /* means that the api call can't return prices */
                                if token_price.0 == 0{
                                    resp!{
                                        &[u8], // the data type
                                        &[], // response data
                                        CANT_GET_TOKEN_VALUE, // response message
                                        StatusCode::EXPECTATION_FAILED, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }
                                }
                                /* 
                                    stripe will divide the amount by 100 in checkout page to get the cent value
                                */
                                let usd_token_price = token_price.0;

                                /*  -------------------------------------------------------------
                                    note that we don't store the product, price and session data
                                    response came from stripe in db cause later on we can fetch 
                                    a single data of either product, price or checkout session 
                                    using stripe api or can be viewable in stripe dashboard
                                    ------------------------------------------------------------- */
                                let product = create_product(
                                    redis_client.clone(), 
                                    usd_token_price, 
                                    charge_wallet_request.tokens, 
                                    &charge_wallet_request.buyer_cid
                                ).await;

                                if product.id.is_none(){
                                    
                                    resp!{
                                        &[u8], // the data type
                                        &[], // response data
                                        STRIPE_PRODUCT_OBJECT_ISSUE, // response message
                                        StatusCode::EXPECTATION_FAILED, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }
                                }

                                let price_id = create_price(
                                    redis_client.clone(), 
                                    usd_token_price, 
                                    &product.id.as_ref().unwrap(),
                                ).await;

                                if price_id.is_empty(){

                                    resp!{
                                        &[u8], // the data type
                                        &[], // response data
                                        STRIPE_PRICE_OBJECT_ISSUE, // response message
                                        StatusCode::EXPECTATION_FAILED, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }
                                }

                                let checkout_session_data = create_session(
                                    redis_client.clone(), 
                                    &price_id, 
                                    charge_wallet_request.tokens,
                                    /*  
                                        since calling unwrap() takes the ownership of the object
                                        and the type will be dropped from the ram, thus if the type
                                        is being used in other scopes it's better to borrow it 
                                        or clone it which we've used as_ref() to borrow it also if 
                                        the user is here means that he has definitely the cid cause 
                                        to build cid we need verified mail
                                    */
                                    user.region.as_ref().unwrap(),
                                    user.mail.as_ref().unwrap() 
                                ).await;

                                if checkout_session_data.session_id.is_empty() || 
                                    checkout_session_data.session_url.is_empty() ||
                                    checkout_session_data.expires_at == 0{

                                    resp!{
                                        &[u8], // the data type
                                        &[], // response data
                                        STRIPE_SESSION_OBJECT_ISSUE, // response message
                                        StatusCode::EXPECTATION_FAILED, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }
                                }

                                /* store users_checkouts data */
                                let new_user_checkout = NewUserCheckout{
                                    user_cid: charge_wallet_request_object.buyer_cid,
                                    product_id: product.id.unwrap(),
                                    price_id,
                                    payment_status: checkout_session_data.payment_status,
                                    payment_intent: checkout_session_data.payment_intent,
                                    c_status: checkout_session_data.status,
                                    checkout_session_url: checkout_session_data.session_url,
                                    checkout_session_id: checkout_session_data.session_id,
                                    checkout_session_expires_at: checkout_session_data.expires_at,
                                    tokens: charge_wallet_request_object.tokens,
                                    usd_token_price,
                                    tx_signature: charge_wallet_request_object.tx_signature,
                                };
                                match UserCheckout::insert(new_user_checkout, connection).await{
                                    Ok(user_checkout_data) => {

                                        resp!{
                                            UserCheckoutData, // the data type
                                            user_checkout_data, // response data
                                            STRIPE_STARTED_PAYAMENT, // response message
                                            StatusCode::OK, // status code
                                            None::<Cookie<'_>>, // cookie
                                        }
                                    },
                                    Err(resp) => {
                                        resp
                                    }
                                }

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

#[post("/cid/build")]
#[passport(user)]
pub(self) async fn make_cid(
    req: HttpRequest,
    id_: web::Json<NewIdRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let new_object_id_request = id_.0;
    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();
    
    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            let mut user_ip = "".to_string();

            /* ---------------------------------------------------------------------------------
                if we're getting 127.0.0.1 for client ip addr inside the incoming request means
                the address 127.0.0.1 is the loopback address, which means the request is 
                coming from the same machine where the server is running. if we're running 
                both the server and the browser on the same computer and we're connecting 
                to localhost or 127.0.0.1 in the browser, then this behavior is expected.
                if Actix application is behind a reverse proxy like Nginx or Apache, the proxy 
                may be forwarding requests to your application in such a way that all client 
                connections appear to come from the loopback address. to fix this issue and get 
                the original client's IP address, you can use the X-Forwarded-For or X-Real-IP 
                headers. These headers are typically set by the reverse proxy to indicate the 
                original IP address of the client, also we have to make sure that these are set
                inside the nginx config file:

                proxy_set_header Host $host;
                proxy_set_header X-Real-IP $remote_addr;
                proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
               ---------------------------------------------------------------------------------
            */
            if let Some(header) = req.headers().get("X-Forwarded-For") {
                if let Ok(ip_str) = header.to_str() {
                    user_ip = ip_str.to_string();
                }
            }

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

            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* check that the user has a verified mail or not */
                    let get_user = User::find_by_id(_id, connection).await;
                    let Ok(user) = get_user else{
                        
                        let err_resp = get_user.unwrap_err();
                        return err_resp;
                    };

                    /* if the mail wasn't verified user can't create id */
                    if user.mail.is_none() || 
                        !user.is_mail_verified{
                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            NOT_VERIFIED_MAIL, // response message
                            StatusCode::NOT_ACCEPTABLE, // status code
                            None::<Cookie<'_>>, // cookie
                        }
                    }

                    /* ------------------------------------------------------------ */
                    /* ------------------ NO NEED TO BE VERIFIED ------------------ */
                    /* ------------------------------------------------------------ */
                    /* if the phone wasn't verified user can't create id */
                    // if user.phone_number.is_none() || 
                    //     !user.is_phone_verified{
                    //     resp!{
                    //         &[u8], // the date type
                    //         &[], // the data itself
                    //         NOT_VERIFIED_PHONE, // response message
                    //         StatusCode::NOT_ACCEPTABLE, // status code
                    //         None::<Cookie<'_>>, // cookie
                    //     }
                    // }
                    /* ------------------------------------------------------------ */
                    /* ------------------------------------------------------------ */
                    /* ------------------------------------------------------------ */

                    let identifier_key = format!("{}-make-cid", _id);

                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "make_cid");
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
                        
                        /* building new id contains the public and private key and the snowflake id */
                        let get_new_id = Id::new_or_update(
                            new_object_id_request.clone(), 
                            _id, 
                            /* 
                                we're using new_object_id_request username since 
                                the username inside the JWT might be empty
                            */
                            new_object_id_request.username.clone().to_lowercase(),
                            user_ip,
                            redis_client.to_owned(),
                            redis_actix_actor.clone(),
                            connection
                        ).await;

                        /* 
                            if we found a user simply we'll update all its fields with 
                            new one inside the body of NewIdRequest object except cid and 
                            the snowflake id, then return the updated data as the err response 
                            of the new_or_update method.
                        */
                        let Ok(mut new_id) = get_new_id else{
                            let resp = get_new_id.unwrap_err();
                            return resp;
                        };

                        /* 
                            saveing the new Id into db, also if we're here means 
                            that we're creating a new Id for the user since on the 
                            second request it'll return the founded user info
                        */
                        let save_user_data = new_id.save(redis_client.to_owned(), redis_actix_actor.clone(), connection).await;
                        let Ok(user_data) = save_user_data else{
                            let resp = save_user_data.unwrap_err();
                            return resp;
                        };

                        /* ----------------------------------------------- */
                        /* creating a default private gallery for the user */
                        /* ----------------------------------------------- */
                        let create_new_gal = UserPrivateGallery::insert(
                            NewUserPrivateGalleryRequest{
                                owner_cid: new_id.clone().new_cid.unwrap_or(String::from("")),
                                gal_name: format!("{} first private room at time {}", new_id.username, chrono::Local::now().to_string()),
                                gal_description: format!("{} first private room at time {}", new_id.username, chrono::Local::now().to_string()),
                                extra: None,
                                tx_signature: String::from(""),
                                hash_data: String::from(""),
                            }, redis_actix_actor.clone(), connection).await;
                        
                        let Ok(new_gal) = create_new_gal else{
                            let error_resp = create_new_gal.unwrap_err();
                            return error_resp;
                        };
                        
                        resp!{
                            UserIdResponse, // the data type
                            user_data, // response data
                            ID_BUILT, // response message
                            StatusCode::CREATED, // status code
                            None::<Cookie<'_>>, // cookie
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

#[post("/deposit/to/{contract_address}")]
#[passport(user)]
pub(self) async fn deposit(
    req: HttpRequest,
    contract_address: web::Path<String>,
    deposit: web::Json<NewUserDepositRequest>,
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

                    let identifier_key = format!("{}-deposit", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "deposit");
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

                        let deposit_object = deposit.to_owned();

                        let find_recipient_screen_cid = User::find_by_username_or_mail_or_scid(&deposit_object.recipient, connection).await;
                        let Ok(recipient_info) = find_recipient_screen_cid else{
                            
                            resp!{
                                String, // the data type
                                deposit_object.recipient, // response data
                                &RECIPIENT_NOT_FOUND, // response message
                                StatusCode::NOT_FOUND, // status code
                                None::<Cookie<'_>>, // cookie
                            }
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
                            &deposit_object.from_cid, 
                            &deposit_object.tx_signature, 
                            &deposit_object.hash_data, 
                            Some(deposit_object.amount), 
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };



                        let new_balance = user.balance.unwrap() - (deposit_object.amount + gastracker::calculate_gas_in_token(redis_client.clone()).await.unwrap());
                        let mut mint_tx_hash = String::from("");
                        let mut token_id = String::from("");

                        /* deposit_object.recipient_screen_cid must be the keccak256 of the recipient public key */
                        if recipient_info.screen_cid.is_none(){
                            resp!{
                                String, // the date type
                                deposit_object.recipient, // the data itself
                                RECIPIENT_SCREEN_CID_NOT_FOUND, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        }

                        if recipient_info.cid.clone().unwrap() == deposit_object.from_cid{

                            resp!{
                                String, // the date type
                                deposit_object.from_cid, // the data itself
                                SENDER_CANT_BE_RECEIVER, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        }

                        let polygon_recipient_address = recipient_info.clone().screen_cid.unwrap();
                        let contract_owner = env::var("GIFT_CARD_POLYGON_NFT_OWNER_ADDRESS").unwrap();
                        let contract_address_env = env::var("GIFT_CARD_POLYGON_NFT_CONTRACT_ADDRESS").unwrap();

                        if contract_address.to_owned() != contract_address_env{
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                INVALID_CONTRACT_ADDRESS, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        }


                        // sender and recipient must be friend of each other
                        let depositor_screen_cid = &user.screen_cid.as_ref().unwrap();
                        let check_we_are_friend = UserFan::are_we_friends(
                            &polygon_recipient_address.clone(), 
                            depositor_screen_cid, connection).await;
                        
                        if check_we_are_friend.is_ok() && *check_we_are_friend.as_ref().unwrap(){

                            let (tx_hash, tid, res_mint_status) = start_minting_card_process(
                                depositor_screen_cid.to_string(),
                                deposit_object.clone(),  
                                contract_address.clone(),
                                contract_owner.clone(),
                                polygon_recipient_address.clone(),
                                deposit_object.nft_img_url.clone(),
                                deposit_object.nft_name,
                                deposit_object.nft_desc,
                                redis_client.clone(),
                                redis_actix_actor.clone(), 
                                connection
                            ).await;
                            
                            if res_mint_status == 1{
    
                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    CANT_MINT_CARD, // response message
                                    StatusCode::EXPECTATION_FAILED, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
                            }
    
                            mint_tx_hash = tx_hash; // moving into another type
                            token_id = tid;
    
                            let mut uubd = None;

                            if !mint_tx_hash.is_empty(){

                                let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actix_actor.clone(), connection).await;
                                let Ok(updated_user_balance_data) = update_user_balance else{
                                    
                                    let err_resp = update_user_balance.unwrap_err();
                                    return err_resp;
                                    
                                };

                                uubd = Some(updated_user_balance_data);
                                
                                match UserDeposit::insert(deposit.to_owned(), mint_tx_hash, token_id, polygon_recipient_address, deposit_object.nft_img_url, redis_actix_actor, connection).await{
                                    Ok(user_deposit_data) => {
    
                                        resp!{
                                            UserDepositData, // the data type
                                            user_deposit_data, // response data
                                            DEPOSITED_SUCCESSFULLY, // response message
                                            StatusCode::CREATED, // status code
                                            None::<Cookie<'_>>, // cookie
                                        }
    
                                    },
                                    Err(resp) => {
                                        /* 
                                            🥝 response can be one of the following:
                                            
                                            - DIESEL INSERT ERROR RESPONSE
                                        */
                                        resp
                                    }
                                }                                    
    
                                
                            } else{
                                
                                if uubd.is_some(){
                                    let updated_user_balance_data = uubd.unwrap();
                                    let new_balance = updated_user_balance_data.balance.unwrap() + deposit_object.amount;
                                    let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actix_actor, connection).await;
                                    let Ok(updated_user_data) = update_user_balance else{
        
                                        let err_resp = update_user_balance.unwrap_err();
                                        return err_resp;
                                        
                                    };
                                }
    
                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    CANT_MINT_CARD, // response message
                                    StatusCode::FAILED_DEPENDENCY, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
                            }

                        } else{
                
                            let recipient_username = recipient_info.clone().username;
                            let depositor_username = user.clone().username;
                            let resp_msg = format!("{recipient_username:} Is Not A Friend Of {depositor_username:}");
                            let resp = Response::<'_, &[u8]>{
                                data: Some(&[]),
                                message: &resp_msg,
                                status: 406,
                                is_error: true
                            };
                            return 
                                Ok(HttpResponse::NotAcceptable().json(resp));
                            
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

#[get("/deposit/get/all/")]
#[passport(user)]
pub(self) async fn get_all_user_deposits(
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
                    if user.cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserDeposit::get_all_for(user.cid.unwrap().to_string(), limit, connection).await{
                        Ok(user_deposits) => {

                            resp!{
                                Vec<UserDepositDataWithWalletInfo>, // the data type
                                user_deposits, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }


                        },
                        Err(resp) => {
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                            */
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

#[post("/withdraw/from/{contract_address}")]
#[passport(user)]
pub(self) async fn withdraw(
    req: HttpRequest,
    contract_address: web::Path<String>,
    withdraw: web::Json<NewUserWithdrawRequest>,
    app_state: web::Data<AppState>,
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

                    let identifier_key = format!("{}-withdraw", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "withdraw");
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
                    if is_rate_limited!{ /* chill 30 seconds */
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

                    } else { /* not rate limited, we're ok to go */

                        let withdraw_object = withdraw.to_owned();
                    
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
                            &withdraw_object.recipient_cid, 
                            &withdraw_object.tx_signature, 
                            &withdraw_object.hash_data, 
                            None, /* no need to charge the user for this call */
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        let get_deposit_info = UserDeposit::find_by_id(withdraw_object.deposit_id, connection).await;
                        let Ok(deposit_info) = get_deposit_info else{

                            let error = get_deposit_info.unwrap_err();
                            return error;
                        };


                        /* generate keccak256 from recipient_cid to check aginst the one in db */
                        let polygon_recipient_address = walletreq::evm::get_keccak256_from(withdraw_object.recipient_cid.to_owned().clone());
                        if deposit_info.recipient_screen_cid != polygon_recipient_address ||
                        withdraw_object.recipient_cid != user.cid.unwrap(){
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                NO_DEPOSIT_FOR_THIS_RECIPIENT, // response message
                                StatusCode::NOT_FOUND, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        }

                        let token_id = deposit_info.nft_id;
                        let mut transfer_tx_hash = String::from("");
                        
                        let res_transfer = start_transferring_card_process(
                            contract_address.to_owned(), 
                            token_id,
                            polygon_recipient_address,
                            redis_client.clone(),
                            redis_actix_actor.clone(), 
                            connection
                        ).await;

                        if res_transfer.1 == 1{

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                CANT_TRANSFER_CARD, // response message
                                StatusCode::EXPECTATION_FAILED, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        }

                        transfer_tx_hash = res_transfer.0; // moving into another type

                        let new_balance = if user.balance.is_none(){0 + deposit_info.amount} else{user.balance.unwrap() + deposit_info.amount};
                        let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actix_actor.clone(), connection).await;
                        let Ok(updated_user_balance_data) = update_user_balance else{

                            let err_resp = update_user_balance.unwrap_err();
                            return err_resp;
                            
                        };

                        if !transfer_tx_hash.is_empty(){

                            match UserWithdrawal::insert(withdraw.to_owned(), transfer_tx_hash, connection).await{
                                Ok(user_withdrawal_data) => {
                                    
                                    resp!{
                                        UserWithdrawalData, // the data type
                                        user_withdrawal_data, // response data
                                        WITHDRAWN_SUCCESSFULLY, // response message
                                        StatusCode::CREATED, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }

                                },
                                Err(resp) => {
                                    /* 
                                        🥝 response can be one of the following:
                                        
                                        - DIESEL INSERT ERROR RESPONSE
                                        - DEPOSIT OBJECT NOT FOUND
                                        - ALREADY_WITHDRAWN
                                    */
                                    resp
                                }
                            }
                                

                        } else{

                            let new_balance = updated_user_balance_data.balance.unwrap() - deposit_info.amount;
                            let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actix_actor, connection).await;
                            let Ok(updated_user_balance_data) = update_user_balance else{

                                let err_resp = update_user_balance.unwrap_err();
                                return err_resp;
                                
                            };

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                CANT_TRANSFER_CARD, // response message
                                StatusCode::EXPECTATION_FAILED, // status code
                                None::<Cookie<'_>>, // cookie
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

#[get("/withdraw/get/all/")]
#[passport(user)]
pub(self) async fn get_all_user_withdrawals(
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
                    if user.cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }
    
                    match UserWithdrawal::get_all_for(user.cid.unwrap().to_string(), limit, connection).await{
                        Ok(user_withdrawals) => {

                            resp!{
                                Vec<UserWithdrawalData>, // the data type
                                user_withdrawals, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }


                        },
                        Err(resp) => {
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                            */
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

#[get("/checkout/get/all/unpaid/")]
#[passport(user)]
pub(self) async fn get_all_user_unpaid_checkouts(
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
                    if user.cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }
    
                    match UserCheckout::get_all_unpaid_for(&user.cid.unwrap(), limit, connection).await{
                        Ok(user_checkouts) => {

                            resp!{
                                Vec<UserCheckoutData>, // the data type
                                user_checkouts, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }


                        },
                        Err(resp) => {
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                            */
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

#[get("/checkout/get/all/paid/")]
#[passport(user)]
pub(self) async fn get_all_user_paid_checkouts(
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
                    if user.cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }
    
                    match UserCheckout::get_all_paid_for(&user.cid.unwrap(), limit, connection).await{
                        Ok(user_checkouts) => {

                            resp!{
                                Vec<UserCheckoutData>, // the data type
                                user_checkouts, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }


                        },
                        Err(resp) => {
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                            */
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

#[get("/deposit/get/all/unclaimed/")]
#[passport(user)]
pub(self) async fn get_recipient_unclaimed_deposits(
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
                    if user.cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    /* generate keccak256 from recipient_cid to mint nft to */
                    let polygon_recipient_address = walletreq::evm::get_keccak256_from(user.cid.unwrap().to_owned().clone());

                    match UserDeposit::get_unclaimeds_for(polygon_recipient_address, limit, connection).await{
                        Ok(user_unclaimeds) => {

                            resp!{
                                Vec<UserDepositDataWithWalletInfo>, // the data type
                                user_unclaimeds, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        },
                        Err(resp) => {
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                            */
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

#[get("/get-my-wallet-info")]
#[passport(user)]
async fn get_my_wallet_info(
        req: HttpRequest,   
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

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
    
                        
                        match User::fetch_wallet_by_username_or_mail_or_scid_or_cid(&user.username, connection).await{

                            Ok(user_info) => {
            
                                resp!{
                                    UserWalletInfoResponse, // the data type
                                    user_info, // response data
                                    FETCHED, // response message
                                    StatusCode::OK, // status code
                                    None::<Cookie<'_>>, // cookie
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

pub mod exports{
    pub use super::make_cid;
    pub use super::get_my_wallet_info;
    pub use super::get_token_value;
    pub use super::get_gas_fee;
    pub use super::get_all_user_withdrawals;
    pub use super::get_all_user_deposits;
    pub use super::get_recipient_unclaimed_deposits;
    pub use super::get_all_user_unpaid_checkouts;
    pub use super::get_all_user_paid_checkouts;
    pub use super::deposit; /**** gift card money transfer ****/
    pub use super::withdraw; /**** gift card money claim ****/
    pub use super::charge_wallet_request; /**** buy in-app token ****/
}