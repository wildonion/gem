






use crate::*;
use crate::models::users::User;
use super::users_tasks::UserTask;
use crate::helpers::misc::{Response, gen_random_chars, gen_random_idx, gen_random_number, Limit};
use crate::schema::{tasks, users, users_tasks};
use crate::schema::tasks::dsl::*;
use crate::schema::users_tasks::dsl::*;
use crate::constants::*;




/* 

    diesel migration generate tasks ---> create tasks migration sql files
    diesel migration run            ---> apply sql files to db 
    diesel migration redo           ---> drop tables 

*/

#[derive(Queryable, Selectable, Serialize, Deserialize, Identifiable, Associations, Debug, PartialEq, Clone)]
#[diesel(belongs_to(User, foreign_key=admin_id))]
#[diesel(table_name=tasks)]
pub struct Task{
    pub id: i32,
    pub task_name: String, /* username, code, tweet, retweet, comment, hashtag, like */
    pub task_description: Option<String>,
    pub task_score: i32,
    pub task_priority: i32,
    pub hashtag: String, /* hashtag that must be inside one of the user tweets */
    pub tweet_content: String, /* content that the user must tweet it */
    pub retweet_id: String, /* the tweet id that its content must be matched with one of the user tweet content, or its comment must be checked  */
    pub like_tweet_id: String, /* the tweet id that must be inside user likes */
    pub admin_id: i32, // amdin id who has defined the tasks
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskData{
    pub id: i32,
    pub task_name: String,
    pub task_description: Option<String>,
    pub task_score: i32,
    pub task_priority: i32,
    pub hashtag: String,
    pub tweet_content: String,
    pub retweet_id: String,
    pub like_tweet_id: String,
    pub admin_id: i32, // amdin id who has defined the tasks
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewTaskRequest{
    pub task_name: String,
    pub task_description: String,
    pub task_score: i32,
    pub task_priority: i32,
    pub hashtag: String,
    pub tweet_content: String,
    pub retweet_id: String,
    pub like_tweet_id: String,
    pub admin_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditTaskRequest{
    pub task_id: i32,
    pub task_name: String,
    pub task_description: String,
    pub task_priority: i32,
    pub hashtag: String,
    pub tweet_content: String,
    pub retweet_id: String,
    pub like_tweet_id: String,
    pub task_score: i32,
}

#[derive(Serialize, Deserialize)]
#[derive(Insertable, AsChangeset)]
#[diesel(table_name=tasks)]
pub struct EditTask<'t>{
    pub task_name: &'t str,
    pub task_description: &'t str,
    pub task_priority: i32,
    pub hashtag: &'t str,
    pub tweet_content: &'t str,
    pub retweet_id: &'t str,
    pub like_tweet_id: &'t str,
    pub task_score: i32,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name=tasks)]
pub struct NewTask<'t>{
    pub task_name: &'t str,
    pub task_description: Option<&'t str>,
    pub task_score: i32,
    pub task_priority: i32,
    pub hashtag: &'t str,
    pub tweet_content: &'t str,
    pub retweet_id: &'t str,
    pub like_tweet_id: &'t str,
    pub admin_id: i32
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl Task{


    pub async fn find_by_id(job_id: i32, connection: &mut DbPoolConnection) -> Result<TaskData, PanelHttpResponse>{

        let single_task = tasks
            .filter(id.eq(job_id))
            .first::<Task>(connection);
                        
        let Ok(task) = single_task else{
            let resp = Response{
                data: Some(job_id),
                message: TASK_NOT_FOUND,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(
            TaskData{
                id: task.id,
                task_name: task.task_name,
                task_description: task.task_description,
                task_score: task.task_score,
                task_priority: task.task_priority,
                hashtag: task.hashtag,
                tweet_content: task.tweet_content,
                retweet_id: task.retweet_id,
                like_tweet_id: task.like_tweet_id,
                admin_id: task.admin_id,
                created_at: task.created_at.to_string(),
                updated_at: task.updated_at.to_string(),
            }
        )

    }

    pub async fn insert(
        new_task: NewTaskRequest, 
        redis_client: &RedisClient, 
        connection: &mut DbPoolConnection) -> Result<usize, PanelHttpResponse>{
        
        let single_task = tasks
            .filter(task_name.eq(new_task.task_name.clone()))
            .first::<Task>(connection);

        if single_task.is_ok(){

            let resp = Response{
                data: Some(new_task.task_name.clone()),
                message: FOUND_TASK,
                status: 302,
                is_error: false
            };
            return Err(
                Ok(HttpResponse::Found().json(resp))
            );
            
            
        }

        let random_chars = gen_random_chars(5);
        let random_task_name = format!("{}-{}", new_task.task_name.as_str(), random_chars);

        let task = NewTask{
            task_name: random_task_name.as_str(),
            task_description: Some(new_task.task_description.as_str()),
            task_score: new_task.task_score,
            task_priority: new_task.task_priority,
            hashtag: &new_task.hashtag,
            tweet_content: &new_task.tweet_content,
            retweet_id: &new_task.retweet_id,
            like_tweet_id: &new_task.like_tweet_id,
            admin_id: new_task.admin_id,
        };

        /* ---------------------------------------- */
        // publish new task to redis pubsub channel
        /* ---------------------------------------- */
        /* 
            we're publishing asyncly to the redis pubsub XTASK channel,
            the topic data will be subscribed to in xord bot and broadcasted 
            to a discord channel later
        */
        let mut con = redis_client.get_async_connection().await.unwrap();
        let json_stringified_new_task = serde_json::to_string_pretty(&task).unwrap();
        let _: () = con.publish("XTASKS", json_stringified_new_task).await.unwrap();


        match diesel::insert_into(tasks::table)
            .values(&task)
            .execute(connection)
            {
                Ok(affected_row) => Ok(affected_row),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "Task::insert");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
                    

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn delete(job_id: i32, connection: &mut DbPoolConnection) -> Result<usize, PanelHttpResponse>{

        /* we must first delete from users_tasks */

        match UserTask::delete_by_task(job_id, connection).await {
            Ok(users_tasks_rows_deleted) => {

                match diesel::delete(tasks.filter(tasks::id.eq(job_id.to_owned())))
                    .execute(connection)
                    {
                        Ok(mut num_deleted) => {
                            
                            /* also delete any tasks record if there was any */

                            num_deleted += users_tasks_rows_deleted;

                            Ok(num_deleted)
                        
                        },
                        Err(e) => {

                            let resp_err = &e.to_string();


                            /* custom error handler */
                            use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                             
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec();  
                            let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "Task::delete");
                            let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                            let resp = Response::<&[u8]>{
                                data: Some(&[]),
                                message: resp_err,
                                status: 500,
                                is_error: true
                            };
                            return Err(
                                Ok(HttpResponse::InternalServerError().json(resp))
                            );

                        }
                    }

            },
            Err(e) => {
                
                return Err(e);
            }
        }
        
    }

    pub async fn edit(new_task: EditTaskRequest, connection: &mut DbPoolConnection) -> Result<TaskData, PanelHttpResponse>{

        match diesel::update(tasks.find(new_task.task_id.to_owned()))
            .set(EditTask{
                /* 
                    task name and description are of type &str 
                    thus by borrowing new_task struct fields we
                    can convert them into &str 
                */
                task_name: &new_task.task_name, 
                task_description: &new_task.task_description,
                task_score: new_task.task_score,
                task_priority: new_task.task_priority,
                hashtag: &new_task.hashtag,
                tweet_content: &new_task.tweet_content,
                retweet_id: &new_task.retweet_id,
                like_tweet_id: &new_task.like_tweet_id,
            })
            .returning(Task::as_returning())
            .get_result(connection)
            {
                Ok(updated_task) => {
                    Ok(
                        TaskData{
                            id: updated_task.id,
                            task_name: updated_task.task_name,
                            task_description: updated_task.task_description,
                            task_score: updated_task.task_score,
                            task_priority: updated_task.task_priority,
                            hashtag: updated_task.hashtag,
                            tweet_content: updated_task.tweet_content,
                            retweet_id: updated_task.retweet_id,
                            like_tweet_id: updated_task.like_tweet_id,
                            admin_id: updated_task.admin_id,
                            created_at: updated_task.created_at.to_string(),
                            updated_at: updated_task.updated_at.to_string(),
                        }
                    )
                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "Task::edit");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }
                    
    }

    pub async fn get_all_admin(owner_id: i32, limit: web::Query<Limit>,
        connection: &mut DbPoolConnection) -> Result<Vec<TaskData>, PanelHttpResponse>{

        let from = limit.from.unwrap_or(0);
        let to = limit.to.unwrap_or(10);

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }
        
        /* get the passed in admin info by its id */
        let user = match users::table
            .order(users::created_at.desc())
            .filter(users::id.eq(owner_id))
            .select(User::as_select())
            .offset(from)
            .limit((to - from) + 1)
            .get_result::<User>(connection)
            {
                Ok(single_user) => single_user,
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "Task::get_all_admin");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            };

        /* get all tasks belonging to the passed in admin id */
        match Task::belonging_to(&user)
            .select(Task::as_select())
            .load(connection)
            {
                Ok(admin_tasks) => {
                    Ok(
                        admin_tasks
                            .clone()
                            .into_iter()
                            .map(|t| TaskData{
                                id: t.id,
                                task_name: t.task_name,
                                task_description: t.task_description,
                                task_score: t.task_score,
                                task_priority: t.task_priority,
                                hashtag: t.hashtag,
                                tweet_content: t.tweet_content,
                                retweet_id: t.retweet_id,
                                like_tweet_id: t.like_tweet_id,
                                admin_id: t.admin_id,
                                created_at: t.created_at.to_string(),
                                updated_at: t.updated_at.to_string(),
                            })
                            .collect::<Vec<TaskData>>()
                    )
                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "Task::get_all_admin");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn get_all_admin_without_limit(owner_id: i32,
        connection: &mut DbPoolConnection) -> Result<Vec<TaskData>, PanelHttpResponse>{
        
        /* get the passed in admin info by its id */
        let user = match users::table
            .filter(users::id.eq(owner_id))
            .select(User::as_select())
            .get_result::<User>(connection)
            {
                Ok(single_user) => single_user,
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "Task::get_all_admin");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            };

        /* get all tasks belonging to the passed in admin id */
        match Task::belonging_to(&user)
            .select(Task::as_select())
            .load(connection)
            {
                Ok(admin_tasks) => {
                    Ok(
                        admin_tasks
                            .clone()
                            .into_iter()
                            .map(|t| TaskData{
                                id: t.id,
                                task_name: t.task_name,
                                task_description: t.task_description,
                                task_score: t.task_score,
                                task_priority: t.task_priority,
                                hashtag: t.hashtag,
                                tweet_content: t.tweet_content,
                                retweet_id: t.retweet_id,
                                like_tweet_id: t.like_tweet_id,
                                admin_id: t.admin_id,
                                created_at: t.created_at.to_string(),
                                updated_at: t.updated_at.to_string(),
                            })
                            .collect::<Vec<TaskData>>()
                    )
                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "Task::get_all_admin");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn get_all(limit: web::Query<Limit>,
        connection: &mut DbPoolConnection) -> Result<Vec<TaskData>, PanelHttpResponse>{
        
        let from = limit.from.unwrap_or(0);
        let to = limit.to.unwrap_or(10);

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        match tasks
            .order(tasks::created_at.desc())
            .offset(from)
            .limit((to - from) + 1)
            .load::<Task>(connection)
        {
            Ok(all_tasks) => {
                Ok(
                    all_tasks
                        .into_iter()
                        .map(|t| TaskData{
                            id: t.id,
                            task_name: t.task_name,
                            task_description: t.task_description,
                            task_score: t.task_score,
                            task_priority: t.task_priority,
                            hashtag: t.hashtag,
                            tweet_content: t.tweet_content,
                            retweet_id: t.retweet_id,
                            like_tweet_id: t.like_tweet_id,
                            admin_id: t.admin_id,
                            created_at: t.created_at.to_string(),
                            updated_at: t.updated_at.to_string(),
                        })
                        .collect::<Vec<TaskData>>()
                )
            },
            Err(e) => {

                let resp_err = &e.to_string();


                /* custom error handler */
                use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                 
                let error_content = &e.to_string();
                let error_content = error_content.as_bytes().to_vec();  
                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "Task::get_all");
                let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
                
                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: resp_err,
                    status: 500,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::InternalServerError().json(resp))
                );

            }
        }

    }

}