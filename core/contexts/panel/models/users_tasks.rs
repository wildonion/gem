

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
    diesel migration run            ---> apply sql files to db 
    diesel migration redo           ---> drop tables 

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

impl UserTask{

    pub async fn insert(
        doer_id: i32, job_id: i32, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<usize, Result<HttpResponse, actix_web::Error>>{

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

                    let resp = Response::<&[u8]>{
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

    pub async fn reports(doer_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<FetchUserTaskReport, Result<HttpResponse, actix_web::Error>>{

        let user = match users::table
            .filter(users::id.eq(doer_id))
            .select(User::as_select())
            .get_result(connection)
            {
                Ok(fetched_user) => fetched_user,
                Err(e) => {

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: &e.to_string(),
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

                    let resp = Response::<&[u8]>{
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

    pub async fn tasks_per_user(connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<(UserData, Vec<TaskData>)>, Result<HttpResponse, actix_web::Error>>{

        let all_users = match users::table
            .select(User::as_select())
            .load(connection)
            {
                Ok(fetched_users) => fetched_users,
                Err(e) => {

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: &e.to_string(),
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            };

        let jobs = match UserTask::belonging_to(&all_users)
            .inner_join(tasks::table)
            .select((UserTask::as_select(), Task::as_select()))
            .load(connection)
            {
                Ok(fetched_user_tasks) => fetched_user_tasks,
                Err(e) => {

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: &e.to_string(),
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            };
    
        /* all users including their tasks */
        let tasks_per_user: Vec<(UserData, Vec<TaskData>)> = jobs
            .grouped_by(&all_users)
            .into_iter()
            .zip(all_users)
            /* converting the zipped users and jobs pairs into Vec<(User, Vec<Task>)> using map */
            .map(|(t, user)| {
                let user_data = UserData { 
                    id: user.id, 
                    username: user.username, 
                    activity_code: user.activity_code,
                    twitter_username: user.twitter_username, 
                    facebook_username: user.facebook_username, 
                    discord_username: user.discord_username, 
                    wallet_address: user.wallet_address, 
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
                };
                (user_data, t.into_iter().map(|(_, task)| {
                    let task_data = TaskData{
                        id: task.id,
                        task_name: task.task_name,
                        task_description: task.task_description,
                        task_score: task.task_score,
                        admin_id: task.admin_id,
                        created_at: task.created_at.to_string(),
                        updated_at: task.updated_at.to_string(),
                    };

                    task_data

                }).collect())
            })
            .collect();

        Ok(tasks_per_user)

    }

}