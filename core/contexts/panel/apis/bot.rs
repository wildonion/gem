

use crate::*;
use crate::models::{users::*, tasks::*, users_tasks::*, bot::*};
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
        (status=200, description="Task Verified Successfully", body=[u8]),
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
#[post("/verify-twitter-task/{job_id}/{twitter_username}")]
async fn verify_twitter_task(
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

                    let bot = Bot::new();


                    /*

                        - user login to get the activity code 
                        - user tweet the code then frontend call the verify username api
                        - every 24 hours user gets verified to see that the task is done or not
                            if the task is in there then we'll insert into users_tasks table if it's not in there already 
                            if the task is not inside his/her twitter then we'll remove it from the users_tasks table 
                    
                    */


                    // step1) find the task name related to the passed in id
                    // step2) if it's started with twitter-* then check the name after `-`
                    /* step3) 
                    
                        let res = if name.starts_with("username"){
                            bot.verify_username()
                        } else if name.starts_with("tweet"){
                            bot.verify_tweets()
                        } else if name.starts_with("likes"){
                            bot.verify_likes()
                        } else if name.starts_with("retweets"){
                            bot.verify_retweets()
                        } else if name.starts_with("hashtags"){
                            bot.verify_hashtags()
                        } else{
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                INVALID_TWITTER_TASK_NAME, //// response message
                                StatusCode::NOT_ACCEPTABLE, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }
                        }

                    */


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
    pub use super::verify_twitter_task;
}