




use futures_util::TryStreamExt; /* is needed to call the try_next() method on the mongodb cursor to iterate over future objects */
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use crate::*;
use crate::resp;
use crate::passport;
use crate::constants::*;
use crate::misc::*;
use s3::*;
use crate::models::{
    users::UserData,
    users_tasks::FetchUserTaskReport,
    tasks::TaskData
};


/*
     -------------------------------
    |          SWAGGER DOCS
    | ------------------------------
    |
    |

*/
#[derive(OpenApi)]
#[openapi(
    paths(
        get_admin_data,
        get_user_data,
    ),
    components(
        schemas(
            UserData,
            FetchUserTaskReport,
            TaskData
        )
    ),
    tags(
        (name = "crate::apis::dev", description = "Dev Endpoints")
    ),
    info(
        title = "Dev Access APIs"
    ),
    modifiers(&SecurityAddon),
)]
pub struct DevApiDoc;
struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "jwt",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        )
    }
}


/*
     ------------------------
    |          APIS
    | ------------------------
    |
    |

*/
#[utoipa::path(
    context_path = "/dev",
    responses(
        (status=200, description="Fetched Successfully", body=[u8]),
        (status=403, description="Invalid Token", body=[u8]),
        (status=403, description="No Authorization Header Is Provided", body=[u8]),
        (status=500, description="Storage Issue", body=[u8])
    ),
    params(
        ("admin_id" = String, Path, description = "admin id")
    ),
    tag = "crate::apis::dev",
    security(
        ("jwt" = [])
    )
)]
#[get("/mafia/get/admin/{admin_id}/data/")]
async fn get_admin_data(
        req: HttpRequest, 
        admin_id: web::Path<String>, // mongodb object id of admin or god
        limit: web::Query<Limit>,
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    
    if let Some(header_value) = req.headers().get("Authorization"){
    
        let token = header_value.to_str().unwrap();
        
        /*
            @params: 
                - @token          → JWT

            note that this token must be taken from the conse mafia hyper server
        */
        match mafia_passport!{ token }{
            true => {

                // -------------------------------------------------------------------------------------
                // ------------------------------- ACCESS GRANTED REGION -------------------------------
                // -------------------------------------------------------------------------------------

                let storage = storage.as_ref().to_owned();
                let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
                let mongo_db = storage.clone().unwrap();


                match storage.clone().unwrap().get_pgdb().await{
                    Some(pg_pool) => {
            

                        let god_id = admin_id.to_owned();
                        let host = env::var("HOST").expect("⚠️ no host variable set");
                        let port = env::var("MAFIA_PORT").expect("⚠️ no port variable set");
                        let get_event_api = format!("http://{}:{}/event/get/all/god-with-id/{}", host, port, god_id);
                        let mut all_god_events = Vec::<EventInfo>::new();

                        let get_response_value = reqwest::Client::new()
                            .get(get_event_api.as_str())
                            .header("Authorization", token)
                            .send()
                            .await;

                        let Ok(response_value) = get_response_value else{

                            let err = get_response_value.unwrap_err();
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                &err.to_string(), // response message
                                StatusCode::EXPECTATION_FAILED, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        };

                        /* if we're here means that the conse mafia hyper server is up and we got a response from it */
                        let response_value = response_value.json::<serde_json::Value>().await.unwrap();

                        let data = response_value.get("data");
                        if data.is_some(){

                            let events = data.unwrap().to_owned();
                            all_god_events = serde_json::from_value::<Vec<EventInfo>>(events).unwrap();
                            
                        }

                        if all_god_events.is_empty(){
                            let resp_message_value = response_value.get("message").unwrap().to_owned();
                            let resp_message = serde_json::from_value::<String>(resp_message_value).unwrap();

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                &resp_message, // response message
                                StatusCode::EXPECTATION_FAILED, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        } else{

                            let from = limit.from.unwrap_or(0) as usize;
                            let to = limit.to.unwrap_or(10) as usize;

                            if to < from {
                                let resp = Response::<'_, &[u8]>{
                                    data: Some(&[]),
                                    message: INVALID_QUERY_LIMIT,
                                    status: 406,
                                };
                                return Ok(HttpResponse::NotAcceptable().json(resp));
                                
                            }

                            let limited_all_god_events = &all_god_events[from..to].to_vec();

                            resp!{
                                Vec<EventInfo>, // the data type
                                limited_all_god_events.to_owned(), // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
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

                // -------------------------------------------------------------------------------------
                // -------------------------------------------------------------------------------------
                // -------------------------------------------------------------------------------------

            },
            false => {
                
                resp!{
                    &[u8], // the date type
                    &[], // the data itself
                    INVALID_TOKEN, // response message
                    StatusCode::FORBIDDEN, // status code
                    None::<Cookie<'_>>, // cookie
                }
            }
        }

    } else{
        
        resp!{
            &[u8], // the date type
            &[], // the data itself
            NOT_AUTH_HEADER, // response message
            StatusCode::FORBIDDEN, // status code
            None::<Cookie<'_>>, // cookie
        }
    }

}

#[utoipa::path(
    context_path = "/dev",
    responses(
        (status=200, description="Fetched Successfully", body=[u8]),
        (status=403, description="Invalid Token", body=[u8]),
        (status=403, description="No Authorization Header Is Provided", body=[u8]),
        (status=500, description="Storage Issue", body=[u8])
    ),
    params(
        ("user_id" = String, Path, description = "user id")
    ),
    tag = "crate::apis::dev",
    security(
        ("jwt" = [])
    )
)]
#[get("/mafia/get/user/{user_id}/data/")]
async fn get_user_data(
        req: HttpRequest, 
        limit: web::Query<Limit>,
        user_id: web::Path<String>, // mongodb object id of user or player  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    
    if let Some(header_value) = req.headers().get("Authorization"){
    
        let token = header_value.to_str().unwrap();
        
        /*
            @params: 
                - @token          → JWT

            note that this token must be taken from the conse mafia hyper server
        */
        match mafia_passport!{ token }{
            true => {

                // -------------------------------------------------------------------------------------
                // ------------------------------- ACCESS GRANTED REGION -------------------------------
                // -------------------------------------------------------------------------------------

                let storage = storage.as_ref().to_owned();
                let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
                let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
                
                match storage.clone().unwrap().get_pgdb().await{
                    Some(pg_pool) => {
                        
                        let user_id = user_id.to_owned();
                        let host = env::var("HOST").expect("⚠️ no host variable set");
                        let port = env::var("MAFIA_PORT").expect("⚠️ no port variable set");
                        let get_event_api = format!("http://{}:{}/event/get/all/player/{}", host, port, user_id);
                        let mut player_events = Vec::<PlayerEventInfo>::new();
                        

                        let get_response_value = reqwest::Client::new()
                            .get(get_event_api.as_str())
                            .header("Authorization", token)
                            .send()
                            .await;

                        let Ok(response_value) = get_response_value else{

                            let err = get_response_value.unwrap_err();
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                &err.to_string(), // response message
                                StatusCode::EXPECTATION_FAILED, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        };

                        /* if we're here means that the conse mafia hyper server is up and we got a response from it */
                        let response_value = response_value.json::<serde_json::Value>().await.unwrap();

                        let data = response_value.get("data");
                        if data.is_some(){

                            let events = data.unwrap().to_owned();
                            player_events = serde_json::from_value::<Vec<PlayerEventInfo>>(events).unwrap();
                            
                        }

                        if player_events.is_empty(){
                            let resp_message_value = response_value.get("message").unwrap().to_owned();
                            let resp_message = serde_json::from_value::<String>(resp_message_value).unwrap();

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                &resp_message, // response message
                                StatusCode::EXPECTATION_FAILED, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        } else{

                            let from = limit.from.unwrap_or(0) as usize;
                            let to = limit.to.unwrap_or(10) as usize;

                            if to < from {
                                let resp = Response::<'_, &[u8]>{
                                    data: Some(&[]),
                                    message: INVALID_QUERY_LIMIT,
                                    status: 406,
                                };
                                return Ok(HttpResponse::NotAcceptable().json(resp));
                                
                            }

                            let limited_player_events = &player_events[from..to].to_vec();

                            resp!{
                                Vec<PlayerEventInfo>, // the data type
                                limited_player_events.to_owned(), // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
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

                // -------------------------------------------------------------------------------------
                // -------------------------------------------------------------------------------------
                // -------------------------------------------------------------------------------------

            },
            false => {
                
                resp!{
                    &[u8], // the date type
                    &[], // the data itself
                    INVALID_TOKEN, // response message
                    StatusCode::FORBIDDEN, // status code
                    None::<Cookie<'_>>, // cookie
                }
            }
        }

    } else{
        
        resp!{
            &[u8], // the date type
            &[], // the data itself
            NOT_AUTH_HEADER, // response message
            StatusCode::FORBIDDEN, // status code
            None::<Cookie<'_>>, // cookie
        }
    }

}



pub mod exports{
    pub use super::get_admin_data;
    pub use super::get_user_data;
}