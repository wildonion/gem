


use crate::*;
use crate::misc::Response;
use crate::schema::users;
use crate::schema::users::dsl::*;
use crate::schema::{users_withdrawals, users_deposits::dsl::users_deposits, users_deposits::id as users_deposits_id};
use crate::constants::*;
use crate::models::users::{User, UserData, UserRole};
use crate::schema::users_withdrawals::dsl::*;
use super::users_deposits::UserDeposit;
use crate::schema::users_mails::*;
use crate::schema::users_mails;
use crate::schema::users_mails::dsl::*;




/* 

    diesel migration generate users_mail ---> create users_mail migration sql files
    diesel migration run                 ---> apply sql files to db 
    diesel migration redo                ---> drop tables 

*/
#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(User))]
#[diesel(table_name=users_mails)]
#[diesel(primary_key(user_id))]
pub struct UserMail {
    pub user_id: i32,
    pub mail: String,
    pub code: String,
    pub exp: chrono::NaiveDateTime, /* expires at */
    pub vat: chrono::NaiveDateTime /* verified at */
}

#[derive(Insertable)]
#[diesel(table_name=users_mails)]
pub struct NewUserMail<'s>{
    pub user_id: i32,
    pub mail: &'s str,
    pub code: &'s str,
}

impl UserMail{

    pub async fn save(user_mail: &str, receiver_id: i32, random_code: String, two_mins_later: chrono::NaiveDateTime,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Self, PanelHttpResponse>{

        
        let single_user = users::table
            .filter(users::id.eq(receiver_id))
            .first::<User>(connection);

        let Ok(user) = single_user else{

            let resp = Response{
                data: Some(receiver_id),
                message: USER_NOT_FOUND,
                status: 404
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
            
        };

        match diesel::insert_into(users_mails::table)
            .values(&NewUserMail{
                user_id: receiver_id,
                mail: user_mail,
                code: &random_code
            })
            .execute(connection)
            {
                Ok(affected_row) => Ok(
                    Self{ 
                        user_id: receiver_id, 
                        mail: user_mail.to_string(), 
                        code: random_code.to_string(), 
                        exp: two_mins_later, 
                        vat: chrono::Local::now().naive_local()
                    }
                ),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)));
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