


use crate::*;
use crate::helpers::misc::Response;
use crate::schema::users;
use crate::schema::users_mails;
use crate::schema::users::dsl::*;
use crate::schema::{users_withdrawals, users_deposits::dsl::users_deposits, users_deposits::id as users_deposits_id};
use crate::constants::*;
use crate::models::users::{User, UserData, UserRole};
use crate::schema::users_withdrawals::dsl::*;
use super::users_deposits::UserDeposit;
use crate::schema::users_mails::*;
use crate::schema::users_mails::dsl::*;




/* 

    diesel migration generate users_mails ---> create users_mails migration sql files
    diesel migration run                  ---> apply sql files to db 
    diesel migration redo                 ---> drop tables 

*/
#[derive(Identifiable, Selectable, Queryable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=users_mails)]
pub struct UserMail {
    pub id: i32,
    pub user_id: i32,
    pub mail: String,
    pub code: String,
    pub exp: i64, /* expires at */
    pub vat: i64 /* verified at */
}

#[derive(Insertable)]
#[diesel(table_name=users_mails)]
pub struct NewUserMail<'s>{
    pub user_id: i32,
    pub mail: &'s str,
    pub code: &'s str,
    pub exp: i64
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserMail{

    pub async fn save(user_mail: &str, receiver_id: i32, random_code: String, five_mins_later: chrono::NaiveDateTime,
        connection: &mut DbPoolConnection) -> Result<usize, PanelHttpResponse>{

        
        let single_user = users::table
            .filter(users::id.eq(receiver_id))
            .first::<User>(connection);

        let Ok(user) = single_user else{

            let resp = Response{
                data: Some(receiver_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
            
        };

        let new_user_mail = NewUserMail{
            user_id: receiver_id,
            mail: user_mail,
            code: &random_code,
            exp: five_mins_later.timestamp_millis()
        };
        match diesel::insert_into(users_mails::table)
            .values(&new_user_mail)
            .execute(connection)
            {
                Ok(affected_row) => Ok(affected_row),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserMail::save");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }


    }


    pub async fn update_vat(user_mail_id: i32, user_vat: i64, 
        connection: &mut DbPoolConnection) -> Result<Self, PanelHttpResponse>{

        match diesel::update(users_mails.find(user_mail_id))
                .set(users_mails::vat.eq(user_vat))
                .returning(users_mails::all_columns)
                .get_result::<UserMail>(connection)
                {
                    Ok(updated_user_mail) => Ok(updated_user_mail),
                    Err(e) => {
                        
                        let resp_err = &e.to_string();

                        /* custom error handler */
                        use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                            
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserMail::update_vat");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        let resp = Response::<&[u8]>{
                            data: Some(&[]),
                            message: resp_err,
                            status: 500,
                            is_error: true
                        };
                        return Err(
                            Ok(HttpResponse::InternalServerError().json(resp))
                        );

                    }
                }


    }

}