


use crate::*;
use crate::misc::Response;
use crate::schema::users;
use crate::schema::users_phones;
use crate::schema::users::dsl::*;
use crate::schema::{users_withdrawals, users_deposits::dsl::users_deposits, users_deposits::id as users_deposits_id};
use crate::constants::*;
use crate::models::users::{User, UserData, UserRole};
use crate::schema::users_withdrawals::dsl::*;
use super::users_deposits::UserDeposit;
use crate::schema::users_phones::*;
use crate::schema::users_phones::dsl::*;




/* 

    diesel migration generate users_phones ---> create users_phone migration sql files
    diesel migration run                   ---> apply sql files to db 
    diesel migration redo                  ---> drop tables 

*/
#[derive(Identifiable, Selectable, Queryable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=users_phones)]
pub struct UserPhone {
    pub id: i32,
    pub user_id: i32,
    pub phone: String,
    pub code: String,
    pub exp: i64, /* expires at */
    pub vat: i64 /* verified at */
}

#[derive(Insertable)]
#[diesel(table_name=users_phones)]
pub struct NewUserPhone<'s>{
    pub user_id: i32,
    pub phone: &'s str,
    pub code: &'s str,
    pub exp: i64
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserPhone{

    pub async fn save(user_phone: &str, receiver_id: i32, random_code: String, two_mins_later: chrono::NaiveDateTime,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<usize, PanelHttpResponse>{

        
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

        let new_user_phone = NewUserPhone{
            user_id: receiver_id,
            phone: user_phone,
            code: &random_code,
            exp: two_mins_later.timestamp_millis()
        };
        match diesel::insert_into(users_phones::table)
            .values(&new_user_phone)
            .execute(connection)
            {
                Ok(affected_row) => Ok(affected_row),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserPhone::save");
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


    pub async fn update_vat(user_phone_id: i32, user_vat: i64, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Self, PanelHttpResponse>{

        match diesel::update(users_phones.find(user_phone_id))
                .set(users_phones::vat.eq(user_vat))
                .returning(users_phones::all_columns)
                .get_result::<UserPhone>(connection)
                {
                    Ok(updated_user_phone) => Ok(updated_user_phone),
                    Err(e) => {
                        
                        let resp_err = &e.to_string();

                        /* custom error handler */
                        use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                            
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserPhone::update_vat");
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