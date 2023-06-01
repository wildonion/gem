

use crate::*;
use crate::models::{users::*, tasks::*, users_tasks::*};
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;



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
        
    ),
    components(
        schemas(
            UserData,
            FetchUserTaskReport,
            TaskData
        )
    )
)]
pub struct BotApiDoc;

/*
     ------------------------
    |          APIS
    | ------------------------
    |
    |


    following apis will be used to check that a user task 
    is done or not which sents a request to the twitter bot 
    to check the twitter activities of the passed in username,
    also there must be an scheduler inside the code to call 
    this API every like 24 hours constantly since the tweet 
    by the user may be deleted in the pas hours and thus the 
    shouldn't gets scored. 

    twitter tasks can be the followings:
        - tweet
        - retweet
        - hashtags
        - likes 

*/
#[utoipa::path(
    context_path = "/bot",
    responses(
        (status=200, description="Task Verified Successfully", body=UserData),
        (status=403, description="Bot Is Busy", body=[u8]),
        (status=404, description="User Not Found", body=i32), // not found by id
        (status=404, description="User Not Found", body=String), // not found by wallet
        (status=404, description="No Value Found In Cookie", body=[u8]),
        (status=403, description="JWT Not Found In Cookie", body=[u8]),
        (status=406, description="No Time Hash Found In Cookie", body=[u8]),
        (status=406, description="Invalid Cookie Format", body=[u8]),
        (status=403, description="Cookie Has Expired", body=[u8]),
        (status=406, description="Invalid Cookie Time Hash", body=[u8]),
        (status=403, description="Access Denied", body=i32),
        (status=406, description="No Expiration Time Found In Cookie", body=[u8]),
        (status=500, description="Storage Issue", body=[u8])
    ),
    params(
        ("job_id", description = "task id"),
        ("twitter_username", description = "twitter username")
    ),
)]
#[post("/verify-task/{job_id}/{twitter_username}")]
async fn verify_task(
        req: HttpRequest,
        account_name: web::Path<String>, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match User::passport(req, Some(UserRole::User), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    let wallet = token_data.wallet.unwrap();

                    
                    // 🥑 todo - call conse twitter apis to verify a user activities to check the task as done
                    // 🥑 todo - call the other one twitter bot apis here
                    // 🥑 todo - call /do-task/{task_id}/{user_id} api to done a user task after successful verification
                    // ... 








                    todo!()

                },
                Err(resp) => {
                    
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
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
}


pub mod exports{
    pub use super::verify_task;
}