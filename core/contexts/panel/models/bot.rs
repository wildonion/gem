


use crate::*;
use super::{users::User, users_tasks::UserTask, tasks::TaskData};
use crate::constants::*;
use crate::misc::*;
use crate::schema::users_tasks;
use crate::schema::users_tasks::dsl::*;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Twitter{
    pub endpoint: Option<String>,
    bearer_token: String,
    access_token: String, 
    access_token_secret: String,
    consumer_key: String,
    consumer_secret: String,
    client_id: String,
    client_secret: String
}

impl Twitter{

    /* 
        self in other methods is behind a shared reference means that its fields
        can't be moved into other scopes due to the rule of if a type is behind a 
        pointer it can't be moved because when we call unwrap() on a type it takes 
        it's ownership, thus we can clone or borrow its fields using clone() or as_ref()
        methods or using &.   
    */

    pub fn new(api: Option<String>) -> Self{
        
        let bearer_token = env::var("TWITTER_BEARER_TOKEN").unwrap_or("".to_string());
        let access_token = env::var("TWITTER_ACCESS_TOKEN").unwrap_or("".to_string());
        let access_token_secret = env::var("TWITTER_ACCESS_TOKEN_SECRET").unwrap_or("".to_string());
        let consumer_key = env::var("TWITTER_CONSUMER_KEY").unwrap_or("".to_string());
        let consumer_secret = env::var("TWITTER_CONSUMER_SECRET").unwrap_or("".to_string());
        let client_id = env::var("TWITTER_CLIENT_ID").unwrap_or("".to_string());
        let client_secret = env::var("TWITTER_CLIENT_SECRET").unwrap_or("".to_string());
        
        Self{
            endpoint: if api.is_some(){
                api
            } else{
                Some("".to_string()) // we're using conse twitter APIs
            },
            bearer_token,
            access_token,
            access_token_secret,
            consumer_key,
            consumer_secret,
            client_id,
            client_secret,
        }
    }

    /* VERIFY THE GIVEN TWITTER USERNAME  */

    pub async fn verify_username(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient,
        doer_id: i32) -> Result<HttpResponse, actix_web::Error>{

        if self.endpoint.is_some(){

            let res_user_find = User::find_by_id(doer_id, connection).await;
            let Ok(user) = res_user_find else{
                return res_user_find.unwrap_err();
            };


            let user_existance_endpoint = format!("{}/user-existance", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.twitter_username.unwrap_or("".to_string()));
            
            verify!(
                user_existance_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client
            )
                
            
        } else{

            // https://crates.io/crates/twitter-v2
            // 🥑 todo - verify user existance logic using twitter API 
            // ...

            todo!()
        }

    }

    /* VERIFY THAT THE USER HAS TWEETED THE ACTIVITY CODE OR NOT */

    pub async fn verify_activity_code(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> Result<HttpResponse, actix_web::Error>{

        if self.endpoint.is_some(){

            let res_user_find = User::find_by_id(doer_id, connection).await;
            let Ok(user) = res_user_find else{
                return res_user_find.unwrap_err();
            };


            let user_existance_endpoint = format!("{}/user-verification", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.twitter_username.unwrap_or("".to_string()));
            map.insert("code", user.activity_code); /* activity code used to check that the user is activated or not */
            
            verify!(
                user_existance_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client
            )


        } else{

            // https://crates.io/crates/twitter-v2
            // 🥑 todo - verify user activity code logic using twitter API 
            // ...

            todo!()
        }


    }

    /* VERIFY THAT USER HAS TWEETED AN SPECIFIC TWEET CONTENT OR NOT */

    pub async fn verify_tweet(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> Result<HttpResponse, actix_web::Error>{

        if self.endpoint.is_some(){
            
            let res_user_find = User::find_by_id(doer_id, connection).await;
            let Ok(user) = res_user_find else{
                return res_user_find.unwrap_err();
            };


            let user_existance_endpoint = format!("{}/check", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.twitter_username.unwrap_or("".to_string()));
            map.insert("tweet_id", "".to_string()); /* for like and retweet  */
            map.insert("type", "tweet".to_string()); /* type of verification  */
            map.insert("text", task.tweet_content); /* tweet text to check that the user has tweet the text or not  */
            map.insert("hashtag", task.hashtag); /* hashtag to check that the user tweet contains it or not  */
            
            verify!(
                user_existance_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client
            )

        } else{

            // https://crates.io/crates/twitter-v2
            // 🥑 todo - verify user tweet logic using twitter API 
            // ...

            todo!()
        
        }

    }
    
    /* VERIFY THAT USER HAS LIKED AN SPECIFIC TWEET OR NOT */

    pub async fn verify_like(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> Result<HttpResponse, actix_web::Error>{

        if self.endpoint.is_some(){

            let res_user_find = User::find_by_id(doer_id, connection).await;
            let Ok(user) = res_user_find else{
                return res_user_find.unwrap_err();
            };
            
            let user_existance_endpoint = format!("{}/check", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.twitter_username.unwrap_or("".to_string()));
            map.insert("tweet_id", task.like_tweet_id); /* for like and retweet  */
            map.insert("type", "like".to_string()); /* type of verification  */
            map.insert("text", task.tweet_content); /* tweet text to check that the user has tweet the text or not  */
            map.insert("hashtag", task.hashtag); /* hashtag to check that the user tweet contains it or not  */
            
            verify!(
                user_existance_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client
            )

        } else{

            // https://crates.io/crates/twitter-v2
            // 🥑 todo - verify user like logic using twitter API
            // ...
        }


        todo!()

    }
    
    
    /* VERIFY THAT USER HAS RETWEETED AN SPECIFIC TWEET OR NOT */

    pub async fn verify_retweet(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> Result<HttpResponse, actix_web::Error>{

        if self.endpoint.is_some(){

            let res_user_find = User::find_by_id(doer_id, connection).await;
            let Ok(user) = res_user_find else{
                return res_user_find.unwrap_err();
            };


            let user_existance_endpoint = format!("{}/check", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.twitter_username.unwrap_or("".to_string()));
            map.insert("tweet_id", task.retweet_id); /* for like and retweet  */
            map.insert("type", "retweet".to_string()); /* type of verification  */
            map.insert("text", task.tweet_content); /* tweet text to check that the user has tweet the text or not  */
            map.insert("hashtag", task.hashtag); /* hashtag to check that the user tweet contains it or not  */
            
            verify!(
                user_existance_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client
            )

        } else{

            // https://crates.io/crates/twitter-v2
            // 🥑 todo - verify user retweet logic using twitter API

            // ...
        }

        todo!()
    }

    /* VERIFY THAT USER TWEETS HAVE A SPECIFIC HASHTAGS OR NOT */

    pub async fn verify_hashtag(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> Result<HttpResponse, actix_web::Error>{

        if self.endpoint.is_some(){

            let res_user_find = User::find_by_id(doer_id, connection).await;
            let Ok(user) = res_user_find else{
                return res_user_find.unwrap_err();
            };


            let user_existance_endpoint = format!("{}/check", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.twitter_username.unwrap_or("".to_string()));
            map.insert("tweet_id", "".to_string()); /* for like and retweet  */
            map.insert("type", "hashtag".to_string()); /* type of verification  */
            map.insert("text", task.tweet_content); /* tweet text to check that the user has tweet the text or not  */
            map.insert("hashtag", task.hashtag); /* hashtag to check that the user tweet contains it or not  */
            
            verify!(
                user_existance_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client
            )

        } else{

            // https://crates.io/crates/twitter-v2
            // 🥑 todo - verify user hashtag logic using twitter API
            // ...
        }


        todo!()

    }

    pub async fn do_task(
        doer_id: i32, job_id: i32, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<HttpResponse, actix_web::Error>{
        
        match UserTask::insert(doer_id, job_id, connection).await{
            Ok(_) => {
                resp!{
                    &[u8], //// the data type
                    &[], //// response data
                    CREATED, //// response message
                    StatusCode::CREATED, //// status code
                    None::<Cookie<'_>>, //// cookie
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