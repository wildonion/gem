

use crate::*;



#[derive(Serialize, Deserialize, Debug)]
pub struct Response<'m, T>{
    pub data: Option<T>,
    pub message: &'m str, //// &str are a slice of String thus they're behind a pointer and every pointer needs a valid lifetime which is 'm in here 
    pub status: u16,
}


/*
    we can define as many as response object 
    since once the scope or method or the match
    arm gets executed the lifetime of the response
    object will be dropped from the ram since rust 
    doesn't have gc :) 
*/
#[macro_export]
macro_rules! resp {
    (
        $data:expr,
        $msg:expr,
        $code:expr,
    ) => {

        {
            use actix_web::HttpResponse;
            use crate::misc::Response;
            
            let code = $code.as_u16();
            let mut res = HttpResponse::build($code);
            
            let response_data = Response{
                data: Some($data),
                message: $msg,
                status: code
            };
            
            let response = Ok(
                res
                    .json(
                        response_data
                    )
            );
            return response;
        }
    }
}


#[macro_export]
macro_rules! server {
    (
        $api_services:expr
    ) => {
        
        {

            use std::env;
            use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse, get, ResponseError};
            use actix_web::middleware::Logger;
            

            env_logger::init_from_env(Env::default().default_filter_or("info"));
            let host = std::env::var("HOST").expect("⚠️ no host variable set");
            let port = std::env::var("PANEL_PORT").expect("⚠️ no panel port variable set").parse::<u16>().unwrap();
            let surrealdb_port = std::env::var("SURREALDB_PORT").expect("⚠️ no surrealdb port variable set").parse::<u16>().unwrap();
            let surrealdb_host = std::env::var("SURREALDB_HOST").expect("⚠️ no surrealdb host variable set");
            let redis_node_addr = std::env::var("REDIS_HOST").expect("⚠️ no redis host variable set");
        
            let client = redis::Client::open(redis_node_addr.as_str()).unwrap();
            let redis_conn = client.get_async_connection().await.unwrap();
            let arced_redis_conn = Arc::new(redis_conn);
            let surrealdb_addr = format!("{}:{}", surrealdb_host, surrealdb_port);
            let storage = Surreal::new::<Ws>(surrealdb_addr.as_str()).await.unwrap();

            /*
                the HttpServer::new function takes a factory function that 
                produces an instance of the App, not the App instance itself. 
                This is because each worker thread needs to have 
                its own App instance.
            */
            HttpServer::new(move ||{
                App::new()
                    /* 
                        REDIS SHARED STATE
                    */
                    .app_data(arced_redis_conn.clone())
                    /* 
                        SURREALDB SHARED STATE
                    */
                    .app_data(storage.clone())
                    .wrap(Logger::default())
                    .wrap(Logger::new("%a %{User-Agent}i %t %P %r %s %b %T %D"))
                    .configure(services::init)
                }) //// each thread of the HttpServer instance needs its own app factory 
                .bind((host.as_str(), port))
                .unwrap()
                .workers(10)
                .run()
                .await

        }
    };
}