




use crate::*;
use super::{users::User, users_tasks::UserTask, tasks::TaskData};
use crate::constants::*;
use crate::misc::*;
use crate::schema::users_tasks;
use crate::schema::users_tasks::dsl::*;



pub struct Twitter{
    pub endpoint: Option<String>,
    pub twitter_api: TwitterApi<BearerToken>,
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
        methods or using &.   
        it's ownership, thus we can clone or borrow its fields using clone() or as_ref()
    */

    pub fn new(api: Option<String>) -> Self{
        
        let bearer_token = env::var("TWITTER_BEARER_TOKEN").unwrap_or("".to_string());
        let access_token = env::var("TWITTER_ACCESS_TOKEN").unwrap_or("".to_string());
        let access_token_secret = env::var("TWITTER_ACCESS_TOKEN_SECRET").unwrap_or("".to_string());
        let consumer_key = env::var("TWITTER_CONSUMER_KEY").unwrap_or("".to_string());
        let consumer_secret = env::var("TWITTER_CONSUMER_SECRET").unwrap_or("".to_string());
        let client_id = env::var("TWITTER_CLIENT_ID").unwrap_or("".to_string());
        let client_secret = env::var("TWITTER_CLIENT_SECRET").unwrap_or("".to_string());
        
        let auth = BearerToken::new(bearer_token.clone());
        let twitter_api = TwitterApi::new(auth);

        Self{
            endpoint: if api.is_some(){
                api
            } else{
                None // we're using conse twitter APIs
            },
            twitter_api,
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

        let res_user_find = User::find_by_id(doer_id, connection).await;
        let Ok(user) = res_user_find else{
            return res_user_find.unwrap_err();
        };

        if self.endpoint.is_some(){

            let user_existance_endpoint = format!("{}/user-existance", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.twitter_username.unwrap_or("".to_string()));
            
            verify!{
                user_existance_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client
            }
                
            
        } else{

            let tusername = user.twitter_username.unwrap_or("".to_string());
            
            let get_user_twitter_data = self.get_twitter_user_info(&tusername).await;
            let Ok(twitter_user_data) = get_user_twitter_data else{
                return get_user_twitter_data.unwrap_err()
            };

            match self.twitter_api
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

                                    match UserTask::find(doer_id, task.id, connection).await{
                                        false => {
            
                                            /* try to insert into users_tasks since it's done */
                                            let res = Twitter::do_task(doer_id, task.id, connection).await;
                                            
                                            resp!{
                                                String, //// the data type
                                                tusername, //// response data
                                                TWITTER_VERIFIED_USERNAME, //// response message
                                                StatusCode::OK, //// status code
                                                None::<Cookie<'_>>, //// cookie
                                            }
                                        
                                        },
                                        _ => {
                    
                                            /* user task has already been inserted  */
                                            let resp = Response::<&[u8]>{
                                                data: Some(&[]),
                                                message: USER_TASK_HAS_ALREADY_BEEN_INSERTED,
                                                status: 302
                                            };
                                            return Ok(
                                                HttpResponse::Found().json(resp)
                                            );
                    
                                        }
                                    }

                                } else{
            
                                    resp!{
                                        String, //// the data type
                                        tusername, //// response data
                                        TWITTER_USER_IS_NOT_VALID, //// response message
                                        StatusCode::NOT_ACCEPTABLE, //// status code
                                        None::<Cookie<'_>>, //// cookie
                                    }
                                }
            
                            },
                            None => {

                                resp!{
                                    String, //// the data type
                                    tusername, //// response data
                                    TWITTER_USER_FOLLOWERS_NOT_FOUND, //// response message
                                    StatusCode::NOT_FOUND, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            }
                        }
                    },
                    Err(e) => {

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            &e.to_string(), //// response message
                            StatusCode::INTERNAL_SERVER_ERROR, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    }
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
        
        if self.endpoint.is_some(){

            let user_existance_endpoint = format!("{}/user-verification", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.twitter_username.unwrap_or("".to_string()));
            map.insert("code", user.activity_code); /* activity code used to check that the user is activated or not */
            
            verify!{
                user_existance_endpoint.as_str(), 
                map,
                task.id,
                doer_id,
                connection,
                redis_client
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

            for tweet in user_tweets{
                if tweet.text.contains(&user_activity_code){
                    
                    is_verified = true;
                    
                }
            }

            if is_verified{

                match UserTask::find(doer_id, task.id, connection).await{
                    false => {

                        /* try to insert into users_tasks since it's done */
                        let res = Twitter::do_task(doer_id, task.id, connection).await;
                        
                        resp!{
                            String, //// the data type
                            tusername, //// response data
                            TWITTER_VERIFIED_CODE, //// response message
                            StatusCode::OK, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    
                    },
                    _ => {

                        /* user task has already been inserted  */
                        let resp = Response::<&[u8]>{
                            data: Some(&[]),
                            message: USER_TASK_HAS_ALREADY_BEEN_INSERTED,
                            status: 302
                        };
                        return Ok(
                            HttpResponse::Found().json(resp)
                        );

                    }
                }

                
            } else{
                resp!{
                    String, //// the data type
                    tusername, //// response data
                    TWITTER_CODE_IS_NOT_VALID, //// response message
                    StatusCode::NOT_ACCEPTABLE, //// status code
                    None::<Cookie<'_>>, //// cookie
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
        
        if self.endpoint.is_some(){

            let user_existance_endpoint = format!("{}/check", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.twitter_username.unwrap_or("".to_string()));
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
                redis_client
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

            for tweet in user_tweets{
                if tweet.text.contains(&tweet_content) && tweet.text.len() == tweet_content.len(){
                    let tweet_id = tweet.id;
                    link = format!("https://twitter.com/{tusername:}/status/{tweet_id:}");
                    is_verified = true;
                    
                }
            }

            if is_verified{

                match UserTask::find(doer_id, task.id, connection).await{
                    false => {

                        /* try to insert into users_tasks since it's done */
                        let res = Twitter::do_task(doer_id, task.id, connection).await;
                        
                        resp!{
                            String, //// the data type
                            link, //// response data
                            TWITTER_VERIFIED_TWEET, //// response message
                            StatusCode::OK, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    
                    },
                    _ => {

                        /* user task has already been inserted  */
                        let resp = Response::<&[u8]>{
                            data: Some(&[]),
                            message: USER_TASK_HAS_ALREADY_BEEN_INSERTED,
                            status: 302
                        };
                        return Ok(
                            HttpResponse::Found().json(resp)
                        );

                    }
                }

                
            } else{
                resp!{
                    String, //// the data type
                    tusername, //// response data
                    TWITTER_NOT_VERIFIED_TWEET_CONTENT, //// response message
                    StatusCode::NOT_ACCEPTABLE, //// status code
                    None::<Cookie<'_>>, //// cookie
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
        
        if self.endpoint.is_some(){

            let user_existance_endpoint = format!("{}/check", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.twitter_username.unwrap_or("".to_string()));
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
                redis_client
            }

        } else{

            let tusername = user.twitter_username.unwrap_or("".to_string());
            let like_tweet_id = task.like_tweet_id;
            
            let get_user_twitter_data = self.get_twitter_user_info(&tusername).await;
            let Ok(twitter_user_data) = get_user_twitter_data else{
                return get_user_twitter_data.unwrap_err()
            };

            match self.twitter_api
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

                                    match UserTask::find(doer_id, task.id, connection).await{
                                        false => {
                    
                                            /* try to insert into users_tasks since it's done */
                                            let res = Twitter::do_task(doer_id, task.id, connection).await;
                                            
                                            resp!{
                                                String, //// the data type
                                                tusername, //// response data
                                                TWITTER_VERIFIED_LIKE, //// response message
                                                StatusCode::OK, //// status code
                                                None::<Cookie<'_>>, //// cookie
                                            }
                                        
                                        },
                                        _ => {
                    
                                            /* user task has already been inserted  */
                                            let resp = Response::<&[u8]>{
                                                data: Some(&[]),
                                                message: USER_TASK_HAS_ALREADY_BEEN_INSERTED,
                                                status: 302
                                            };
                                            return Ok(
                                                HttpResponse::Found().json(resp)
                                            );
                    
                                        }
                                    }

                                    
                                } else{
                                    resp!{
                                        String, //// the data type
                                        tusername, //// response data
                                        TWITTER_NOT_VERIFIED_LIKE, //// response message
                                        StatusCode::NOT_ACCEPTABLE, //// status code
                                        None::<Cookie<'_>>, //// cookie
                                    }
                                }

                            },
                            None => {

                                resp!{
                                    String, //// the data type
                                    tusername, //// response data
                                    TWITTER_USER_TWEETS_NOT_FOUND, //// response message
                                    StatusCode::NOT_FOUND, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }
                            }
                        }

                    },
                    Err(e) => {

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            &e.to_string(), //// response message
                            StatusCode::INTERNAL_SERVER_ERROR, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }
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
        
        if self.endpoint.is_some(){

            let user_existance_endpoint = format!("{}/check", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.twitter_username.unwrap_or("".to_string()));
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
                redis_client
            }

        } else{

            let tusername = user.twitter_username.unwrap_or("".to_string());
            let retweet_id = task.retweet_id.parse::<u64>().unwrap();
            
            let get_user_twitter_data = self.get_twitter_user_info(&tusername).await;
            let Ok(twitter_user_data) = get_user_twitter_data else{
                return get_user_twitter_data.unwrap_err()
            };


            let mut is_verified = false;
            let twitter_main_account = env::var("TWITTER_MAIN_ACCOUNT").unwrap_or("BeGuy".to_string());
            
            match self.twitter_api
                .get_tweet(NumericId::new(retweet_id))
                .tweet_fields([TweetField::Text])
                .send()
                .await
                
                {
                    Ok(res) => {

                        match res.into_data(){
                            Some(tweet_data) => {

                                let tweet_text = tweet_data.text;
                                let tweet_text = format!("RT @{twitter_main_account:}: {tweet_text:}");

                                let get_user_tweets = self.get_twitter_user_tweets(twitter_user_data.id, tusername.clone()).await;
                                let Ok(user_tweets) =  get_user_tweets else{
                                    return get_user_tweets.unwrap_err();
                                };


                                for tweet in user_tweets{
                                    if tweet.text == tweet_text{
                                        is_verified = true;
                                    }
                                }


                                if is_verified{

                                    match UserTask::find(doer_id, task.id, connection).await{
                                        false => {
                    
                                            /* try to insert into users_tasks since it's done */
                                            let res = Twitter::do_task(doer_id, task.id, connection).await;
                                            
                                            resp!{
                                                String, //// the data type
                                                tusername, //// response data
                                                TWITTER_VERIFIED_RETWEET, //// response message
                                                StatusCode::OK, //// status code
                                                None::<Cookie<'_>>, //// cookie
                                            }
                                        
                                        },
                                        _ => {
                    
                                            /* user task has already been inserted  */
                                            let resp = Response::<&[u8]>{
                                                data: Some(&[]),
                                                message: USER_TASK_HAS_ALREADY_BEEN_INSERTED,
                                                status: 302
                                            };
                                            return Ok(
                                                HttpResponse::Found().json(resp)
                                            );
                    
                                        }
                                    }

                                    
                                } else{
                                    resp!{
                                        String, //// the data type
                                        tusername, //// response data
                                        TWITTER_NOT_VERIFIED_RETWEET, //// response message
                                        StatusCode::NOT_ACCEPTABLE, //// status code
                                        None::<Cookie<'_>>, //// cookie
                                    }
                                }

                            },
                            None => {

                                resp!{
                                    u64, //// the data type
                                    retweet_id, //// response data
                                    TWITTER_TWEET_NOT_FOUND, //// response message
                                    StatusCode::NOT_FOUND, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            }
                        }
                    
                    },
                    Err(e) => {

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            &e.to_string(), //// response message
                            StatusCode::INTERNAL_SERVER_ERROR, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }
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
        
        if self.endpoint.is_some(){

            let user_existance_endpoint = format!("{}/check", self.endpoint.as_ref().unwrap());
            let mut map = HashMap::new();
            map.insert("username", user.twitter_username.unwrap_or("".to_string()));
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
                redis_client
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
            for tweet in user_tweets{

                if tweet.text.contains(&hastag){
                    is_verified = true;
                }
                
            }

            if is_verified{

                match UserTask::find(doer_id, task.id, connection).await{
                    false => {

                        /* try to insert into users_tasks since it's done */
                        let res = Twitter::do_task(doer_id, task.id, connection).await;
                        
                        resp!{
                            String, //// the data type
                            tusername, //// response data
                            TWITTER_VERIFIED_HASHTAG, //// response message
                            StatusCode::OK, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    
                    },
                    _ => {

                        /* user task has already been inserted  */
                        let resp = Response::<&[u8]>{
                            data: Some(&[]),
                            message: USER_TASK_HAS_ALREADY_BEEN_INSERTED,
                            status: 302
                        };
                        return Ok(
                            HttpResponse::Found().json(resp)
                        );

                    }
                }

                
            } else{
                resp!{
                    String, //// the data type
                    tusername, //// response data
                    TWITTER_NOT_VERIFIED_HASHTAG, //// response message
                    StatusCode::NOT_ACCEPTABLE, //// status code
                    None::<Cookie<'_>>, //// cookie
                }
            }

        }


    }

    async fn get_twitter_user_info(&self, tusername: &str) -> Result<TwitterUser, Result<HttpResponse, actix_web::Error>>{

        match self.twitter_api
            .get_user_by_username(tusername.clone())
            .user_fields([UserField::Id, UserField::Username, UserField::CreatedAt, UserField::Verified, UserField::Entities])
            .send()
            .await
            {
                Ok(res) => {

                    match res.into_data(){
                        Some(user_data) => {

                            Ok(
                                user_data
                            )
                            
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

    async fn get_twitter_user_tweets(&self, twitter_user_id: NumericId, user_twitter_username: String) -> Result<Vec<Tweet>, Result<HttpResponse, actix_web::Error>>{

        match self.twitter_api
            .get_user_tweets(twitter_user_id)
            .send()
            .await
            {
                Ok(res) => {

                    match res.into_data(){
                        Some(tweets) => {

                            Ok(tweets)

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

    pub async fn do_task(
        doer_id: i32, job_id: i32, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<HttpResponse, actix_web::Error>{
        
        match UserTask::insert(doer_id, job_id, connection).await{
            Ok(_) => {
                resp!{
                    &[u8], //// the data type
                    &[], //// response data
                    TASK_CREATED, //// response message
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