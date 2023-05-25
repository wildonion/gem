


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

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User, foreign_key=admin_id))]
#[diesel(table_name=tasks)]
pub struct Task{
    pub id: i32,
    pub task_name: String,
    pub task_description: String,
    pub task_score: i32,
    pub admin_id: i32, // amdin id who has defined the tasks
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct NewTaskRequest{
    pub task_name: String,
    pub task_description: String,
    pub task_score: i32,
    pub admin_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name=tasks)]
pub struct NewTask<'t>{
    pub task_name: &'t str,
    pub task_description: &'t str,
    pub task_score: i32,
    pub admin_id: i32
}

impl Task{

}