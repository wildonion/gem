



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

#[get("/index/{username}")]
pub async fn index(
        req: HttpRequest, 
        username: web::Path<String>, 
        redis_conn: web::Data<RedisConnection>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {
   
    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_conn.to_owned();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {


            // ...


            resp!{
                String, //// the data type
                username.to_owned(), //// response data
                FETCHED, //// response message
                StatusCode::OK, //// status code
            } 


        },
        None => {
            resp!{
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
            }
        }
    }

}