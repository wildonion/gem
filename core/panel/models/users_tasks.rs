

use crate::*;
use crate::misc::Response;
use crate::schema::users;
use crate::schema::users::dsl::*;
use crate::constants::*;
use crate::schema::tasks;
use crate::schema::tasks::dsl::*;
use crate::schema::users_tasks;
use crate::schema::users_tasks::dsl::*;
use crate::models::{users::{User, UserData, UserRole}, tasks::{Task, TaskData}};





/* 

    diesel migration generate users_tasks ---> create users_tasks migration sql files
    diesel migration run                  ---> apply sql files to db 
    diesel migration redo                 ---> drop tables 

*/
#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Task))]
#[diesel(table_name=users_tasks)]
#[diesel(primary_key(user_id, task_id))]
pub struct UserTask {
    pub user_id: i32,
    pub task_id: i32,
    pub done_at: chrono::NaiveDateTime
}

#[derive(Insertable)]
#[diesel(table_name=users_tasks)]
pub struct NewUserTask{
    pub task_id: i32,
    pub user_id: i32
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct FetchUserTaskReport{
    pub total_score: i32,
    pub done_tasks: Vec<TaskData>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct UserTaskData{
    pub user: UserData,
    pub tasks: Vec<Task>
}

impl UserTask{

    pub async fn all(connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<UserTask>, PanelHttpResponse>{

        match users_tasks.load::<UserTask>(connection)
            {
                Ok(users_task_data) => Ok(users_task_data),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserTask::all");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );
                }
            }

    }

    pub async fn insert(
        doer_id: i32, job_id: i32, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<usize, PanelHttpResponse>{

        let single_task = tasks
            .filter(tasks::id.eq(job_id))
            .first::<Task>(connection);

        let Ok(taks) = single_task else{

            let resp = Response{
                data: Some(job_id),
                message: TASK_NOT_FOUND,
                status: 404
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
            
        };

        match diesel::insert_into(users_tasks::table)
            .values(&NewUserTask{
                task_id: job_id,
                user_id: doer_id
            })
            .execute(connection)
            {
                Ok(affected_row) => Ok(affected_row),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserTask::insert");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn reports(doer_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<FetchUserTaskReport, PanelHttpResponse>{

        let user = match users::table
            .filter(users::id.eq(doer_id))
            .select(User::as_select())
            .get_result(connection)
            {
                Ok(fetched_user) => fetched_user,
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserTask::reports");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            };

        /* get all found user tasks which are done already since users_tasks are done tasks by the user */
        match UserTask::belonging_to(&user)
            .inner_join(tasks::table)
            .select(Task::as_select())
            .load(connection)
            {
                Ok(tasks_info) => {

                    let report = FetchUserTaskReport{
                        total_score: {
                            tasks_info
                                .clone()
                                .into_iter()
                                .map(|task| task.task_score)
                                .collect::<Vec<i32>>()
                                .into_iter()
                                .sum()
                        },
                        done_tasks: {
                            tasks_info
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
                        },
                    };    
                    
                    Ok(report)
                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserTask::reports");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            } 

    }

    pub async fn tasks_per_user(connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<UserTaskData>, PanelHttpResponse>{

        let all_users: Vec<User> = match users::table
            .select(User::as_select())
            .load(connection)
            {
                Ok(fetched_users) => fetched_users,
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserTask::tasks_per_user");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            };

   
        /* get all users tasks belong to all users by joining on UserTask and Task tables */
        let users_jobs: Vec<(UserTask, Task)> = match UserTask::belonging_to(&all_users)
            .inner_join(tasks::table)
            .select((UserTask::as_select(), Task::as_select()))
            .load(connection)
            {
                Ok(fetched_user_tasks) => {
                    fetched_user_tasks
                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserTask::tasks_per_user");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            };
    
        /* all users including their tasks (tasks per each user) */
        let tasks_per_user: Vec<UserTaskData> = users_jobs
            .grouped_by(&all_users)
            .into_iter()
            .zip(all_users)
            .map(|(t, user)| {
                UserTaskData{
                    user: UserData { 
                        id: user.id, 
                        region: user.region.clone(),
                        username: user.username, 
                        activity_code: user.activity_code,
                        twitter_username: user.twitter_username, 
                        facebook_username: user.facebook_username, 
                        discord_username: user.discord_username, 
                        identifier: user.identifier, 
                        user_role: {
                            match user.user_role.clone(){
                                UserRole::Admin => "Admin".to_string(),
                                UserRole::User => "User".to_string(),
                                _ => "Dev".to_string(),
                            }
                        },
                        token_time: user.token_time,
                        last_login: { 
                            if user.last_login.is_some(){
                                Some(user.last_login.unwrap().to_string())
                            } else{
                                Some("".to_string())
                            }
                        },
                        created_at: user.created_at.to_string(),
                        updated_at: user.updated_at.to_string(),
                        mail: user.mail,
                        is_mail_verified: user.is_mail_verified,
                        is_phone_verified: user.is_phone_verified,
                        phone_number: user.phone_number,
                        paypal_id: user.paypal_id,
                        account_number: user.account_number,
                        device_id: user.device_id,
                        social_id: user.social_id,
                        cid: user.cid,
                        screen_cid: user.screen_cid,
                        snowflake_id: user.snowflake_id,
                        stars: user.stars
                    },
                    tasks: {
                        let jobs = t
                            .into_iter()
                            .map(|(_, t)| t)
                            .collect::<Vec<Task>>();
                        jobs
                    }
                }

            })
            .collect();

        Ok(tasks_per_user)

    }

    pub async fn find(doer_id: i32, job_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> bool{

        let single_user_task = users_tasks
            .filter(users_tasks::user_id.eq(doer_id))
            .filter(users_tasks::task_id.eq(job_id))
            .first::<UserTask>(connection);

        let Ok(_) = single_user_task else{

            return false;
            
        };

        return true;
    }

    pub async fn find_by_doer(doer_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> bool{

        let single_user_task = users_tasks
            .filter(users_tasks::user_id.eq(doer_id))
            .first::<UserTask>(connection);

        let Ok(_) = single_user_task else{

            return false;
            
        };

        return true;
    }

    pub async fn delete_by_doer(doer_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<usize, PanelHttpResponse>{

        match diesel::delete(users_tasks.filter(users_tasks::user_id.eq(doer_id)))
            .execute(connection)
            {
                Ok(users_tasks_num_deleted) => Ok(users_tasks_num_deleted),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserTask::delete_by_doer");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );
                }
            }

    }

    pub async fn delete_by_task(job_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<usize, PanelHttpResponse>{

        match diesel::delete(users_tasks.filter(users_tasks::task_id.eq(job_id)))
            .execute(connection)
            {
                Ok(users_tasks_num_deleted) => Ok(users_tasks_num_deleted),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserTask::delete_by_task");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );
                }
            }

    }

}