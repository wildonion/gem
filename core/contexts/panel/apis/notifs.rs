




use crate::*;
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::events::ws::notifs::Notif;




#[get("/")] /* client must be connect to this route then we have a full duplex communication channel */
async fn reveal_role(
    req: HttpRequest, 
    stream: web::Payload, 
    storage: web::Data<Option<Arc<Storage>>> // db shared state data
) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            // todo -
            /*
                `reveal-role-{event_id}`
                `twitter-bot-response`, 
                `ecq-{event_id}`, 
                `mmr-{event_id}`, 
                `reveal-role-{event_id}`
            */
            let notif = Notif::default();
            let resp = ws::start(notif, &req, stream);
            resp

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
}





pub mod exports{
    pub use super::reveal_role;
}