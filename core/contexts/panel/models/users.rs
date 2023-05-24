

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

    pub fn generate_token(&self) -> &str{

        // generate jwt token from user_id
        // ...

        let token = "";
        token
    }

    pub fn hash_pswd(&self, pswd: &str) -> Result<String, argon2::Error>{
        let salt = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
        let salt_bytes = salt.as_bytes();
        let password_bytes = pswd.as_bytes();
        argon2::hash_encoded(password_bytes, salt_bytes, &argon2::Config::default())
    }

    pub fn verify_pswd(&self, raw_pswd: &str) -> Result<bool, argon2::Error>{
        let password_bytes = raw_pswd.as_bytes();
        Ok(argon2::verify_encoded(&self.pswd, password_bytes).unwrap())
    }

}