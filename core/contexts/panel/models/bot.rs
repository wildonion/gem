


use crate::*;
use super::users_tasks::UserTask;
use crate::constants::*;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bot{
    access_token: String, 
    access_token_secret: String,
    consumer_key: String,
    consumer_secret: String 
}

impl Bot{

    pub fn new() -> Self{

        // 🥑 todo - read from env also update deploy.sh to ask user input for these like discord bot  

        Self{
            access_token: "".to_string(),
            access_token_secret: "".to_string(),
            consumer_key: "".to_string(),
            consumer_secret: "".to_string(),
        }
    }

    pub fn verify_username(&self){

        // 🥑 todo 
        // - call twitter api to check the username 
        // - if the username was invalid then
        //  - delete user task from users_tasks if it's in there and is done
        //  - otherwise don't insert it
        // - else call the self.do_task()
    }

    pub fn verify_tweets(&self){
        
        // 🥑 todo 
        // - call twitter api to check the tweets of the user 
        // - if there wasn't any then
        //  - delete user task from users_tasks if it's in there and is done
        //  - otherwise don't insert it
        // - else call the self.do_task()

    }
    
    pub fn verify_likes(&self){
        
        // 🥑 todo 
        // - call twitter api to check the likes of the user 
        // - if there wasn't any then
        //  - delete user task from users_tasks if it's in there and is done
        //  - otherwise don't insert it
        // - else call the self.do_task()

    }
    
    pub fn verify_retweets(&self){
        
        // 🥑 todo 
        // - call twitter api to check the retweets of the user 
        // - if there wasn't any then
        //  - delete user task from users_tasks if it's in there and is done
        //  - otherwise don't insert it
        // - else call the self.do_task()

    }

    pub fn verify_hashtags(&self){
        
        // 🥑 todo 
        // - call twitter api to check the hashtags of the user 
        // - if there wasn't any then
        //  - delete user task from users_tasks if it's in there and is done
        //  - otherwise don't insert it
        // - else call the self.do_task()

    }

    pub async fn do_task(
        &self, doer_id: i32, job_id: i32, 
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