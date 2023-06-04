




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
    security(
        ("jwt" = [])
    )
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
)]
#[get("/get/admin/{admin_id}/data")]
async fn get_admin_data(
        req: HttpRequest, 
        admin_id: web::Path<String>, //// mongodb object id of admin or god  
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    
    if let Some(header_value) = req.headers().get("Authorization"){
    
        let token = header_value.to_str().unwrap();
        
        /*
            @params: 
                - @request       → actix request object
                - @storage       → instance inside the request object
                - @access levels → vector of access levels
        */
        match passport!{ token }{
            true => {

                //// -------------------------------------------------------------------------------------
                //// ------------------------------- ACCESS GRANTED REGION -------------------------------
                //// -------------------------------------------------------------------------------------

                let storage = storage.as_ref().to_owned();
                let redis_conn = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
                let mongo_db = storage.clone().unwrap().get_mongodb().await.unwrap();

                match storage.clone().unwrap().get_pgdb().await{
                    Some(pg_pool) => {
            
                        
                        // 🥑 todo - fetch all events related to the passed in admin (god) id from mongodb
                        // 🥑 todo - fetch all events related to the passed in admin (god) id using hyper api calls
                        // ...
            
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            FETCHED, //// response message
                            StatusCode::OK, //// status code
                            None::<Cookie<'_>>, //// cookie
                        } 
            
            
                    },
                    None => {
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            STORAGE_ISSUE, //// response message
                            StatusCode::INTERNAL_SERVER_ERROR, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }
                }

                //// -------------------------------------------------------------------------------------
                //// -------------------------------------------------------------------------------------
                //// -------------------------------------------------------------------------------------

            },
            false => {
                
                resp!{
                    &[u8], //// the date type
                    &[], //// the data itself
                    INVALID_TOKEN, //// response message
                    StatusCode::FORBIDDEN, //// status code
                    None::<Cookie<'_>>, //// cookie
                }
            }
        }

    } else{
        
        resp!{
            &[u8], //// the date type
            &[], //// the data itself
            NOT_AUTH_HEADER, //// response message
            StatusCode::FORBIDDEN, //// status code
            None::<Cookie<'_>>, //// cookie
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
)]
#[get("/get/user/{user_id}/data")]
async fn get_user_data(
        req: HttpRequest, 
        user_id: web::Path<String>, //// mongodb object id of user or player  
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    
    if let Some(header_value) = req.headers().get("Authorization"){
    
        let token = header_value.to_str().unwrap();
        
        /*
            @params: 
                - @request       → actix request object
                - @storage       → instance inside the request object
                - @access levels → vector of access levels
        */
        match passport!{ token }{
            true => {

                //// -------------------------------------------------------------------------------------
                //// ------------------------------- ACCESS GRANTED REGION -------------------------------
                //// -------------------------------------------------------------------------------------

                let storage = storage.as_ref().to_owned();
                let redis_conn = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
                let mongo_db = storage.clone().unwrap().get_mongodb().await.unwrap();

                match storage.clone().unwrap().get_pgdb().await{
                    Some(pg_pool) => {
            
                        
                        // 🥑 todo - fetch all events related to the passed in user (player) id from mongodb
                        // 🥑 todo - fetch all events related to the passed in user (player) id using hyper api calls
                        // ...
            
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            FETCHED, //// response message
                            StatusCode::OK, //// status code
                            None::<Cookie<'_>>, //// cookie
                        } 
            
            
                    },
                    None => {
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            STORAGE_ISSUE, //// response message
                            StatusCode::INTERNAL_SERVER_ERROR, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }
                }

                //// -------------------------------------------------------------------------------------
                //// -------------------------------------------------------------------------------------
                //// -------------------------------------------------------------------------------------

            },
            false => {
                
                resp!{
                    &[u8], //// the date type
                    &[], //// the data itself
                    INVALID_TOKEN, //// response message
                    StatusCode::FORBIDDEN, //// status code
                    None::<Cookie<'_>>, //// cookie
                }
            }
        }

    } else{
        
        resp!{
            &[u8], //// the date type
            &[], //// the data itself
            NOT_AUTH_HEADER, //// response message
            StatusCode::FORBIDDEN, //// status code
            None::<Cookie<'_>>, //// cookie
        }
    }

}



pub mod exports{
    pub use super::get_admin_data;
    pub use super::get_user_data;
}