



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
    |          SWAGGER DO
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
    ),
    tags(
        (name = "crate::apis::bot", description = "Tasks Verification Endpoints")
    ),
    info(
        title = "Twitter Bot Tasks Verification APIs"
    ),
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
    this API must be called in two ways:
        - once the user loggedin to the site to verify his/her tasks 
        - using a cronjob every day at 7 AM to check that all users tasks are verified or not

    also this API doesn't need JWT in header since it'll be called 
    by the /check-users-tasks API which will be called by the crontab   


*/
#[utoipa::path(
    context_path = "/bot",
    responses(
        (status=200, description="The User Task Has Already Been Inserted", body=[u8]),
        (status=417, description="The User Task Has Been Deleted Before", body=[u8]),
        (status=406, description="Task Couldn't Be Verified Successfully (Maybe User Has Been Deleted/Twitter Rate Limit Issue), Deleted Relevant User Task", body=[u8]),
        (status=406, description="Not A Twitter Tasks", body=[u8]),
        (status=406, description="Invalid Twitter Task Type", body=[u8]),
        (status=200, description="Task Verified Successfully", body=[u8]),
        (status=406, description="Task Can't Be Verified Successfully", body=[u8]),
        (status=404, description="Task Not Found", body=i32), // not found by id
        (status=403, description="Bot Is Busy", body=[u8]),
        (status=404, description="User Not Found", body=i32), // not found by id
        (status=404, description="No Value Found In Cookie Or JWT In Header", body=[u8]),
        (status=403, description="JWT Not Found In Cookie", body=[u8]),
        (status=406, description="No Time Hash Found In Cookie", body=[u8]),
        (status=406, description="Invalid Cookie Format", body=[u8]),
        (status=403, description="Cookie Has Been Expired", body=[u8]),
        (status=406, description="Invalid Cookie Time Hash", body=[u8]),
        (status=403, description="Access Denied", body=i32),
        (status=406, description="No Expiration Time Found In Cookie", body=[u8]),
        (status=500, description="Storage Issue", body=[u8])
    ),
    params(
        ("job_id" = i32, Path, description = "task id"),
        ("doer_id" = i32, Path, description = "user id")
    ),
    tag = "crate::apis::bot",
)]
#[post("/verify-user/{doer_id}/twitter-task/{job_id}")]
async fn verify_twitter_task(
        req: HttpRequest,
        path: web::Path<(i32, i32)>, 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            let bot_endpoint = env::var("THIRD_PARY_TWITTER_BOT_ENDPOINT").expect("⚠️ no twitter bot endpoint key variable set");
            let bot = Twitter::new(Some(bot_endpoint));

            /* use this instance if you want to use conse twitter APIs */
            // let bot = Twitter::new(None);

            let doer_id = path.0;
            let job_id = path.1;

            match Task::find_by_id(job_id.to_owned(), connection).await{
                Ok(task) => {
                    
                    let mut splitted_name = task.task_name.split("-");
                    let task_starts_with = splitted_name.next().unwrap();
                    let task_type = splitted_name.next().unwrap(); 
                    
                    if task_starts_with.starts_with("twitter"){
                        
                        match task_type{
                            "username" | "username-"=> { /* all task names start with username */
                                bot.verify_username(task, connection, redis_conn, doer_id.to_owned()).await
                            },
                            "code" | "code-" => { /* all task names start with code */
                                bot.verify_activity_code(task, connection, redis_conn, doer_id.to_owned()).await
                            },
                            "tweet" | "tweet-" => { /* all task names start with tweet */
                                bot.verify_tweet(task, connection, redis_conn, doer_id.to_owned()).await
                            },
                            "retweet" | "retweet-" => { /* all task names start with retweet */
                                bot.verify_retweet(task, connection, redis_conn, doer_id.to_owned()).await
                            },
                            "hashtag" | "hashtag-" => { /* all task names start with hashtag */
                                bot.verify_hashtag(task, connection, redis_conn, doer_id.to_owned()).await
                            },
                            "like" | "like-" => { /* all task names start with like */
                                bot.verify_like(task, connection, redis_conn, doer_id.to_owned()).await
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

#[get("/check-users-tasks")]
async fn check_users_tassk(
        req: HttpRequest,
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let panel_port = env::var("PANEL_PORT").unwrap();
            let api = format!("0.0.0.0:{}", panel_port);
            
            let connection = &mut pg_pool.get().unwrap();

            match UserTask::all(connection).await{
                Ok(users_tasks_data) => {

                    if users_tasks_data.len() > 0{

                        let mut responses = vec![];
                        
                        for user_task in users_tasks_data{
                    
                            let api_path = format!("{}/bot/verify-user/{}/twitter-task/{}", api, user_task.user_id, user_task.task_id);
                            let client = reqwest::Client::new();
                            let res = client
                                .post(api_path.as_str())
                                .send()
                                .await
                                .unwrap();
                            
                            let r = res.text().await.unwrap();
                            responses.push(r.clone());

                            /* publishing the task verification response topic to the redis pubsub channel */
                            info!("📢 publishing task verification response to redis pubsub [task-verification-responses] channel");

                            let mut conn = redis_client.get_connection().unwrap();
                            let _: Result<_, RedisError> = conn.publish::<String, String, String>("task-verification-responses".to_string(), r.clone());

                            /* wait 15 seconds asyncly to avoid twitter rate limit issue */
                            tokio::time::sleep(tokio::time::Duration::from_secs(15)).await; /* sleep asyncly to avoid blocking issues */
                            
                        }

                        resp!{
                            Vec<String>, //// the data type
                            responses, //// response data
                            COMPLETE_VERIFICATION_PROCESS, //// response message
                            StatusCode::OK, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            EMPTY_USERS_TASKS, //// response message
                            StatusCode::NOT_FOUND, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    }

                },
                Err(resp) => {
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
    pub use super::check_users_tassk;
}
