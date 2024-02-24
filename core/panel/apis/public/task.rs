

pub use super::*;


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

pub mod exports{
    pub use super::check_users_task;
    pub use super::tasks_leaderboard;
}