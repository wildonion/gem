



use crate::*;
use crate::resp;
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
#[get("/panel/dev/api/reveal-role")]
pub async fn index(
        req: HttpRequest, 
        id: web::Path<u8>, 
        redis_conn: web::Data<RedisConnection>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {
    
    let id = id.to_owned();
    let data = Dev{id};
    let redis_conn = redis_conn.to_owned();
    let storage = storage.as_ref().to_owned();
    let db = storage.unwrap().get_db().await.unwrap();

    
    // https://redis.com/blog/how-to-create-notification-services-with-redis-websockets-and-vue-js/
    // 🥑 todo - check the header jwt token with hyper server /check-token api
    // 🥑 todo - if the access level was dev then: 
    // 🥑 todo - publish or fire the reveal role topic or event using redis pubsub
    // 🥑 todo - also call the /reveal/roles api of the hyper server
    // 🥑 note - later on client can subs to the fired or 
    //           emitted reveal role event and topics by 
    //           sending websocket connections to the redis 
    //           server docker on the VPS in the meanwhile 
    //           we're sure that the /reveal/roles api has 
    //           called by the dev or the god thus players 
    //           can see the roles without refreshing the page :)



    // return traits using Box, impl and dyn 
    // bound generic to traits and lifetimes in function and struct signature 
    // struct and function param as trait
    // ...


    
    resp!{
        data.clone(), //// response data
        FETCHED, //// response message
        StatusCode::OK, //// status code
    }

}