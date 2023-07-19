




use futures_util::TryStreamExt; /* is needed to call the try_next() method on the mongodb cursor to iterate over future objects */
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use crate::*;
use crate::resp;
use crate::passport;
use crate::constants::*;
use crate::misc::*;
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
#[get("/get/admin/{admin_id}/data")]
async fn get_admin_data(
        req: HttpRequest, 
        admin_id: web::Path<String>, // mongodb object id of admin or god  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> Result<HttpResponse, actix_web::Error> {

    
    if let Some(header_value) = req.headers().get("Authorization"){
    
        let token = header_value.to_str().unwrap();
        
        /*
            @params: 
                - @toke          → JWT
        */
        match passport!{ token }{
            true => {

                // -------------------------------------------------------------------------------------
                // ------------------------------- ACCESS GRANTED REGION -------------------------------
                // -------------------------------------------------------------------------------------

                let storage = storage.as_ref().to_owned();
                let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
                let mongo_db = storage.clone().unwrap();
                let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");


                match storage.clone().unwrap().get_pgdb().await{
                    Some(pg_pool) => {
            
                        let mut all_god_events = vec![];
                        let god_id = admin_id.to_owned();
                        
                        let events = mongo_db.get_mongodb().await.unwrap().database(&db_name).collection::<misc::EventInfo>("events");
                        let mut events_cursor = events.find(doc!{"group_info.god_id": god_id.to_string()}, None).await.unwrap();
                        
                        while let Some(event_info) = events_cursor.try_next().await.unwrap(){
                            all_god_events.push(event_info)
                        }

                        resp!{
                            Vec<EventInfo>, // the data type
                            all_god_events, // response data
                            FETCHED, // response message
                            StatusCode::OK, // status code
                            None::<Cookie<'_>>, // cookie
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
#[get("/get/user/{user_id}/data")]
async fn get_user_data(
        req: HttpRequest, 
        user_id: web::Path<String>, // mongodb object id of user or player  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> Result<HttpResponse, actix_web::Error> {

    
    if let Some(header_value) = req.headers().get("Authorization"){
    
        let token = header_value.to_str().unwrap();
        
        /*
            @params: 
                - @toke          → JWT
        */
        match passport!{ token }{
            true => {

                // -------------------------------------------------------------------------------------
                // ------------------------------- ACCESS GRANTED REGION -------------------------------
                // -------------------------------------------------------------------------------------

                let storage = storage.as_ref().to_owned();
                let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
                let mongo_db = storage.clone().unwrap();
                let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
                
                match storage.clone().unwrap().get_pgdb().await{
                    Some(pg_pool) => {
                        
                    /* creating objectid from 12 bytes or 24 hex char string */
                    let user_id = user_id.to_owned();
                    let user_object_id = ObjectId::parse_str(user_id.as_str()).unwrap();
                    
                    let filter = doc! { "players._id": user_object_id }; // filtering all expired events
                    let events = mongo_db.get_mongodb().await.unwrap().database(&db_name).collection::<misc::EventInfo>("events");
                    let mut all_events = vec![];
                    
                    match events.find(filter, None).await{
                            Ok(mut cursor) => {
                                while let Some(event) = cursor.try_next().await.unwrap(){
                                    all_events.push(event);
                                }
                                let player_events = all_events
                                    .into_iter()
                                    .map(|event| {
                                        misc::PlayerEventInfo{
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


                                resp!{
                                    Vec<PlayerEventInfo>, // the data type
                                    player_events, // response data
                                    FETCHED, // response message
                                    StatusCode::OK, // status code
                                    None::<Cookie<'_>>, // cookie
                                } 

                            },
                            Err(e) => {
                                
                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    &e.to_string(), // response message
                                    StatusCode::INTERNAL_SERVER_ERROR, // status code
                                    None::<Cookie<'_>>, // cookie
                                } 

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