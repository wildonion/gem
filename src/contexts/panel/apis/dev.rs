



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
                let redis_conn = redis_conn.to_owned();
                let storage = storage.as_ref().to_owned();
                let db = storage.unwrap().get_db().await.unwrap();   

                // 🥑 todo - publish or fire the reveal role topic or event using redis pubsub
                // 🥑 todo - also call the /reveal/roles api of the hyper server
                // 🥑 note - later on client can subs to the fired or 
                //           emitted reveal role event and topics by 
                //           sending websocket connections to the redis 
                //           server docker on the VPS in the meanwhile 
                //           we're sure that the /reveal/roles api has 
                //           called by the dev or the god thus players 
                //           can see the roles without refreshing the page :)                 

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