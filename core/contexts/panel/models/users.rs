

use crate::*;
use crate::schema::users;



/* 

    diesel migration generate users ---> create users migration sql files
    diesel migration run            ---> apply sql files to db 
    diesel migration redo           ---> drop tables 

*/
#[derive(Queryable,)]
pub struct User{
    pub id: i32,
    pub username: String,
    pub twitter_username: String,
    pub facebook_username: String,
    pub discord_username: String,
    pub wallet_address: String,
    pub user_role: UserRole,
    pub pswd: String,
    pub last_login: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[derive(diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::Userrole"]
pub enum UserRole{
    Admin,
    User,
    Dev
}

#[derive(Insertable)]
#[diesel(table_name=users)]
pub struct NewUser<'l> {
    pub username: &'l str,
    pub twitter_username: &'l str,
    pub facebook_username: &'l str,
    pub discord_username: &'l str,
    pub wallet_address: &'l str,
    pub user_role: UserRole,
    pub pswd: &'l str,
    pub last_login: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}


impl User{

    pub fn get_token(&self) -> &str{
        let token = "";
        token
    }

}