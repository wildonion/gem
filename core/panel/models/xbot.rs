





use crate::*;
use super::{users::User, users_tasks::UserTask, tasks::TaskData};
use crate::constants::*;
use crate::misc::*;
use s3::*;
use crate::schema::users_tasks;
use crate::schema::users_tasks::dsl::*;



#[derive(Debug)]
pub struct Twitter{
    pub endpoint: Option<String>,
    pub keys: Vec<Keys>,
    pub apis: Vec<TwitterApi<BearerToken>>
}

impl Twitter{


    /* 
        self in other methods is behind a shared reference means that its fields
        can't be moved into other scopes due to the rule of if a type is behind a 
        pointer it can't be moved because when we call unwrap() on a type it takes 
        methods or using &.   
        it's ownership, thus we can clone or borrow its fields using clone() or as_ref()
    */

    pub async fn new(api: Option<String>) -> Result<Self, PanelHttpResponse>{

        let file_open = std::fs::File::open("twitter-accounts.json");
        let Ok(file) = file_open else{

            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: &file_open.unwrap_err().to_string(),
                status: 500,
                is_error: true
            };
            return 
                Err(
                    Ok(HttpResponse::InternalServerError().json(resp))
                );  

        };

        let accounts_value: serde_json::Value = serde_json::from_reader(file).unwrap();
        let accounts_json_string = serde_json::to_string(&accounts_value).unwrap(); // reader in serde_json::from_reader can be a tokio tcp stream, a file or a buffer that contains the u8 bytes
        let twitter = serde_json::from_str::<misc::TwitterAccounts>(&accounts_json_string).unwrap(); 
        let twitter_accounts = twitter.keys;
        
        let mut apis = vec![];
        for account in twitter_accounts.clone(){
            let auth = BearerToken::new(account.twitter_bearer_token.clone());
            let twitter_api = TwitterApi::new(auth);
            apis.push(twitter_api);

        }

        Ok(
            Self{
                endpoint: if api.is_some(){
                    api
                } else{
                    None // we're using conse twitter APIs
                },
                apis,
                keys: twitter_accounts
            }
        )
    }

    pub async fn verify_user_with_xbot(&self, account_name: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<bool, Result<HttpResponse, actix_web::Error>>{

        let key = env::var("XBOT_KEY").unwrap();
        let user_existance_endpoint = format!("{}/user-existance/{}", self.endpoint.as_ref().unwrap(), key);
        let mut map = HashMap::new();
        map.insert("username", account_name);

        let response_value: serde_json::Value = reqwest::Client::new()
            .post(user_existance_endpoint)
            .json(&map)
            .send()
            .await.unwrap()
            .json()
            .await.unwrap();

        /* I believe that the bot code has some shity response structure :) since I didn't designed it*/

        let data_field = response_value.get("data");
        if data_field.is_some(){
            let status = data_field.unwrap().get("status");
            if status.is_some(){

                let bool_status = status.unwrap().to_string();
                if bool_status == "false"{

                    /* twitter error */

                    let resp = Response::<String>{
                        data: Some(account_name.to_string()),
                        message: TWITTER_USER_IS_NOT_VALID,
                        status: 406,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 

                } else{

                    Ok(true)

                }
            } else{

                /* twitter rate limit issue */

                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: TWITTER_RATE_LIMIT,
                    status: 406,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                );  
            
            }
        } else{

            /* twitter rate limit issue */

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: TWITTER_RATE_LIMIT,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );  
        }

    }

    /* VERIFY THE GIVEN TWITTER USERNAME  */

    pub async fn verify_username(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient,
        doer_id: i32) -> PanelHttpResponse{

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return res_user_find.unwrap_err();
        };

        /* ------------------------ */
        /* THIRD PARTY TWITTER BOT */
        /* ------------------------ */
        if self.endpoint.is_some(){

            let key = env::var("XBOT_KEY").unwrap();
            let user_existance_endpoint = format!("{}/user-existance/{}", self.endpoint.as_ref().unwrap(), key);
            let mut map = HashMap::new();
            map.insert("username", user.clone().twitter_username.unwrap_or("".to_string()));
            
            verify!{
                user_existance_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client,
                &user.twitter_username.unwrap_or("".to_string()),
                "username", /* task type */
                None
            }
                
            
        } else{

            resp!{
                &[u8], // the data type
                &[], // response data
                TWITTER_INVALID_BOT_ENDPOINT, // response message
                StatusCode::NOT_ACCEPTABLE, // status code
                None::<Cookie<'_>>, // cookie
            }

        }

    }

    /* VERIFY THAT THE USER HAS TWEETED THE ACTIVITY CODE OR NOT */

    pub async fn verify_activity_code(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> PanelHttpResponse{

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return res_user_find.unwrap_err();
        };
        
        /* ------------------------ */
        /* THIRD PARTY TWITTER BOT  */
        /* ------------------------ */
        if self.endpoint.is_some(){

            let key = env::var("XBOT_KEY").unwrap();
            let verify_endpoint = format!("{}/user-existance/{}", self.endpoint.as_ref().unwrap(), key);
            let mut map = HashMap::new();
            map.insert("username", user.clone().twitter_username.unwrap_or("".to_string()));
            map.insert("code", user.activity_code); /* activity code used to check that the user is activated or not */
            
            verify!{
                verify_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client,
                &user.twitter_username.unwrap_or("".to_string()),
                "code", /* task type */
                None
            }


        } else{

            resp!{
                &[u8], // the data type
                &[], // response data
                TWITTER_INVALID_BOT_ENDPOINT, // response message
                StatusCode::NOT_ACCEPTABLE, // status code
                None::<Cookie<'_>>, // cookie
            }

        }


    }

    /* VERIFY THAT USER HAS TWEETED AN SPECIFIC TWEET CONTENT OR NOT */

    pub async fn verify_tweet(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> PanelHttpResponse{

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return res_user_find.unwrap_err();
        };
        
        /* ------------------------ */
        /* THIRD PARTY TWITTER BOT  */
        /* ------------------------ */
        if self.endpoint.is_some(){

            let key = env::var("XBOT_KEY").unwrap();
            let tweet_endpoint = format!("{}/check/{}", self.endpoint.as_ref().unwrap(), key);
            let mut map = HashMap::new();
            map.insert("username", user.clone().twitter_username.unwrap_or("".to_string()));
            map.insert("tweet_id", "".to_string()); /* for like and retweet  */
            map.insert("type", "tweet".to_string()); /* type of verification  */
            map.insert("text", task.tweet_content); /* tweet text to check that the user has tweet the text or not  */
            map.insert("hashtag", task.hashtag); /* hashtag to check that the user tweet contains it or not  */
            
            verify!{
                tweet_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client,
                &user.twitter_username.unwrap_or("".to_string()),
                "tweet", /* task type */
                None
            }

        } else{

            resp!{
                &[u8], // the data type
                &[], // response data
                TWITTER_INVALID_BOT_ENDPOINT, // response message
                StatusCode::NOT_ACCEPTABLE, // status code
                None::<Cookie<'_>>, // cookie
            }
        
        }

    }
    
    /* VERIFY THAT USER HAS LIKED AN SPECIFIC TWEET OR NOT */

    pub async fn verify_like(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> PanelHttpResponse{

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return res_user_find.unwrap_err();
        };
        
        /* ------------------------ */
        /* THIRD PARTY TWITTER BOT  */
        /* ------------------------ */
        if self.endpoint.is_some(){

            let key = env::var("XBOT_KEY").unwrap();
            let like_endpoint = format!("{}/check/{}", self.endpoint.as_ref().unwrap(), key);
            let mut map = HashMap::new();
            map.insert("username", user.clone().twitter_username.unwrap_or("".to_string()));
            map.insert("tweet_id", task.like_tweet_id); /* for like and retweet  */
            map.insert("type", "like".to_string()); /* type of verification  */
            map.insert("text", task.tweet_content); /* tweet text to check that the user has tweet the text or not  */
            map.insert("hashtag", task.hashtag); /* hashtag to check that the user tweet contains it or not  */
            
            verify!{
                like_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client,
                &user.twitter_username.unwrap_or("".to_string()),
                "like", /* task type */
                None
            }

        } else{

            resp!{
                &[u8], // the data type
                &[], // response data
                TWITTER_INVALID_BOT_ENDPOINT, // response message
                StatusCode::NOT_ACCEPTABLE, // status code
                None::<Cookie<'_>>, // cookie
            }

        }

    }
    
    /* VERIFY THAT USER HAS RETWEETED AN SPECIFIC TWEET OR NOT */

    pub async fn verify_retweet(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> PanelHttpResponse{

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return res_user_find.unwrap_err();
        };
        
        /* ------------------------ */
        /* THIRD PARTY TWITTER BOT  */
        /* ------------------------ */
        if self.endpoint.is_some(){

            let key = env::var("XBOT_KEY").unwrap();
            let retweet_endpoint = format!("{}/check/{}", self.endpoint.as_ref().unwrap(), key);
            let mut map = HashMap::new();
            map.insert("username", user.clone().twitter_username.unwrap_or("".to_string()));
            map.insert("tweet_id", task.retweet_id); /* for like and retweet  */
            map.insert("type", "retweet".to_string()); /* type of verification  */
            map.insert("text", task.tweet_content); /* tweet text to check that the user has tweet the text or not  */
            map.insert("hashtag", task.hashtag); /* hashtag to check that the user tweet contains it or not  */
            
            verify!{
                retweet_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client,
                &user.twitter_username.unwrap_or("".to_string()),
                "retweet", /* task type */
                None
            }

        } else{

            resp!{
                &[u8], // the data type
                &[], // response data
                TWITTER_INVALID_BOT_ENDPOINT, // response message
                StatusCode::NOT_ACCEPTABLE, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }


    /* VERIFY THAT USER TWEETS HAVE A SPECIFIC HASHTAGS OR NOT */

    pub async fn verify_hashtag(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> PanelHttpResponse{

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return res_user_find.unwrap_err();
        };
        
        /* ------------------------ */
        /* THIRD PARTY TWITTER BOT  */
        /* ------------------------ */
        if self.endpoint.is_some(){

            let key = env::var("XBOT_KEY").unwrap();
            let hashtag_endpoint = format!("{}/check/{}", self.endpoint.as_ref().unwrap(), key);
            let mut map = HashMap::new();
            map.insert("username", user.clone().twitter_username.unwrap_or("".to_string()));
            map.insert("tweet_id", "".to_string()); /* for like and retweet  */
            map.insert("type", "hashtag".to_string()); /* type of verification  */
            map.insert("text", task.tweet_content); /* tweet text to check that the user has tweet the text or not  */
            map.insert("hashtag", task.hashtag); /* hashtag to check that the user tweet contains it or not  */
            
            verify!{
                hashtag_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client,
                &user.twitter_username.unwrap_or("".to_string()),
                "hashtag", /* task type */
                None
            }

        } else{

            resp!{
                &[u8], // the data type
                &[], // response data
                TWITTER_INVALID_BOT_ENDPOINT, // response message
                StatusCode::NOT_ACCEPTABLE, // status code
                None::<Cookie<'_>>, // cookie
            }

        }


    }

    pub async fn do_task(
        doer_id: i32, job_id: i32, task_type: &str, tusername: &str, tweet_link: Option<&str>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> PanelHttpResponse{
        
        match UserTask::insert(doer_id, job_id, connection).await{
            Ok(_) => {

                match task_type{
                    "username" => {
                        
                        let resp_content = format!("{}, {task_type:} Task Is Done By {tusername:}", TWITTER_VERIFIED_USERNAME);
                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            &resp_content, // response message
                            StatusCode::CREATED, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    },
                    "code" => {

                        let resp_content = format!("{}, {task_type:} Task Is Done By {tusername:}", TWITTER_VERIFIED_CODE);
                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            &resp_content, // response message
                            StatusCode::CREATED, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    },
                    "hashtag" => {
                        
                        let resp_content = format!("{}, {task_type:} Task Is Done By {tusername:}", TWITTER_VERIFIED_HASHTAG);
                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            &resp_content, // response message
                            StatusCode::CREATED, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    },
                    "like" => {

                        let resp_content = format!("{}, {task_type:} Task Is Done By {tusername:}", TWITTER_VERIFIED_LIKE);
                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            &resp_content, // response message
                            StatusCode::CREATED, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    },
                    "tweet" => {

                        let link = tweet_link.unwrap_or("-"); /* the or part means that we're using the third party bot */
                        let resp_content = format!("{}, {task_type:} Task Is Done By {tusername:} With The Link: {link:}", TWITTER_VERIFIED_TWEET);
                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            &resp_content, // response message
                            StatusCode::CREATED, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    },
                    _ => { // retweet

                        let resp_content = format!("{}, {task_type:} Task Is Done By {tusername:}", TWITTER_VERIFIED_RETWEET);
                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            &resp_content, // response message
                            StatusCode::CREATED, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    }
                }

                
            },
            Err(resp) => {

                /* 
                    🥝 response can be one of the following:
                    
                    - DIESEL INSERT ERROR RESPONSE
                    - TASK_NOT_FOUND
                */
                resp
            }
        }

    }

}

pub async fn catch_x_log(redis_client: redis::Client){

    /* ---------------------------------------------------------------------------------------------------------------- */
    /* --------------------------- logging number of requests sent to twitter every 15 mins --------------------------- */
    /* ---------------------------------------------------------------------------------------------------------------- */
    /* 
        
        HashMap<next_15mins_interval, requests_so_far>
        
        let redis_x_15mins_interval = {1695920136: 200, 1695920158: 400, ......., 1695920167: 890};
        
        the key is the next interval and the value is the total requests sent 
        so far to the X before reaching the next 15 mins interval.
    */
    let x_15mins_interval = 900_000u64;
    let now = chrono::Local::now().timestamp_millis() as u64;
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();

    let redis_result_x_15mins_interval_request: RedisResult<String> = redis_conn.get("x_15mins_interval_request").await;
    let mut redis_x_15mins_interval = match redis_result_x_15mins_interval_request{
        Ok(data) => serde_json::from_str::<HashMap<u64, u64>>(data.as_str()).unwrap(),
        Err(e) => HashMap::new()
    };

    /* updating twitter 15 mins interval rate limit requests  */
    if redis_x_15mins_interval.is_empty(){
        let next_interval = now + x_15mins_interval;
        /* adding new interval requests then update it */
        redis_x_15mins_interval.insert(next_interval, 0);
        redis_x_15mins_interval
            .entry(next_interval)
            .and_modify(|reqs| { *reqs+=2 } )
            .or_insert(2);
    } else{
        let last_interval = redis_x_15mins_interval.keys().max().unwrap_or(&now);
        if now - last_interval > x_15mins_interval{
            /* we have to go for the next interval */
            let next_interval = last_interval + x_15mins_interval;
            redis_x_15mins_interval.insert(last_interval + x_15mins_interval, 0);
            /* updating new interval requests */
            redis_x_15mins_interval
                .entry(next_interval)
                .and_modify(|reqs| { *reqs+=2 } )
                .or_insert(2);
        } else{
            /* updating the old interval requests */
            redis_x_15mins_interval
            .entry(*last_interval)
            .and_modify(|reqs| { *reqs+=2 } )
            .or_insert(2);
        }
    }

    let rl_data = serde_json::to_string(&redis_x_15mins_interval).unwrap();
    let _: () = redis_conn.set("x_15mins_interval_request", rl_data).await.unwrap();
    /* ----------------------------------------------------------------------------------------------------------- */

}