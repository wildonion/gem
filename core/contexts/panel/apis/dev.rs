



use std::time::Duration;

use serenity::model::prelude::Connection;

use crate::*;
use crate::resp;
use crate::passport;
use crate::constants::*;
use crate::misc::*;


/*
     ------------------------
    |        SCHEMAS
    | ------------------------
    |
    |

*/
#[derive(Serialize, Deserialize, Clone)]
pub struct Dev{
    pub id: u8,
}


/*
     ------------------------
    |          APIS
    | ------------------------
    |
    |

*/
#[get("/reveal-role")]
pub async fn index(
        req: HttpRequest, 
        id: web::Path<u8>, 
        redis_conn: web::Data<RedisConnection>, //// redis shared state data 
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

                let id = id.to_owned();
                let data = Dev{id};
                let storage = storage.as_ref().to_owned();
                
                let redis_conn = redis_conn.to_owned();
                let mongo_db = storage.clone().unwrap().get_mongodb().await.unwrap();   
                let pg_pool = storage.unwrap().get_pgdb().await.unwrap();   

                // 🥑 todo - publish or fire the reveal role topic or event using redis pubsub
                // 🥑 todo - also call the /reveal/roles api of the hyper server                 
                // ...


                resp!{
                    Dev, //// the data type
                    data.clone(), //// response data
                    FETCHED, //// response message
                    StatusCode::OK, //// status code
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
                }
            }
        }

    } else{
        
        resp!{
            &[u8], //// the date type
            &[], //// the data itself
            NOT_AUTH_HEADER, //// response message
            StatusCode::FORBIDDEN, //// status code
        }
    }

}