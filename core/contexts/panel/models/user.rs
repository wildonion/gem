





use crate::*;
use crate::schema::users;



/* 

    diesel migration generate users
    diesel migration run
    diesel migration redo

*/
#[derive(Queryable)]
pub struct User<'l>{
    pub id: u32,
    pub twitter_username: &'l str,
    pub wallet_address: &'l str,
    pub user_role: UserRole,
    pub last_login: chrono::NaiveDate,
    pub created_at: chrono::NaiveDate,
    pub updated_at: chrono::NaiveDate,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum UserRole{
    Admin,
    User,
    Dev
}

#[derive(Insertable)]
#[diesel(table_name=users)]
pub struct NewUser<'l> {
    pub twitter_username: &'l str,
    pub wallet_address: &'l str,
    pub user_role: UserRole,
    pub last_login: chrono::NaiveDate,
}
