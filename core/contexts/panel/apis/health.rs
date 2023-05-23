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
pub struct Health{
    pub status: String,
}


/*
     ------------------------
    |          APIS
    | ------------------------
    |
    |

*/
#[get("/reveal-role")]
async fn index(
        req: HttpRequest, 
        id: web::Path<u8>, 
        redis_conn: web::Data<RedisConnection>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

        let iam_healthy = Health{
            status: "Alive".to_string()
        };
    
        resp!{
            Health, //// the data type
            iam_healthy, //// response data
            IAM_HEALTHY, //// response message
            StatusCode::OK, //// status code
        }

}


pub mod exports{
    pub use super::index;
}