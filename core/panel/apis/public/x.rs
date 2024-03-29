


pub use super::*;


/*

    following apis will be used to check that a user task 
    is done or not which sents a request to the twitter bot 
    to check the twitter activities of the passed in username,
    this API must be called in two ways:
        - once the user loggedin to the site to verify his/her tasks 
        - using a cronjob every day at 7 AM to check that all users tasks are verified or not

    also this API doesn't need JWT in header since it'll be called 
    by the /check-users-tasks API which will be called by the crontab   

    so in general when a user logs in to the site
        1 - user must update his/her twitter username by calling the `/verify-twitter-account/{account_name}` 
            API which sends a request to the twitter to verify the passed in username and if the username gets 
            verified by twitter then we'll update the user record.
        2 - an activity code is inside the response data of `/login` API, user must tweet this code using his/her twitter application 
        3 - then an API will be called automatically inside the server every 24 hours that checks that are there any tasks which is done by any user but 
            we can also call that API manually using `/verify-user/{doer_id}/twitter-task/{job_id}` route behind a `check` button or inside an interval http call


*/

#[post("/bot/verify-user/{doer_id}/twitter-task/{job_id}")]
pub(self) async fn verify_twitter_task(
        req: HttpRequest,
        path: web::Path<(i32, i32)>, 
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();

    let doer_id = path.0;
    let job_id = path.1;


    /* rate limiter based on doer id */

    let chill_zone_duration = 30_000u64; // 30 milliaseconds chillzone
    let now = chrono::Local::now().timestamp_millis() as u64;
    let mut is_rate_limited = false;
    
    let redis_result_twitter_rate_limiter: RedisResult<String> = redis_conn.get("twitter_rate_limiter").await;
    let mut redis_twitter_rate_limiter = match redis_result_twitter_rate_limiter{
        Ok(data) => {
            let rl_data = serde_json::from_str::<HashMap<u64, u64>>(data.as_str()).unwrap();
            rl_data
        },
        Err(e) => {
            let empty_twitter_rate_limiter = HashMap::<u64, u64>::new();
            let rl_data = serde_json::to_string(&empty_twitter_rate_limiter).unwrap();
            let _: () = redis_conn.set("twitter_rate_limiter", rl_data).await.unwrap();
            HashMap::new()
        }
    };

    if let Some(last_used) = redis_twitter_rate_limiter.get(&(doer_id as u64)){
        if now - *last_used < chill_zone_duration{
            is_rate_limited = true;
        }
    }
    
    if is_rate_limited{

        resp!{
            &[u8], // the data type
            &[], // response data
            TWITTER_VERIFICATION_RATE_LIMIT, // response message
            StatusCode::NOT_ACCEPTABLE, // status code
            None::<Cookie<'_>>, // cookie
        }
        

    } else{

        /* updating the last rquest time */
        /* also this logic can be used to handle shared state between clusters */
        redis_twitter_rate_limiter.insert(doer_id as u64, now); // updating the redis rate limiter map
        let rl_data = serde_json::to_string(&redis_twitter_rate_limiter).unwrap();
        let _: () = redis_conn.set("twitter_rate_limiter", rl_data).await.unwrap(); // writing to redis ram
        

        match storage.clone().unwrap().get_pgdb().await{
            Some(pg_pool) => {
                
                let connection = &mut pg_pool.get().unwrap();
                
                /* check that we're 24 hours limited or not */
                let rl_data = fetch_x_app_rl_data(redis_client.clone()).await.app_rate_limit_info;
                let check_is_24hours_limited = is_bot_24hours_limited(connection, rl_data).await;
                let Ok(not_limited) = check_is_24hours_limited else{
                    let error_resp = check_is_24hours_limited.unwrap_err();
                    return error_resp;
                };

                /*  
                    if we're here means we're not rate limited also if the user task is inside db the 
                    no need to call bot APIs since it's already done and with this logic we can avoid 
                    rate limit issues and sick out the users that want to play with our apio :)
                */
                match UserTask::find(doer_id, job_id, connection).await{
                    false => {
                        
                        let bot_endpoint = env::var("XBOT_ENDPOINT").expect("⚠️ no twitter bot endpoint key variable set");
                        let new_twitter = Twitter::new(Some(bot_endpoint)).await;
                        let Ok(bot) =  new_twitter else{
                            return new_twitter.unwrap_err();
                        };
                        
                        match Task::find_by_id(job_id.to_owned(), connection).await{
                            Ok(task) => {
                                
                                let mut splitted_name = task.task_name.split("-");
                                let task_starts_with = splitted_name.next().unwrap();
                                let task_type = splitted_name.next().unwrap(); 
                                
                                if task_starts_with.starts_with("twitter"){

                                    // ex: task_type => hashtag::2023,12,08T12,23,00-VLwQb
                                    if task_type.contains("::"){
                                        
                                        // now we have : 2023,12,08T12,23,00-VLwQb
                                        let mut splitted_task_type = task_type.split("::");
                                        let before_double_colon = splitted_task_type.next().unwrap();
                                        let after_double_colon = splitted_task_type.next().unwrap();
                                        let mut splitted_exp_time_rand_char = after_double_colon.split("-");
                                        
                                        // now we have : 2023,12,08T12,23,00
                                        let exp_time = splitted_exp_time_rand_char.next().unwrap();
                                        let exp = exp_time.replace(",", ":");
                                        
                                        let datetime = chrono::NaiveDateTime::parse_from_str(&exp, "%Y:%m:%dT%H:%M:%S").unwrap().timestamp();
                                        let now = chrono::Utc::now().timestamp();
                                        if now > datetime{
                                            
                                            resp!{
                                                &[u8], // the data type
                                                &[], // response data
                                                TASK_EXPIRED, // response message
                                                StatusCode::NOT_ACCEPTABLE, // status code
                                                None::<Cookie<'_>>, // cookie
                                            }
    
                                        } 
                                    }
                                    
                                    match task_type{
                                        "username" | "username::"=> { /* all task names start with username */                                    
                                            bot.verify_username(task, connection, redis_client, doer_id.to_owned()).await
                                        },
                                        "code" | "code::" => { /* all task names start with code */
                                            bot.verify_activity_code(task, connection, redis_client, doer_id.to_owned()).await
                                        },
                                        "tweet" | "tweet::" => { /* all task names start with tweet */
                                            bot.verify_tweet(task, connection, redis_client, doer_id.to_owned()).await
                                        },
                                        "retweet" | "retweet::" => { /* all task names start with retweet */
                                            bot.verify_retweet(task, connection, redis_client, doer_id.to_owned()).await
                                        },
                                        "hashtag" | "hashtag::" => { /* all task names start with hashtag */
                                            bot.verify_hashtag(task, connection, redis_client, doer_id.to_owned()).await
                                        },
                                        "like" | "like::" => { /* all task names start with like */
                                            bot.verify_like(task, connection, redis_client, doer_id.to_owned()).await
                                        },
                                        _ => {
            
                                            resp!{
                                                &[u8], // the data type
                                                &[], // response data
                                                INVALID_TWITTER_TASK_NAME, // response message
                                                StatusCode::NOT_ACCEPTABLE, // status code
                                                None::<Cookie<'_>>, // cookie
                                            }
            
                                        }
                                    }
            
                                } else{
                                    
                                    /* -------------------------------------------- */
                                    /* maybe discord or other social media tasks :) */
                                    /* -------------------------------------------- */
            
                                    resp!{
                                        &[u8], // the data type
                                        &[], // response data
                                        NOT_A_TWITTER_TASK, // response message
                                        StatusCode::NOT_ACCEPTABLE, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }
            
                                }
            
                            },
                            Err(resp) => {
                                
                                /* NOT_FOUND_TASK */
                                resp
                            }
                        }


                    },
                    _ => {

                        /* user task has already been inserted  */
                        let resp = Response::<&[u8]>{
                            data: Some(&[]),
                            message: USER_TASK_HAS_ALREADY_BEEN_INSERTED,
                            status: 302,
                            is_error: false
                        };
                        return Ok(
                            HttpResponse::Found().json(resp)
                        );

                    }
                }
    
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

    
}

/*

    following API will be called by a crontab to check all user taks 
    every 24 hours to find out that they've done the task already or 
    not, since they might have deleted the task from their twitter and
    which in that case the user task will be deleted from the table, 
    this API will call the `/verify-user/{doer_id}/twitter-task/{job_id}`
    API against each user task data fetched from the users_tasks table.
    
*/

#[get("/bot/check-users-tasks")]
pub(self) async fn check_users_task(
        req: HttpRequest,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let panel_port = env::var("PANEL_PORT").unwrap();
            let api = format!("0.0.0.0:{}", panel_port);
            
            let connection = &mut pg_pool.get().unwrap();

            match UserTask::all(connection).await{
                Ok(users_tasks_data) => {

                    if users_tasks_data.len() > 0{

                        let three_days_grace = 259_200_000 as u64; /* 72 hours is 259_200_000 milliseconds */
                        let mut responses = vec![];
                        
                        for user_task in users_tasks_data{

                            let now = chrono::Local::now().timestamp_millis() as u64;
                            let done_when = user_task.done_at.timestamp_millis() as u64;

                            /* if user is done the task like 72 hours ago then there is no need to check it again */
                            if now - done_when >= three_days_grace{
                                continue;
                            }

                            let api_path = format!("{}/bot/verify-user/{}/twitter-task/{}", api, user_task.user_id, user_task.task_id);
                            let client = reqwest::Client::new();
                            let res = client
                                .post(api_path.as_str())
                                .send()
                                .await
                                .unwrap();
                            
                            let r = res.text().await.unwrap();
                            responses.push(r.clone());

                            /* 
                                wait 60 seconds asyncly to avoid twitter rate limit issue
                            */
                            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await; /* sleep asyncly to avoid blocking issues by twitter */
                            
                        }

                        resp!{
                            Vec<String>, // the data type
                            responses, // response data
                            COMPLETE_VERIFICATION_PROCESS, // response message
                            StatusCode::OK, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    } else{

                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            EMPTY_USERS_TASKS, // response message
                            StatusCode::NOT_FOUND, // status code
                            None::<Cookie<'_>>, // cookie
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
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }
}

#[get("/get-x-requests")]
pub(self) async fn get_x_requests(
        req: HttpRequest,   
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();


            let rl_data = fetch_x_app_rl_data(redis_client.clone()).await;

            resp!{
                TotalXRlInfo, // the data type
                rl_data, // response data
                FETCHED, // response message
                StatusCode::OK, // status code
                None::<Cookie<'_>>, // cookie
            }
            
        
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
    pub use super::verify_twitter_task;
    pub use super::check_users_task;
    pub use super::get_x_requests;
}