

use crate::*;
use crate::misc::Response;
use crate::schema::users;
use crate::schema::users::dsl::*;
use crate::constants::*;
use crate::schema::tasks;
use crate::schema::tasks::dsl::*;
use crate::schema::users_tasks;
use crate::schema::users_tasks::dsl::*;
use crate::models::{users::User, tasks::Task};





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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FetchUserTaskReport{
    pub total_score: i32,
    pub done_tasks: Vec<Task>,
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
                        done_tasks: tasks_info,
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

    pub async fn tasks_per_user(connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<(User, Vec<Task>)>, Result<HttpResponse, actix_web::Error>>{

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
        let tasks_per_user: Vec<(User, Vec<Task>)> = jobs
            .grouped_by(&all_users)
            .into_iter()
            .zip(all_users)
            .map(|(t, user)| (user, t.into_iter().map(|(_, task)| task).collect()))
            .collect();

        Ok(tasks_per_user)

    }

}