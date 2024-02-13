




use crate::*;
use crate::models::users_collections::{UserCollectionData, UserCollection, CollectionInfoResponse};
use crate::models::users_nfts::{UserNftData, UserNft, NftLike, LikeUserInfo, UserLikeStat, NftUpvoterLikes, NftColInfo, UserCollectionDataGeneralInfo};
use crate::schema::users_galleries::dsl::users_galleries;
use crate::models::users_galleries::{UserPrivateGallery, UserPrivateGalleryData};
use crate::models::{users::*, tasks::*, users_tasks::*, xbot::*};
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use chrono::NaiveDateTime;
use rand::seq::SliceRandom;
use s3req::Storage;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;




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

#[get("/tasks/leaderboard/")]
pub(self) async fn tasks_leaderboard(
        req: HttpRequest,
        limit: web::Query<Limit>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            /* 
                if we need a mutable version of data to be moved between different scopes we can 
                use &mut type in a single thread scenarios otherwise Arc<Mutex or RwLock in multiple 
                thread, by mutating the &mut type in a new scope the actual type will be mutated too, 
                generally we should pass by value instead of reference cause rust will take care of 
                the lifetime and reference counting of the type
            */
            let connection = &mut pg_pool.get().unwrap();

            match UserTask::all_with_limit(limit, connection).await{
                Ok(users_tasks_data) => {

                    let mut leaderboard: Vec<FetchUserTaskReport> = vec![];
                    if !users_tasks_data.is_empty(){
                        for utinfo in users_tasks_data{
                            let get_user_report = UserTask::reports_without_limit(utinfo.user_id, connection).await;
                            leaderboard.push(get_user_report.unwrap());
                        }
                    }

                    resp!{
                        Vec<FetchUserTaskReport>, // the data type
                        leaderboard, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
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

#[get("/get-user-wallet-info/{identifier}")]
pub(self) async fn get_user_wallet_info(
        req: HttpRequest,   
        user_identifier: web::Path<String>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            match User::fetch_wallet_by_username_or_mail_or_scid(&user_identifier.to_owned(), connection).await{

                Ok(user_info) => {

                    resp!{
                        UserWalletInfoResponse, // the data type
                        user_info, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
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

#[get("/nft/get/collections/for/{col_owner}")]
pub(self) async fn get_nft_product_collections(
        req: HttpRequest,
        col_owner: web::Path<String>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            match UserCollection::get_all_nft_product_collections_by_owner(&col_owner.to_owned(), connection).await{

                Ok(collection_info) => {

                    resp!{
                        Vec<CollectionInfoResponse>, // the data type
                        collection_info, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
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

#[get("/get-users-wallet-info/")]
pub(self) async fn get_users_wallet_info(
        req: HttpRequest,   
        limit: web::Query<Limit>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            match User::fetch_all_users_wallet_info(limit, connection).await{

                Ok(users_info) => {

                    resp!{
                        Vec<Option<UserWalletInfoResponseWithBalance>>, // the data type
                        {
                            let mut users_info = users_info
                                .into_iter()
                                .map(|user|{
                                    if user.username == "adminy" || user.username == "devdevy"{
                                        None
                                    } else{
                                        Some(user)
                                    }
                                })
                                .collect::<Vec<Option<UserWalletInfoResponseWithBalance>>>();
                            users_info.retain(|user| user.is_some());
                            users_info
                            
                        }, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
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

#[get("/get-top-nfts/")]
pub(self) async fn get_top_nfts(
        req: HttpRequest,   
        limit: web::Query<Limit>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            let from = limit.from.unwrap_or(0) as usize;
            let to = limit.to.unwrap_or(10) as usize;

            if to < from {
                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: INVALID_QUERY_LIMIT,
                    status: 406,
                    is_error: true
                };
                return Ok(HttpResponse::NotAcceptable().json(resp));
                
            }

            let get_nfts = UserNft::get_all(connection).await;
            let Ok(nfts) = get_nfts else{
                let err_resp = get_nfts.unwrap_err();
                return err_resp;
            };

            let mut nft_like_map = vec![];
            for nft in nfts{
                
                let nft_likes = nft.likes;
                let mut decoded_likes = if nft_likes.is_some(){
                    serde_json::from_value::<Vec<NftLike>>(nft_likes.unwrap()).unwrap()
                } else{
                    vec![]
                };  
                
                for like in decoded_likes{
                    nft_like_map.push(
                        NftUpvoterLikes{
                            id: nft.id,
                            upvoter_screen_cids: like.upvoter_screen_cids.len() as u64
                        }
                    );
                }

            }
            
            // sort by the most likes to less ones
            nft_like_map.sort_by(|nl1, nl2|{

                let nl1_likes = nl1.upvoter_screen_cids;
                let nl2_likes = nl2.upvoter_screen_cids;

                nl2_likes.cmp(&nl1_likes)

            });
            
            let top_nfts = nft_like_map
                .into_iter()
                .map(|nlinfo|{

                    let nft = UserNft::find_by_id_none_async(nlinfo.id, connection).unwrap();
                    NftColInfo{
                        col_data: {
                            let col_info = UserCollection::find_by_contract_address_none_async(&nft.contract_address, connection).unwrap();
                            UserCollectionDataGeneralInfo{
                                id: col_info.id,
                                contract_address: col_info.contract_address,
                                col_name: col_info.col_name,
                                symbol: col_info.symbol,
                                owner_screen_cid: col_info.owner_screen_cid,
                                metadata_updatable: col_info.metadata_updatable,
                                freeze_metadata: col_info.freeze_metadata,
                                base_uri: col_info.base_uri,
                                royalties_share: col_info.royalties_share,
                                royalties_address_screen_cid: col_info.royalties_address_screen_cid,
                                collection_background: col_info.collection_background,
                                extra: col_info.extra,
                                col_description: col_info.col_description,
                                contract_tx_hash: col_info.contract_tx_hash,
                                created_at: col_info.created_at.to_string(),
                                updated_at: col_info.updated_at.to_string(),
                            }
                        },
                        nfts_data: nft,
                    }

                })
                .collect::<Vec<NftColInfo>>();


            let sliced = if from < top_nfts.len(){
                if top_nfts.len() > to{
                    let data = &top_nfts[from..to+1];
                    data.to_vec()
                } else{
                    let data = &top_nfts[from..top_nfts.len()];
                    data.to_vec()
                }
            } else{
                vec![]
            };
            
            resp!{
                Vec<NftColInfo>, // the data type
                sliced, // response data
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

#[get("/get-all-minted-nfts/")]
pub(self) async fn get_all_nfts(
        req: HttpRequest,   
        limit: web::Query<Limit>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            let from = limit.from.unwrap_or(0) as usize;
            let to = limit.to.unwrap_or(10) as usize;

            if to < from {
                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: INVALID_QUERY_LIMIT,
                    status: 406,
                    is_error: true
                };
                return Ok(HttpResponse::NotAcceptable().json(resp));
                
            }

            let get_nfts = UserNft::get_all(connection).await;
            let Ok(nfts) = get_nfts else{
                let err_resp = get_nfts.unwrap_err();
                return err_resp;
            };

            let mut minted_ones = vec![];
            for nft in nfts{
                if nft.is_minted.is_some() && nft.is_minted.clone().unwrap(){
                    minted_ones.push(
                        NftColInfo{
                            col_data: {
                                let col_info = UserCollection::find_by_contract_address(&nft.contract_address, connection).await.unwrap();
                                UserCollectionDataGeneralInfo{
                                    id: col_info.id,
                                    contract_address: col_info.contract_address,
                                    col_name: col_info.col_name,
                                    symbol: col_info.symbol,
                                    owner_screen_cid: col_info.owner_screen_cid,
                                    metadata_updatable: col_info.metadata_updatable,
                                    freeze_metadata: col_info.freeze_metadata,
                                    base_uri: col_info.base_uri,
                                    royalties_share: col_info.royalties_share,
                                    royalties_address_screen_cid: col_info.royalties_address_screen_cid,
                                    collection_background: col_info.collection_background,
                                    extra: col_info.extra,
                                    col_description: col_info.col_description,
                                    contract_tx_hash: col_info.contract_tx_hash,
                                    created_at: col_info.created_at.to_string(),
                                    updated_at: col_info.updated_at.to_string(),
                                }
                            },
                            nfts_data: nft,
                        }
                    )
                }

            }
            
            // let mut rng = rand::thread_rng();
            // minted_ones.shuffle(&mut rng);

            minted_ones.sort_by(|nftcol1, nftcol2|{

                let nftcol1_created_at = NaiveDateTime
                    ::parse_from_str(&nftcol1.nfts_data.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                let nftcol2_created_at = NaiveDateTime
                    ::parse_from_str(&nftcol2.nfts_data.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                nftcol2_created_at.cmp(&nftcol1_created_at)

            });

            let sliced = if from < minted_ones.len(){
                if minted_ones.len() > to{
                    let data = &minted_ones[from..to+1];
                    data.to_vec()
                } else{
                    let data = &minted_ones[from..minted_ones.len()];
                    data.to_vec()
                }
            } else{
                vec![]
            };

            resp!{
                Vec<NftColInfo>, // the data type
                sliced, // response data
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

#[get("/search/")]
pub(self) async fn search(
        req: HttpRequest,   
        query: web::Query<Search>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            let from = query.from.unwrap_or(0) as usize;
            let to = query.to.unwrap_or(10) as usize;

            if to < from {
                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: INVALID_QUERY_LIMIT,
                    status: 406,
                    is_error: true
                };
                return Ok(HttpResponse::NotAcceptable().json(resp));
                
            }

            /* search in users */
            let query_to_seatch = format!("%{}%", query.q);
            let get_users_info = users::table
                .filter(
                    users::username.ilike(query_to_seatch.as_str())
                        .or(
                            users::screen_cid.ilike(query_to_seatch.as_str())
                        )
                        .or(
                            users::mail.ilike(query_to_seatch.as_str())
                        )
                        .or(
                            users::phone_number.ilike(query_to_seatch.as_str())
                        )
                )
                .order(users::created_at.desc())
                .load::<User>(connection);

            let Ok(users_info) = get_users_info else{
                let err = get_users_info.unwrap_err();
                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: &err.to_string(),
                    status: 500,
                    is_error: true
                };
                return 
                    Ok(HttpResponse::InternalServerError().json(resp))
                
            };

            let users_info = 
                users_info
                    .into_iter()
                    .map(|u|{
                        UserData { 
                            id: u.id, 
                            region: u.region.clone(),
                            username: u.clone().username,
                            bio: u.bio.clone(),
                            avatar: u.avatar.clone(),
                            banner: u.banner.clone(), 
                            wallet_background: u.wallet_background.clone(), 
                            activity_code: u.clone().activity_code, 
                            twitter_username: u.clone().twitter_username, 
                            facebook_username: u.clone().facebook_username, 
                            discord_username: u.clone().discord_username, 
                            identifier: u.clone().identifier, 
                            user_role: {
                                match u.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: u.token_time,
                            balance: u.balance,
                            last_login: { 
                                if u.last_login.is_some(){
                                    Some(u.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: u.created_at.to_string(),
                            updated_at: u.updated_at.to_string(),
                            mail: u.clone().mail,
                            google_id: u.clone().google_id,
                            microsoft_id: u.clone().microsoft_id,
                            is_mail_verified: u.is_mail_verified,
                            is_phone_verified: u.is_phone_verified,
                            phone_number: u.clone().phone_number,
                            paypal_id: u.clone().paypal_id,
                            account_number: u.clone().account_number,
                            device_id: u.clone().device_id,
                            social_id: u.clone().social_id,
                            cid: u.clone().cid,
                            screen_cid: u.clone().screen_cid,
                            snowflake_id: u.snowflake_id,
                            stars: u.stars,
                            extra: u.clone().extra,
                        }
                    })
                    .collect::<Vec<UserData>>();
                         
            /* search in galleries, collections and nfts */
            let get_user_galleries_info = users_galleries
                .load::<UserPrivateGallery>(connection);

            let Ok(galleries_info) = get_user_galleries_info else{
                let err = get_user_galleries_info.unwrap_err();
                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: &err.to_string(),
                    status: 500,
                    is_error: true
                };
                return 
                    Ok(HttpResponse::InternalServerError().json(resp))
                
            };

            let mut galleries_info = 
                galleries_info
                    .into_iter()
                    .map(|g|{

                        UserPrivateGalleryData{ 
                            id: g.id, 
                            owner_screen_cid: g.owner_screen_cid, 
                            collections: g.collections, 
                            gal_name: g.gal_name, 
                            gal_description: g.gal_description, 
                            invited_friends: g.invited_friends, 
                            extra: g.extra, 
                            gallery_background: g.gallery_background,
                            created_at: g.created_at.to_string(), 
                            updated_at: g.updated_at.to_string() 
                        }

                    })
                    .collect::<Vec<UserPrivateGalleryData>>();

            /* order based on newest ones */
            galleries_info.sort_by(|g1, g2|{

                let g1_created_at = NaiveDateTime
                    ::parse_from_str(&g1.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                let g2_created_at = NaiveDateTime
                    ::parse_from_str(&g2.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                g2_created_at.cmp(&g1_created_at)

            });

            let mut found_collections = vec![];
            let mut found_nfts = vec![];
            for gallery in galleries_info{

                let cols = gallery.collections;
                let decoded_cols = if cols.is_some(){
                    serde_json::from_value::<Vec<UserCollectionData>>(cols.unwrap()).unwrap()
                } else{
                    vec![]
                };

                let match_collections = decoded_cols.clone()
                    .into_iter()
                    .map(|col| {

                        if col.col_name.contains(&query.q) ||
                            col.col_description.contains(&query.q) ||
                            col.owner_screen_cid.contains(&query.q) || 
                            col.contract_address.contains(&query.q) || 
                            col.contract_tx_hash.clone().unwrap_or(String::from("")).contains(&query.q)
                            {
                                /* -----------------------------------------------------------------
                                    > in those case that we don't want to create a separate struct 
                                    and allocate an instance of it to map a utf8 bytes data coming
                                    from a server or client into its feilds we can use serde_json::to_value()
                                    which maps an instance of a structure into a serde json value 
                                    or serde_json::json!({}) to create a json value from those fields 
                                    that we want to return them, but if we want to mutate data in rust we 
                                    have to convert the json value or received bytes into the structure, 
                                */
                                Some(
                                    serde_json::json!({
                                        "id": col.id,
                                        "contract_address": col.contract_address,
                                        "col_name": col.col_name,
                                        "symbol": col.symbol,
                                        "owner_screen_cid": col.owner_screen_cid,
                                        "metadata_updatable": col.metadata_updatable,
                                        "freeze_metadata": col.freeze_metadata,
                                        "base_uri": col.base_uri,
                                        "royalties_share": col.royalties_share,
                                        "royalties_address_screen_cid": col.royalties_address_screen_cid,
                                        "collection_background": col.collection_background,
                                        "extra": col.extra,
                                        "col_description": col.col_description,
                                        "contract_tx_hash": col.contract_tx_hash,
                                        "created_at": col.created_at,
                                        "updated_at": col.updated_at,
                                    })
                                )
                        } else{
                            None
                        }
                    })
                    .collect::<Vec<Option<serde_json::Value>>>();
                
                found_collections.extend(match_collections);
                found_collections.retain(|col| col.is_some());

                for collection in decoded_cols{
                    
                    let colnfts = collection.clone().nfts;
                    let decoded_nfts = if colnfts.is_some(){
                        serde_json::from_value::<Vec<UserNftData>>(colnfts.unwrap()).unwrap()
                    } else{
                        vec![]
                    };

                    let match_nfts = decoded_nfts
                        .into_iter()
                        .map(|nft| {
                            if nft.is_minted.is_some() && nft.is_minted.unwrap() == true && 
                            (
                                nft.nft_name.contains(&query.q) ||
                                nft.nft_description.contains(&query.q) ||
                                nft.current_owner_screen_cid.contains(&query.q) ||
                                nft.contract_address.contains(&query.q) ||
                                nft.onchain_id.clone().unwrap().contains(&query.q) ||
                                nft.tx_hash.clone().unwrap().contains(&query.q)
                            ){
                                Some(
                                    NftColInfo{ 
                                        col_data: UserCollectionDataGeneralInfo{
                                            id: collection.id,
                                            contract_address: collection.clone().contract_address,
                                            col_name: collection.clone().col_name,
                                            symbol: collection.clone().symbol,
                                            owner_screen_cid: collection.clone().owner_screen_cid,
                                            metadata_updatable: collection.clone().metadata_updatable,
                                            freeze_metadata: collection.clone().freeze_metadata,
                                            base_uri: collection.clone().base_uri,
                                            royalties_share: collection.clone().royalties_share,
                                            royalties_address_screen_cid: collection.clone().royalties_address_screen_cid,
                                            collection_background: collection.clone().collection_background,
                                            extra: collection.clone().extra,
                                            col_description: collection.clone().col_description,
                                            contract_tx_hash: collection.clone().contract_tx_hash,
                                            created_at: collection.created_at.to_string(),
                                            updated_at: collection.updated_at.to_string(),
                                        }, 
                                        nfts_data: nft 
                                    }
                                )
                            } else{
                                None
                            }
                        })
                        .collect::<Vec<Option<NftColInfo>>>();
                    

                    found_nfts.extend(match_nfts);
                    found_nfts.retain(|nft| nft.is_some());

                }

            }

            /* order based on newest ones */
            found_nfts.sort_by(|n1, n2|{

                /* 
                    cannot move out of `*n1` which is behind a shared reference
                    move occurs because `*n1` has type `std::option::Option<NftColInfo>`, 
                    which does not implement the `Copy` trait and unwrap() takes the 
                    ownership of the instance.
                    also we must create a longer lifetime for `NftColInfo::default()` by 
                    putting it inside a type so we can take a reference to it and pass the 
                    reference to the `unwrap_or()`, cause &NftColInfo::default() will be dropped 
                    at the end of the `unwrap_or()` statement while we're borrowing it.
                */
                let n1_default = NftColInfo::default();
                let n2_default = NftColInfo::default();
                let n1 = n1.as_ref().unwrap_or(&n1_default);
                let n2 = n2.as_ref().unwrap_or(&n2_default);

                let n1_created_at = NaiveDateTime
                    ::parse_from_str(&n1.nfts_data.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                let n2_created_at = NaiveDateTime
                    ::parse_from_str(&n2.nfts_data.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                n2_created_at.cmp(&n1_created_at)

            });

            /* order based on newest ones */
            found_collections.sort_by(|c1, c2|{

                /* 
                    cannot move out of `*c1` which is behind a shared reference
                    move occurs because `*c1` has type `std::option::Option<UserCollectionData>`, 
                    which does not implement the `Copy` trait and unwrap() takes the 
                    ownership of the instance.
                    also we must create a longer lifetime for `UserCollectionData::default()` by 
                    putting it inside a type so we can take a reference to it and pass the 
                    reference to the `unwrap_or()`, cause &UserCollectionData::default() will be dropped 
                    at the end of the `unwrap_or()` statement while we're borrowing it.
                */
                let c1_default = UserCollectionData::default();
                let c2_default = UserCollectionData::default();
                let c1 = serde_json::from_value::<UserCollectionData>(c1.clone().unwrap()).unwrap_or(c1_default);
                let c2 = serde_json::from_value::<UserCollectionData>(c2.clone().unwrap()).unwrap_or(c2_default);

                let c1_created_at = NaiveDateTime
                    ::parse_from_str(&c1.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                let c2_created_at = NaiveDateTime
                    ::parse_from_str(&c2.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                c2_created_at.cmp(&c1_created_at)

            });

            /*  
                first we need to slice the current vector convert that type into 
                another vector, the reason behind doing this is becasue we can't
                call to_vec() on the slice directly since the lifetime fo the slice
                will be dropped while is getting used we have to create a longer 
                lifetime then call to_vec() on that type
            */            
            let found_collections = if from < found_collections.len(){
                if found_collections.len() > to{
                    let data = &found_collections[from..to+1];
                    data.to_vec()
                } else{
                    let data = &found_collections[from..found_collections.len()];
                    data.to_vec()
                }
            } else{
                vec![]
            };

            let found_nfts = if from < found_nfts.len(){
                if found_nfts.len() > to{
                    let data = &found_nfts[from..to+1];
                    data.to_vec()
                } else{
                    let data = &found_nfts[from..found_nfts.len()];
                    data.to_vec()
                }
            } else{
                vec![]
            };

            let users_info = if from < users_info.len(){
                if users_info.len() > to{
                    let data = &users_info[from..to+1];
                    data.to_vec()
                } else{
                    let data = &users_info[from..users_info.len()];
                    data.to_vec()
                }
            } else{
                vec![]
            };

            let mut matched_data = HashMap::new();
            matched_data.insert("collection".to_string(), serde_json::to_value(&found_collections).unwrap());
            matched_data.insert("users".to_string(), serde_json::to_value(&users_info).unwrap());
            matched_data.insert("nfts".to_string(), serde_json::to_value(&found_nfts).unwrap());
            
            resp!{
                HashMap<String, serde_json::Value>, // the data type
                matched_data, // response data
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
    pub use super::tasks_leaderboard;
    pub use super::get_user_wallet_info;
    pub use super::get_users_wallet_info;
    pub use super::search;
    pub use super::get_top_nfts;
    pub use super::get_all_nfts;
    pub use super::get_nft_product_collections;
}
