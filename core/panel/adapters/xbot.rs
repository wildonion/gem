


#[macro_export]
macro_rules! verify {
    (
      $endpoint:expr,
      $body:expr,
      $task_id:expr,
      $doer_id:expr,
      $connection:expr,
      $redis_client:expr,
      $task_type:expr,
      $tusername:expr,
      $tweet_link:expr
    ) 
    => {

        { // this is required if we want to import modules and use the let statements

            use crate::models::xbot::Twitter;
            use crate::helpers::misc::Response;

            info!("🤖 sending request to the twitter bot hosted on [{:#?}]", $endpoint);
            let response_value: serde_json::Value = reqwest::Client::new()
                .post($endpoint)
                .json(&$body)
                .send()
                .await.unwrap()
                .json()
                .await.unwrap();

            /* I believe that the bot code has some shity response structure :) since I didn't designed it*/

            let data_field = response_value.get("data");
            if data_field.is_some(){
                let status = data_field.unwrap().get("status");
                if status.is_some(){

                    // ------------- catching an x request 
                    catch_x_log($redis_client.clone()).await;
                    // --------------------------------------

                    let bool_status = status.unwrap().to_string();
                    if bool_status == "false"{

                        /* twitter error */

                        match diesel::delete(users_tasks
                            .filter(users_tasks::task_id.eq($task_id)))
                            .filter(users_tasks::user_id.eq($doer_id))
                            .execute($connection)
                            {
                                Ok(num_deleted) => {
                                    
                                    if num_deleted > 0{
            
                                        let resp = Response::<&[u8]>{
                                            data: Some(&[]),
                                            message: TASK_NOT_VERIFIED,
                                            status: 406,
                                            is_error: true,
                                        };
                                        return Ok(
                                            HttpResponse::NotAcceptable().json(resp)
                                        );                                
            
                                    } else{
                                        
                                        let resp = Response::<&[u8]>{
                                            data: Some(&[]),
                                            message: USER_TASK_HAS_ALREADY_BEEN_DELETED,
                                            status: 417,
                                            is_error: true
                                        };
                                        return Ok(
                                            HttpResponse::ExpectationFailed().json(resp)
                                        ); 
            
                                    }
                                
                                },
                                Err(e) => {
            
                                    let resp = Response::<&[u8]>{
                                        data: Some(&[]),
                                        message: &e.to_string(),
                                        status: 500,
                                        is_error: true
                                    };
                                    return Ok(
                                        HttpResponse::InternalServerError().json(resp)
                                    );
            
                                }
                            }

                    } else{

                        /* task is verified by twitter */

                        match UserTask::find($doer_id, $task_id, $connection).await{
                            false => {

                                /* try to insert into users_tasks since it's done */
                                let res = Twitter::do_task($doer_id, $task_id, $tusername, $task_type, $tweet_link, $connection).await;
                                return res;
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

                    }
                } else{

                    /* twitter rate limit issue */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: TWITTER_RATE_LIMIT,
                        status: 406,
                        is_error: true
                    };
                    return Ok(
                        HttpResponse::NotAcceptable().json(resp)
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
                return Ok(
                    HttpResponse::NotAcceptable().json(resp)
                );  
            }
        }
    }
}