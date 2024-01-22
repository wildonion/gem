




pub mod cors{



    use crate::constants::*;
    use hyper::{header, Body, Response, StatusCode, http::HeaderValue};
    
    // CORS middleware allow method,
    // by adding this api to the router 
    // config we're allowing the client 
    // to access all resources of that router 

    pub async fn allow(mut res: Response<Body>) -> RendezvousResult<Response<Body>, hyper::Error> { // res must be mutable to borrow its headers mutably
        let headers = res.headers_mut();
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
        headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("*"));
        headers.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("true"));
        headers.insert(header::ACCESS_CONTROL_MAX_AGE , HeaderValue::from_static("3600"));
        headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("*"));
        headers.insert(header::ACCESS_CONTROL_EXPOSE_HEADERS, HeaderValue::from_static("*"));
        *res.status_mut() = StatusCode::OK; // NOTE - this is so important since the browsers will check for the 200 status inside the response 
        Ok(res)
    }



}





pub mod logging{

    use crate::constants::*;
    use log::{info, error};
    use hyper::{header, Method, Body, Request};
    use routerify::prelude::RequestExt;

    pub async fn logger(req: Request<Body>) -> Result<Request<Body>, hyper::Error>{
        info!("{} - {} {} {}", chrono::Local::now(), req.remote_addr(), req.method(), req.uri().path());
        Ok(req)
    }

}




pub mod auth{
    
    
    use crate::constants::*;
    use log::{info, error};
    use hyper::{Method, Body};
    use redis::AsyncCommands;
    use crate::misc::jwt;
    use jsonwebtoken::TokenData;
    


    pub async fn pass(req: hyper::Request<Body>) -> Result<(TokenData<jwt::Claims>, hyper::Request<Body>), String>{ // the return type is a Result of type TokenData claims and hyper::Request body
        
        use dotenv::dotenv;
        dotenv().expect("⚠️ .env file not found");
        /* -=-=-=-=-=-=-=-=-=-=-= REDIS SETUP -=-=-=-=-=-=-=-=-=-=-= */

        let redis_password = std::env::var("REDIS_PASSWORD").unwrap_or("".to_string());
        let redis_username = std::env::var("REDIS_USERNAME").unwrap_or("".to_string());
        let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
        let redis_port = std::env::var("REDIS_PORT").unwrap_or("6379".to_string()).parse::<u64>().unwrap();

        let redis_conn_url = if !redis_password.is_empty(){
            format!("redis://:{}@{}:{}", redis_password, redis_host, redis_port)
        } else if !redis_password.is_empty() && !redis_username.is_empty(){
            format!("redis://{}:{}@{}:{}", redis_username, redis_password, redis_host, redis_port)
        } else{
            format!("redis://{}:{}", redis_host, redis_port)
        };

        let redis_client = redis::Client::open(redis_conn_url.as_str()).unwrap();
        let mut redis_conn = redis_client.get_async_connection().await.unwrap();
        
        /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
        
        let mut authenticate_pass: bool = false;
        let mut user_data_inside_token: Option<TokenData<jwt::Claims>> = None;
        let mut jwt_error: Option<jsonwebtoken::errors::Error> = None;
        if Method::OPTIONS == *req.method() {
            authenticate_pass = true; // since OPTIONS route don't need to be checked since it might be fucking CORS preflight request :)
        } else{
            for ignore_route in IGNORE_ROUTES.iter(){
                if req.uri().path().starts_with(ignore_route){
                    authenticate_pass = true;
                    break;
                }
            }
            if !authenticate_pass{
                if let Some(authen_header) = req.headers().get("Authorization"){
                    if let Ok(authen_str) = authen_header.to_str(){
                        if authen_str.starts_with("bearer") || authen_str.starts_with("Bearer"){
                            let token = authen_str[6..authen_str.len()].trim();
                            match jwt::deconstruct(token).await{
                                Ok(token_data) => {

                                    // this must be saved in redis when user do a login
                                    let user_jwt_key = format!("jwt-{}", token_data.claims._id.unwrap().to_string());
                                    let user_jwt: String = redis_conn.get(user_jwt_key).await.unwrap();
                                    if user_jwt.is_empty() || user_jwt != token.to_string(){ // user did a logout
                                        return Err(NOT_FOUND_TOKEN.to_string()); // terminate this scope
                                    }

                                    authenticate_pass = true; // means we've found the token inside the request header and decoded successfully 
                                    user_data_inside_token = Some(token_data);
                                },
                                Err(e) => {
                                    jwt_error = Some(e);
                                }
                            }
                        }
                    }
                } else{
                    return Err(NOT_FOUND_TOKEN.to_string());
                }
            }
        }
        if authenticate_pass{
            Ok((user_data_inside_token.unwrap(), req)) // since we can't copy or clone the req object we have to return the request object back to where it has been called 
        } else{
            Err(jwt_error.unwrap().to_string())
        }
    }



    pub mod user{
        
        use hyper::Body;
        use crate::schemas;
        use mongodb::{Client, bson::{self, doc, oid::ObjectId}}; // self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
        use std::env;
        
        

        pub async fn exists(storage: Option<&Client>, user_id: Option<ObjectId>, access_level: u8) -> bool{
    
            ////////////////// DB Ops

            let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
            let serialized_access_level = bson::to_bson(&access_level).unwrap(); // we have to serialize the access_level to BSON Document object in order to find a user with this info cause mongodb can't do serde ops on raw u8
            let users = storage.unwrap().database(&db_name).collection::<schemas::auth::UserInfo>("users"); // selecting users collection to fetch all user infos into the UserInfo struct
            match users.find_one(doc!{"_id": user_id, "access_level": serialized_access_level}, None).await.unwrap(){ // finding user based on username, _id and access_level
                Some(user_doc) => true, 
                None => false,
            }
    
            //////////////////
     
        }


    }


}