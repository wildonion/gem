

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
        verify_twitter_task
    ),
    components(
        schemas(
            UserData,
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
    also this API must be called once the user loggedin to 
    the site to verify his/her tasks. 


*/
#[utoipa::path(
    context_path = "/bot",
    responses(
        (status=200, description="The User Task Has Already Been Inserted", body=[u8]),
        (status=417, description="The User Task Has Deleted Before", body=[u8]),
        (status=406, description="Task Couldn't Be Verified Successfully (Maybe User Has Deleted/Twitter Rate Limit Issue), Deleted Relevant User Task", body=[u8]),
        (status=406, description="Not A Twitter Tasks", body=[u8]),
        (status=406, description="Invalid Twitter Task Type", body=[u8]),
        (status=200, description="Task Verified Successfully", body=[u8]),
        (status=406, description="Task Can't Be Verified Successfully", body=[u8]),
        (status=404, description="Task Not Found", body=i32), // not found by id
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
        ("doer_id", description = "user id")
    ),
)]
#[post("/verify-twitter-task/{job_id}/{doer_id}")]
async fn verify_twitter_task(
        req: HttpRequest,
        job_id: web::Path<i32>,
        doer_id: web::Path<i32>, 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match User::passport(req, Some(UserRole::User), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    let wallet = token_data.wallet.unwrap();

                    let bot_endpoint = env::var("THIRD_PARY_TWITTER_BOT_ENDPOINT").expect("⚠️ no twitter bot endpoint key variable set");
                    let bot = Twitter::new(Some(bot_endpoint));

                    /* use this instance if you want to use conse twitter APIs */
                    // let bot = Twitter::new(None);

                    match Task::find_by_id(job_id.to_owned(), connection).await{
                        Ok(task) => {
                            
                            let mut splitted_name = task.task_name.split("-");
                            let task_starts_with = splitted_name.next().unwrap();
                            let task_type = splitted_name.next().unwrap(); 
                            
                            if task_starts_with.starts_with("twitter-"){
                                
                                match task_type{
                                    "username" => {
                                        bot.verify_username(task, connection, doer_id.to_owned()).await
                                    },
                                    "code" => {
                                        bot.verify_activity_code(task, connection, doer_id.to_owned()).await
                                    },
                                    "tweet" => {
                                        bot.verify_tweet(task, connection, doer_id.to_owned()).await
                                    },
                                    "retweet" => {
                                        bot.verify_retweet(task, connection, doer_id.to_owned()).await
                                    },
                                    "hashtag" => {
                                        bot.verify_hashtag(task, connection, doer_id.to_owned()).await
                                    },
                                    "like" => {
                                        bot.verify_like(task, connection, doer_id.to_owned()).await
                                    },
                                    _ => {

                                        resp!{
                                            &[u8], //// the data type
                                            &[], //// response data
                                            INVALID_TWITTER_TASK_NAME, //// response message
                                            StatusCode::NOT_ACCEPTABLE, //// status code
                                            None::<Cookie<'_>>, //// cookie
                                        }

                                    }
                                }

                            } else{

                                /* maybe discord tasks :) */

                                resp!{
                                    &[u8], //// the data type
                                    &[], //// response data
                                    NOT_A_TWITTER_TASK, //// response message
                                    StatusCode::NOT_ACCEPTABLE, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            }

                        },
                        Err(resp) => {
                            
                            /* NOT_FOUND_TASK */
                            resp
                        }
                    }

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