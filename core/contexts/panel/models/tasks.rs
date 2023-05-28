


use crate::*;
use crate::models::users::User;
use crate::misc::Response;
use crate::schema::{tasks, users};
use crate::schema::tasks::dsl::*;
use crate::constants::*;



/* 

    diesel migration generate tasks ---> create tasks migration sql files
    diesel migration run            ---> apply sql files to db 
    diesel migration redo           ---> drop tables 

*/

#[derive(Queryable, Selectable, Serialize, Deserialize, Identifiable, Associations, Debug, PartialEq)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewTaskRequest{
    pub task_name: String,
    pub task_description: String,
    pub task_score: i32,
    pub admin_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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


    pub async fn insert(new_task: NewTaskRequest, 
        redis_connection: RedisConnection, 
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

        // publish/fire new task event/topic using redis 
        // ... 

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

    pub async fn delete(task_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<usize, Result<HttpResponse, actix_web::Error>>{
        
        match diesel::delete(tasks.filter(tasks::id.eq(task_id)))
            .execute(connection)
            {
                Ok(num_deleted) => Ok(num_deleted),
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

    pub async fn edit(new_task: EditTaskRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Task, Result<HttpResponse, actix_web::Error>>{

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
                Ok(updated_task) => Ok(updated_task),
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

    pub async fn get_all_admin(owner_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<Task>, Result<HttpResponse, actix_web::Error>>{

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
                Ok(admin_tasks) => Ok(admin_tasks),
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