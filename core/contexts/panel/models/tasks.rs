


use crate::*;
use crate::models::users::User;
use crate::misc::Response;
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
    pub task_name: String,
    pub task_description: Option<String>,
    pub task_score: i32,
    pub admin_id: i32, // amdin id who has defined the tasks
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct TaskData{
    pub id: i32,
    pub task_name: String,
    pub task_description: Option<String>,
    pub task_score: i32,
    pub admin_id: i32, // amdin id who has defined the tasks
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct NewTaskRequest{
    pub task_name: String,
    pub task_description: String,
    pub task_score: i32,
    pub admin_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct EditTaskRequest{
    pub task_id: i32,
    pub task_name: String,
    pub task_description: String,
    pub task_score: i32,
}

#[derive(Serialize, Deserialize)]
#[derive(Insertable, AsChangeset)]
#[diesel(table_name=tasks)]
pub struct EditTask<'t>{
    pub task_name: &'t str,
    pub task_description: &'t str,
    pub task_score: i32,
}

#[derive(Insertable)]
#[diesel(table_name=tasks)]
pub struct NewTask<'t>{
    pub task_name: &'t str,
    pub task_description: Option<&'t str>,
    pub task_score: i32,
    pub admin_id: i32
}

impl Task{


    pub async fn insert(
        new_task: NewTaskRequest, 
        redis_connection: &RedisClient, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<usize, Result<HttpResponse, actix_web::Error>>{
        
        let single_task = tasks
            .filter(task_name.eq(new_task.task_name.clone()))
            .first::<Task>(connection);

        if single_task.is_ok(){

            let resp = Response{
                data: Some(new_task.task_name.clone()),
                message: FOUND_TASK,
                status: 302
            };
            return Err(
                Ok(HttpResponse::Found().json(resp))
            );
            
            
        }

        let task = NewTask{
            task_name: new_task.task_name.as_str(),
            task_description: Some(new_task.task_description.as_str()),
            task_score: new_task.task_score,
            admin_id: new_task.admin_id,
        };

        // 🥑 todo - publish/fire new task event/topic using redis 
        // ... 
        let publish_task_topic = events::redis::task::Register;


        match diesel::insert_into(tasks::table)
            .values(&task)
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

    pub async fn delete(job_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<usize, Result<HttpResponse, actix_web::Error>>{
        
        match diesel::delete(tasks.filter(tasks::id.eq(job_id)))
            .execute(connection)
            {
                Ok(_) => {
                
                    /* 
                        we must also delete the associated records from the users_tasks table 
                        since a task is deleted thus all the users who have done this task must
                        deleted from the users_tasks table too 
                    */
                    let deleted_rows = match diesel::delete(users_tasks.filter(users_tasks::task_id.eq(task_id)))
                    .execute(connection)
                    {
                        Ok(num_deleted) => num_deleted,
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
                    
                    Ok(deleted_rows)
                    
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

    pub async fn edit(new_task: EditTaskRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<TaskData, Result<HttpResponse, actix_web::Error>>{

        match diesel::update(tasks.find(new_task.task_id.to_owned()))
            .set(EditTask{
                /* 
                    task name and description are of type &str 
                    thus by borrowing new_task struct fields we
                    can convert them into &str 
                */
                task_name: &new_task.task_name, 
                task_description: &new_task.task_description,
                task_score: new_task.task_score
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
                            admin_id: updated_task.admin_id,
                            created_at: updated_task.created_at.to_string(),
                            updated_at: updated_task.updated_at.to_string(),
                        }
                    )
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

    pub async fn get_all_admin(owner_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<TaskData>, Result<HttpResponse, actix_web::Error>>{

        /* get the passed in admin info by its id */
        let user = match users::table
            .filter(users::id.eq(owner_id))
            .select(User::as_select())
            .get_result::<User>(connection)
            {
                Ok(single_user) => single_user,
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
                                admin_id: t.admin_id,
                                created_at: t.created_at.to_string(),
                                updated_at: t.updated_at.to_string(),
                            })
                            .collect::<Vec<TaskData>>()
                    )
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

    pub async fn get_all(connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<TaskData>, Result<HttpResponse, actix_web::Error>>{

        match tasks.load::<Task>(connection)
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
                            admin_id: t.admin_id,
                            created_at: t.created_at.to_string(),
                            updated_at: t.updated_at.to_string(),
                        })
                        .collect::<Vec<TaskData>>()
                )
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

}