



use crate::*;
use crate::resp;
use crate::constants::*;



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
#[get("/panel/dev/api/{id}")]
pub async fn index(
        req: HttpRequest, 
        id: web::Path<u8>, 
        redis_conn: web::Data<RedisConnection>, //// redis shared state data 
        storage: web::Data<Surreal<SurrealClient>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {
    
    let id = id.to_owned();
    let data = Dev{id};
    let redis_conn = redis_conn.to_owned();
    let db = storage.to_owned();
    
    resp!{
        data.clone(), //// response data
        FETCHED, //// response message
        StatusCode::OK, //// status code
    }

}
