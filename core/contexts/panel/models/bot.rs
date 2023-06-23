





use crate::*;
use super::{users::User, users_tasks::UserTask, tasks::TaskData};
use crate::constants::*;
use crate::misc::*;
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

    pub fn new(api: Option<String>) -> Result<Self, Result<HttpResponse, actix_web::Error>>{

        let file_open = std::fs::File::open("twitter-accounts.json");
        let Ok(file) = file_open else{

            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: &file_open.unwrap_err().to_string(),
                status: 500
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

    pub async fn is_twitter_user_verified(&self, doer_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<bool, Result<HttpResponse, actix_web::Error>>{

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return Err(res_user_find.unwrap_err());
        };

        let tusername = user.twitter_username.unwrap_or("".to_string());
            
            let get_user_twitter_data = self.get_twitter_user_info(&tusername).await;
            let Ok(twitter_user_data) = get_user_twitter_data else{
                return Err(get_user_twitter_data.unwrap_err());
            };


            for api in self.apis.clone(){
                match api
                    .get_user_followers(twitter_user_data.id)
                    .send()
                    .await
                    {
                        Ok(res) =>{
    
                            match res.into_data(){
                                Some(followers) => {
    
                                    let account_creation_day = twitter_user_data.created_at.unwrap().day();
                                    let now_day = OffsetDateTime::now_utc().day();
    
                                    if now_day - account_creation_day > 7
                                        && followers.len() > 10{
    
                                        return Ok(true);
    
                                    } else{

                                        let resp = Response{
                                            data: Some(tusername),
                                            message: TWITTER_USER_IS_NOT_VALID,
                                            status: 406
                                        };
                                        return Err(
                                            Ok(HttpResponse::NotAcceptable().json(resp))
                                        );
                                    }
                
                                },
                                None => {

                                    let resp = Response{
                                        data: Some(tusername),
                                        message: TWITTER_USER_FOLLOWERS_NOT_FOUND,
                                        status: 404
                                    };
                                    return Err(
                                        Ok(HttpResponse::NotFound().json(resp))
                                    );
    
                                }
                            }
                        },
                        Err(e) => {
    
                            if e.to_string().contains("[429 Too Many Requests]"){
                                continue;
                            } else{

                                let resp = Response{
                                    data: Some(tusername),
                                    message: &e.to_string(),
                                    status: 500
                                };
                                return Err(
                                    Ok(HttpResponse::InternalServerError().json(resp))
                                );
                                

                            }
    
                        }
                    }

            }

            let resp = Response{
                data: Some(tusername),
                message: TWITTER_CANT_LOOP_OVER_ACCOUNTS,
                status: 500
            };
            return Err(
                Ok(HttpResponse::InternalServerError().json(resp))
            );


        }



    /* VERIFY THE GIVEN TWITTER USERNAME  */

    pub async fn verify_username(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient,
        doer_id: i32) -> Result<HttpResponse, actix_web::Error>{

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return res_user_find.unwrap_err();
        };

        /* ------------------------ */
        /* THIRD PARTY TWITTER BOT */
        /* ------------------------ */
        if self.endpoint.is_some(){

            let user_existance_endpoint = format!("{}/user-existance", self.endpoint.as_ref().unwrap());
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

            let tusername = user.twitter_username.unwrap_or("".to_string());
            
            let get_user_twitter_data = self.get_twitter_user_info(&tusername).await;
            let Ok(twitter_user_data) = get_user_twitter_data else{
                return get_user_twitter_data.unwrap_err()
            };


            for api in self.apis.clone(){
                match api
                    .get_user_followers(twitter_user_data.id)
                    .send()
                    .await
                    {
                        Ok(res) =>{
    
                            match res.into_data(){
                                Some(followers) => {
    
                                    let account_creation_day = twitter_user_data.created_at.unwrap().day();
                                    let now_day = OffsetDateTime::now_utc().day();
    
                                    if now_day - account_creation_day > 7
                                        && followers.len() > 10{
    
                                        /* try to insert into users_tasks since it's done */
                                        let res = Twitter::do_task(doer_id, task.id, "username", &tusername.clone(), None, connection).await;


                                        /* publishing the twitter bot response to the redis pubsub channel */
                                        info!("📢 publishing twitter bot response to redis pubsub [twitter-bot-response] channel");
                                        let mut redis_conn = redis_client.get_async_connection().await.unwrap();
                                        let pubsub_message = format!("{TWITTER_VERIFIED_USERNAME:} == {tusername:}");
                                        let _: Result<_, RedisError> = redis_conn.publish::<String, String, String>("twitter-bot-response".to_string(), pubsub_message).await;

                                        return res;
    
                                    } else{
                
                                        resp!{
                                            String, // the data type
                                            tusername, // response data
                                            TWITTER_USER_IS_NOT_VALID, // response message
                                            StatusCode::NOT_ACCEPTABLE, // status code
                                            None::<Cookie<'_>>, // cookie
                                        }
                                    }
                
                                },
                                None => {
    
                                    resp!{
                                        String, // the data type
                                        tusername, // response data
                                        TWITTER_USER_FOLLOWERS_NOT_FOUND, // response message
                                        StatusCode::NOT_FOUND, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }
    
                                }
                            }
                        },
                        Err(e) => {
    
                            if e.to_string().contains("[429 Too Many Requests]"){
                                continue;
                            } else{
                                
                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    &e.to_string(), // response message
                                    StatusCode::INTERNAL_SERVER_ERROR, // status code
                                    None::<Cookie<'_>>, // cookie
                                }

                            }
    
                        }
                    }

            }

            resp!{
                &[u8], // the data type
                &[], // response data
                TWITTER_CANT_LOOP_OVER_ACCOUNTS, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }


        }

    }

    /* VERIFY THAT THE USER HAS TWEETED THE ACTIVITY CODE OR NOT */

    pub async fn verify_activity_code(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> Result<HttpResponse, actix_web::Error>{

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return res_user_find.unwrap_err();
        };
        
        /* ------------------------ */
        /* THIRD PARTY TWITTER BOT  */
        /* ------------------------ */
        if self.endpoint.is_some(){

            let user_existance_endpoint = format!("{}/user-verification", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.clone().twitter_username.unwrap_or("".to_string()));
            map.insert("code", user.activity_code); /* activity code used to check that the user is activated or not */
            
            verify!{
                user_existance_endpoint.as_str(), 
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

            let tusername = user.twitter_username.unwrap_or("".to_string());
            let user_activity_code = user.activity_code;
            
            let get_user_twitter_data = self.get_twitter_user_info(&tusername).await;
            let Ok(twitter_user_data) = get_user_twitter_data else{
                return get_user_twitter_data.unwrap_err()
            };


            let get_user_tweets = self.get_twitter_user_tweets(twitter_user_data.id, tusername.clone()).await;
            let Ok(user_tweets) =  get_user_tweets else{
                return get_user_tweets.unwrap_err();
            };

            let mut is_verified = false;

            for tweet in user_tweets{ /* the scope of user_tweets in here is accessible */
                if tweet.text.contains(&user_activity_code){
                    
                    is_verified = true;
                    
                }
            }

            if is_verified{

                /* try to insert into users_tasks since it's done */
                let res = Twitter::do_task(doer_id, task.id, "username", &tusername.clone(), None, connection).await;

                /* publishing the twitter bot response to the redis pubsub channel */
                info!("📢 publishing twitter bot response to redis pubsub [twitter-bot-response] channel");
                let mut redis_conn = redis_client.get_async_connection().await.unwrap();   
                let pubsub_message = format!("{TWITTER_VERIFIED_CODE:} == {tusername:}");
                let _: Result<_, RedisError> = redis_conn.publish::<String, String, String>("twitter-bot-response".to_string(), pubsub_message).await;
                
                res

                
            } else{
                resp!{
                    String, // the data type
                    tusername, // response data
                    TWITTER_CODE_IS_NOT_VALID, // response message
                    StatusCode::NOT_ACCEPTABLE, // status code
                    None::<Cookie<'_>>, // cookie
                }
            }

        }


    }

    /* VERIFY THAT USER HAS TWEETED AN SPECIFIC TWEET CONTENT OR NOT */

    pub async fn verify_tweet(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> Result<HttpResponse, actix_web::Error>{

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return res_user_find.unwrap_err();
        };
        
        /* ------------------------ */
        /* THIRD PARTY TWITTER BOT  */
        /* ------------------------ */
        if self.endpoint.is_some(){

            let user_existance_endpoint = format!("{}/check", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.clone().twitter_username.unwrap_or("".to_string()));
            map.insert("tweet_id", "".to_string()); /* for like and retweet  */
            map.insert("type", "tweet".to_string()); /* type of verification  */
            map.insert("text", task.tweet_content); /* tweet text to check that the user has tweet the text or not  */
            map.insert("hashtag", task.hashtag); /* hashtag to check that the user tweet contains it or not  */
            
            verify!{
                user_existance_endpoint.as_str(), 
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

            let tusername = user.twitter_username.unwrap_or("".to_string());
            let tweet_content = task.tweet_content;
            
            let get_user_twitter_data = self.get_twitter_user_info(&tusername).await;
            let Ok(twitter_user_data) = get_user_twitter_data else{
                return get_user_twitter_data.unwrap_err()
            };


            let get_user_tweets = self.get_twitter_user_tweets(twitter_user_data.id, tusername.clone()).await;
            let Ok(user_tweets) =  get_user_tweets else{
                return get_user_tweets.unwrap_err();
            };


            let mut is_verified = false;
            let mut link = String::from("");

            for tweet in user_tweets{ /* the scope of user_tweets in here is accessible */
                if tweet.text.contains(&tweet_content) && tweet.text.len() == tweet_content.len(){
                    let tweet_id = tweet.id;
                    link = format!("https://twitter.com/{tusername:}/status/{tweet_id:}");
                    is_verified = true;
                    
                }
            }

            if is_verified{

                /* try to insert into users_tasks since it's done */
                let res = Twitter::do_task(doer_id, task.id, "username", &tusername.clone(), Some(link.as_str()), connection).await;
                        
                /* publishing the twitter bot response to the redis pubsub channel */
                info!("📢 publishing twitter bot response to redis pubsub [twitter-bot-response] channel");
                let mut redis_conn = redis_client.get_async_connection().await.unwrap();   
                let pubsub_message = format!("{TWITTER_VERIFIED_TWEET:} == {tusername:}");
                let _: Result<_, RedisError> = redis_conn.publish::<String, String, String>("twitter-bot-response".to_string(), pubsub_message).await;
                
                res

                
            } else{
                resp!{
                    String, // the data type
                    tusername, // response data
                    TWITTER_NOT_VERIFIED_TWEET_CONTENT, // response message
                    StatusCode::NOT_ACCEPTABLE, // status code
                    None::<Cookie<'_>>, // cookie
                }
            }
        
        }

    }
    
    /* VERIFY THAT USER HAS LIKED AN SPECIFIC TWEET OR NOT */

    pub async fn verify_like(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> Result<HttpResponse, actix_web::Error>{

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return res_user_find.unwrap_err();
        };
        
        /* ------------------------ */
        /* THIRD PARTY TWITTER BOT  */
        /* ------------------------ */
        if self.endpoint.is_some(){

            let user_existance_endpoint = format!("{}/check", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.clone().twitter_username.unwrap_or("".to_string()));
            map.insert("tweet_id", task.like_tweet_id); /* for like and retweet  */
            map.insert("type", "like".to_string()); /* type of verification  */
            map.insert("text", task.tweet_content); /* tweet text to check that the user has tweet the text or not  */
            map.insert("hashtag", task.hashtag); /* hashtag to check that the user tweet contains it or not  */
            
            verify!{
                user_existance_endpoint.as_str(), 
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

            let tusername = user.twitter_username.unwrap_or("".to_string());
            let like_tweet_id = task.like_tweet_id;
            
            let get_user_twitter_data = self.get_twitter_user_info(&tusername).await;
            let Ok(twitter_user_data) = get_user_twitter_data else{
                return get_user_twitter_data.unwrap_err()
            };

            for api in self.apis.clone(){
                match api
                    .get_user_liked_tweets(twitter_user_data.id)
                    .send()
                    .await
                    {
                        Ok(res) => {
    
                            match res.into_data(){
                                Some(tweets) => {
    
                                    let mut is_verified = false;
    
                                    for tweet in tweets{
                                        if tweet.id.to_string() == like_tweet_id{
                                            
                                            is_verified = true;
                                            
                                        }
                                    }
    
                                    if is_verified{
    
                                        /* try to insert into users_tasks since it's done */
                                        let res = Twitter::do_task(doer_id, task.id, "username", &tusername.clone(), None, connection).await;
                                                
                                        /* publishing the twitter bot response to the redis pubsub channel */
                                        info!("📢 publishing twitter bot response to redis pubsub [twitter-bot-response] channel");
                                        let mut redis_conn = redis_client.get_async_connection().await.unwrap();   
                                        let pubsub_message = format!("{TWITTER_VERIFIED_LIKE:} == {tusername:}");
                                        let _: Result<_, RedisError> = redis_conn.publish::<String, String, String>("twitter-bot-response".to_string(), pubsub_message).await;

                                        return res;
    
                                        
                                    } else{
                                        resp!{
                                            String, // the data type
                                            tusername, // response data
                                            TWITTER_NOT_VERIFIED_LIKE, // response message
                                            StatusCode::NOT_ACCEPTABLE, // status code
                                            None::<Cookie<'_>>, // cookie
                                        }
                                    }
    
                                },
                                None => {
    
                                    resp!{
                                        String, // the data type
                                        tusername, // response data
                                        TWITTER_USER_TWEETS_NOT_FOUND, // response message
                                        StatusCode::NOT_FOUND, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }
                                }
                            }
    
                        },
                        Err(e) => {
    
                            if e.to_string().contains("[429 Too Many Requests]"){
                                continue;
                            } else{
                                
                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    &e.to_string(), // response message
                                    StatusCode::INTERNAL_SERVER_ERROR, // status code
                                    None::<Cookie<'_>>, // cookie
                                }

                            }
                        }
                    }

            }

            resp!{
                &[u8], // the data type
                &[], // response data
                TWITTER_CANT_LOOP_OVER_ACCOUNTS, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }

        }

    }
    
    
    /* VERIFY THAT USER HAS RETWEETED AN SPECIFIC TWEET OR NOT */

    pub async fn verify_retweet(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> Result<HttpResponse, actix_web::Error>{

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return res_user_find.unwrap_err();
        };
        
        /* ------------------------ */
        /* THIRD PARTY TWITTER BOT  */
        /* ------------------------ */
        if self.endpoint.is_some(){

            let user_existance_endpoint = format!("{}/check", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.clone().twitter_username.unwrap_or("".to_string()));
            map.insert("tweet_id", task.retweet_id); /* for like and retweet  */
            map.insert("type", "retweet".to_string()); /* type of verification  */
            map.insert("text", task.tweet_content); /* tweet text to check that the user has tweet the text or not  */
            map.insert("hashtag", task.hashtag); /* hashtag to check that the user tweet contains it or not  */
            
            verify!{
                user_existance_endpoint.as_str(), 
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

            let tusername = user.twitter_username.unwrap_or("".to_string());
            let retweet_id = task.retweet_id.parse::<u64>().unwrap();
            
            let get_user_twitter_data = self.get_twitter_user_info(&tusername).await;
            let Ok(twitter_user_data) = get_user_twitter_data else{
                return get_user_twitter_data.unwrap_err()
            };


            let mut is_verified = false;

            
            for api in self.apis.clone(){
                match api
                    .get_tweet(NumericId::new(retweet_id))
                    .tweet_fields([TweetField::Text])
                    .send()
                    .await
                    
                    {
                        Ok(res) => {

                            match res.into_data(){
                                Some(tweet_data) => {

                                    let tweet_text = tweet_data.text;

                                    let get_user_tweets = self.get_twitter_user_tweets(twitter_user_data.id, tusername.clone()).await;
                                    let Ok(user_tweets) =  get_user_tweets else{
                                        return get_user_tweets.unwrap_err();
                                    };

                                    /* if the user tweet contains the specified tweet then the task is verified */
                                    for tweet in user_tweets{ /* the scope of user_tweets in here is accessible */
                                        if tweet.text == tweet_text{
                                            is_verified = true;
                                        }
                                    }


                                    if is_verified{

                                        /* try to insert into users_tasks since it's done */
                                        let res = Twitter::do_task(doer_id, task.id, "username", &tusername.clone(), None, connection).await;

                                        /* publishing the twitter bot response to the redis pubsub channel */
                                        info!("📢 publishing twitter bot response to redis pubsub [twitter-bot-response] channel");
                                        let mut redis_conn = redis_client.get_async_connection().await.unwrap();   
                                        let pubsub_message = format!("{TWITTER_VERIFIED_RETWEET:} == {tusername:}");
                                        let _: Result<_, RedisError> = redis_conn.publish::<String, String, String>("twitter-bot-response".to_string(), pubsub_message).await;


                                        return res;

                                        
                                    } else{
                                        resp!{
                                            String, // the data type
                                            tusername, // response data
                                            TWITTER_NOT_VERIFIED_RETWEET, // response message
                                            StatusCode::NOT_ACCEPTABLE, // status code
                                            None::<Cookie<'_>>, // cookie
                                        }
                                    }

                                },
                                None => {

                                    resp!{
                                        u64, // the data type
                                        retweet_id, // response data
                                        TWITTER_TWEET_NOT_FOUND, // response message
                                        StatusCode::NOT_FOUND, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }

                                }
                            }
                        
                        },
                        Err(e) => {

                            if e.to_string().contains("[429 Too Many Requests]"){
                                continue;
                            } else{
                                
                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    &e.to_string(), // response message
                                    StatusCode::INTERNAL_SERVER_ERROR, // status code
                                    None::<Cookie<'_>>, // cookie
                                }

                            }
                        }
                    }

            }
        
            resp!{
                &[u8], // the data type
                &[], // response data
                TWITTER_CANT_LOOP_OVER_ACCOUNTS, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }

        }
    }


    /* VERIFY THAT USER TWEETS HAVE A SPECIFIC HASHTAGS OR NOT */

    pub async fn verify_hashtag(&self, 
        task: TaskData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient, 
        doer_id: i32) -> Result<HttpResponse, actix_web::Error>{

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return res_user_find.unwrap_err();
        };
        
        /* ------------------------ */
        /* THIRD PARTY TWITTER BOT  */
        /* ------------------------ */
        if self.endpoint.is_some(){

            let user_existance_endpoint = format!("{}/check", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.clone().twitter_username.unwrap_or("".to_string()));
            map.insert("tweet_id", "".to_string()); /* for like and retweet  */
            map.insert("type", "hashtag".to_string()); /* type of verification  */
            map.insert("text", task.tweet_content); /* tweet text to check that the user has tweet the text or not  */
            map.insert("hashtag", task.hashtag); /* hashtag to check that the user tweet contains it or not  */
            
            verify!{
                user_existance_endpoint.as_str(), 
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

            let tusername = user.twitter_username.unwrap_or("".to_string());
            let hastag = task.hashtag;
            
            let get_user_twitter_data = self.get_twitter_user_info(&tusername).await;
            let Ok(twitter_user_data) = get_user_twitter_data else{
                return get_user_twitter_data.unwrap_err()
            };


            let get_user_tweets = self.get_twitter_user_tweets(twitter_user_data.id, tusername.clone()).await;
            let Ok(user_tweets) =  get_user_tweets else{
                return get_user_tweets.unwrap_err();
            };

            let mut is_verified = true;
            for tweet in user_tweets{ /* the scope of user_tweets in here is accessible */

                if tweet.text.contains(&hastag){
                    is_verified = true;
                }
                
            }

            if is_verified{

                /* try to insert into users_tasks since it's done */
                let res = Twitter::do_task(doer_id, task.id, "username", &tusername.clone(), None, connection).await;
                        
                /* publishing the twitter bot response to the redis pubsub channel */
                info!("📢 publishing twitter bot response to redis pubsub [twitter-bot-response] channel");
                let mut redis_conn = redis_client.get_async_connection().await.unwrap();   

                let pubsub_message = format!("{TWITTER_VERIFIED_HASHTAG:} == {tusername:}");
                let _: Result<_, RedisError> = redis_conn.publish::<String, String, String>("twitter-bot-response".to_string(), pubsub_message).await;

                res

                
            } else{
                resp!{
                    String, // the data type
                    tusername, // response data
                    TWITTER_NOT_VERIFIED_HASHTAG, // response message
                    StatusCode::NOT_ACCEPTABLE, // status code
                    None::<Cookie<'_>>, // cookie
                }
            }

        }


    }

    async fn get_twitter_user_info(&self, tusername: &str) -> Result<TwitterUser, Result<HttpResponse, actix_web::Error>>{

        for api in self.apis.clone(){
            match api
                .get_user_by_username(tusername.clone())
                .user_fields([UserField::Id, UserField::Username, UserField::CreatedAt, UserField::Verified, UserField::Entities])
                .send()
                .await
                {
                    Ok(res) => {
    
                        match res.into_data(){
                            Some(user_data) => {
    
                                return Ok(
                                    user_data
                                );
                                
                            },
                            None => {
    
                                let resp = Response{
                                    data: Some(tusername.to_string()),
                                    message: TWITTER_USER_DATA_NOT_FOUND,
                                    status: 404
                                };
                                return Err(
                                    Ok(HttpResponse::NotFound().json(resp))
                                );
    
                            }
                        }   
                    },
                    Err(e) => {
    
                        /* 
                            since the return type is [u8] which is not sized 
                            thus we must put it behind a pointer or return 
                            its slice form which is &[u8] which requires a
                            valid lifetime to be passed in Response struct
                            signature, also the type of the response data 
                            must be specified
                        */

                        if e.to_string().contains("[429 Too Many Requests]"){
                            continue;
                        } else{
                            
                            let resp = Response::<'_, &[u8]>{
                                data: Some(&[]),
                                message: &e.to_string(),
                                status: 500
                            };
                            return Err(
                                Ok(HttpResponse::InternalServerError().json(resp))
                            );
    
                        }
    
                    }
                }
        
        }

        let resp = Response::<'_, &[u8]>{
            data: Some(&[]),
            message: TWITTER_CANT_LOOP_OVER_ACCOUNTS,
            status: 500
        };
        return Err(
            Ok(HttpResponse::InternalServerError().json(resp))
        );

    }

    async fn get_twitter_user_tweets(&self, twitter_user_id: NumericId, user_twitter_username: String) -> Result<Vec<Tweet>, Result<HttpResponse, actix_web::Error>>{

        for api in self.apis.clone(){
            match api
                .get_user_tweets(twitter_user_id)
                .send()
                .await
                {
                    Ok(res) => {
    
                        match res.into_data(){
                            Some(tweets) => {
    
                                return Ok(tweets);
    
                            },
                            None => {
    
                                let resp = Response{
                                    data: Some(user_twitter_username),
                                    message: TWITTER_USER_TWEETS_NOT_FOUND,
                                    status: 404
                                };
                                return Err(
                                    Ok(HttpResponse::NotFound().json(resp))
                                );
                            }
                        }
    
                    },
                    Err(e) => {
    
                        /* 
                            since the return type is [u8] which is not sized 
                            thus we must put it behind a pointer or return 
                            its slice form which is &[u8] which requires a
                            valid lifetime to be passed in Response struct
                            signature, also the type of the response data 
                            must be specified
                        */
    
                        if e.to_string().contains("[429 Too Many Requests]"){
                            continue;
                        } else{
                            
                            let resp = Response::<'_, &[u8]>{
                                data: Some(&[]),
                                message: &e.to_string(),
                                status: 500
                            };
                            return Err(
                                Ok(HttpResponse::InternalServerError().json(resp))
                            );
    
                        }
    
                        
                    }
                }

        }

        let resp = Response::<'_, &[u8]>{
            data: Some(&[]),
            message: TWITTER_CANT_LOOP_OVER_ACCOUNTS,
            status: 500
        };
        return Err(
            Ok(HttpResponse::InternalServerError().json(resp))
        );
            

    }

    pub async fn do_task(
        doer_id: i32, job_id: i32, task_type: &str, tusername: &str, tweet_link: Option<&str>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<HttpResponse, actix_web::Error>{
        
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