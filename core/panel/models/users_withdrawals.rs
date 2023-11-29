

use crate::*;
use crate::misc::{Response, Limit};
use crate::schema::users::dsl::*;
use crate::schema::{users_withdrawals, users_deposits::dsl::users_deposits, users_deposits::id as users_deposits_id};
use crate::constants::*;
use crate::models::users::{User, UserData, UserRole};
use crate::schema::users_withdrawals::dsl::*;
use super::users_deposits::UserDeposit;





/* 

    diesel migration generate users_withdrawals ---> create users_withdrawals migration sql files
    diesel migration run                        ---> apply sql files to db 
    diesel migration redo                       ---> drop tables 

*/

#[derive(Identifiable, Selectable, Queryable, Debug, Serialize, Deserialize)]
#[diesel(table_name=users_withdrawals)]
pub struct UserWithdrawal { /* note that the ordering of fields must be the same as the table fields in up.sql */
    pub id: i32,
    pub deposit_id: i32,
    pub transfer_tx_hash: String,
    pub recipient_cid: String,
    pub tx_signature: String,
    pub wat: chrono::NaiveDateTime
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserWithdrawRequest{
    pub deposit_id: i32,
    pub recipient_cid: String,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Insertable, Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
#[diesel(table_name=users_withdrawals)]
pub struct NewUserWithdrawal{
    pub deposit_id: i32,
    pub recipient_cid: String,
    pub transfer_tx_hash: String,
    pub tx_signature: String
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct UserWithdrawalData{
    pub id: i32,
    pub deposit_id: i32,
    pub transfer_tx_hash: String,
    pub recipient_cid: String,
    pub signature: String,
    pub wat: chrono::NaiveDateTime
}


/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserWithdrawal{

    pub async fn insert(user_withdraw_request: NewUserWithdrawRequest, succ_transfer_tx_hash: String, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserWithdrawalData, PanelHttpResponse>{

        let new_user_withdrawal = NewUserWithdrawal{
            recipient_cid: user_withdraw_request.recipient_cid.clone(),
            deposit_id: user_withdraw_request.deposit_id,
            transfer_tx_hash: succ_transfer_tx_hash,
            tx_signature: user_withdraw_request.tx_signature
        };

        let get_user_deposit = users_deposits
            .filter(users_deposits_id.eq(user_withdraw_request.deposit_id))
            .first::<UserDeposit>(connection);

        let Ok(user_deposit) = get_user_deposit else{

            let resp = Response::<'_, i32>{
                data: Some(user_withdraw_request.deposit_id),
                message: DEPOSITED_NOT_FOUND,
                status: 404,
                is_error: true
            };

            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };


        let get_user_withdrawal = users_withdrawals
            .filter(recipient_cid.eq(user_withdraw_request.recipient_cid.clone()))
            .filter(deposit_id.eq(user_withdraw_request.deposit_id))
            .first::<UserWithdrawal>(connection);

        match get_user_withdrawal{
            Ok(user_withdrawal) => {

                let resp = Response::<'_, UserWithdrawal>{
                    data: Some(user_withdrawal),
                    message: ALREADY_WITHDRAWN,
                    status: 302,
                    is_error: false
                };
    
                return Err(
                    Ok(HttpResponse::Found().json(resp))
                );

            },
            Err(e) => {
                /* brand new withdrawal object */
                match diesel::insert_into(users_withdrawals)
                    .values(&new_user_withdrawal)
                    .returning(UserWithdrawal::as_returning())
                    .get_result::<UserWithdrawal>(connection)
                    {
                        Ok(user_withdrawal) => {

                            /* updating the is_claimed field inside the users_deposits table for this deposit */
                            let get_updated_user_deposit = UserDeposit::set_claim(user_withdrawal.deposit_id, connection).await;
                            let Ok(_) = get_updated_user_deposit else{
                                /* let client know about the result of can't updating the field using actix http response */
                                let error_resp = get_updated_user_deposit.unwrap_err();
                                return Err(error_resp);
                            };

                            Ok(UserWithdrawalData{ 
                                id: user_withdrawal.id, 
                                recipient_cid: user_withdrawal.recipient_cid,
                                signature: user_withdrawal.tx_signature,
                                deposit_id: user_withdraw_request.deposit_id,
                                transfer_tx_hash: user_withdrawal.transfer_tx_hash,
                                wat: user_withdrawal.wat 
                            })

                        },
                        Err(e) => {

                            let resp_err = &e.to_string();

                            /* custom error handler */
                            use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                            
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec();  
                            let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserWithdrawal::insert");
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

    }

    pub async fn get_all_for(withdrawer_cid: String, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<UserWithdrawalData>, PanelHttpResponse>{

        let from = limit.from.unwrap_or(0);
        let to = limit.to.unwrap_or(10);

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        let user_withdrawals = users_withdrawals
            .order(wat.desc())
            .offset(from)
            .limit((to - from) + 1)
            .filter(recipient_cid.eq(withdrawer_cid.clone()))
            .load::<UserWithdrawal>(connection);
            
        let Ok(deposits) = user_withdrawals else{
            let resp = Response{
                data: Some(withdrawer_cid.clone()),
                message: CID_HAS_NO_WITHDRAWAL_YET,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };

        Ok(
            deposits
                .into_iter()
                .map(|d| {
                    UserWithdrawalData{
                        id: d.id,
                        recipient_cid: d.recipient_cid,
                        transfer_tx_hash: d.transfer_tx_hash,
                        deposit_id: d.deposit_id,
                        signature: d.tx_signature,
                        wat: d.wat,
                    }
                }).collect::<Vec<UserWithdrawalData>>()
        )


    }

    pub async fn get_all(limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<UserWithdrawalData>, PanelHttpResponse>{

        let from = limit.from.unwrap_or(0);
        let to = limit.to.unwrap_or(10);

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        let user_withdrawals = users_withdrawals
            .order(wat.desc())
            .offset(from)
            .limit((to - from) + 1)
            .load::<UserWithdrawal>(connection);
            
        let Ok(deposits) = user_withdrawals else{
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: NO_WITHDRAWAL_YET,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };

        Ok(
            deposits
                .into_iter()
                .map(|d| {
                    UserWithdrawalData{
                        id: d.id,
                        recipient_cid: d.recipient_cid,
                        deposit_id: d.deposit_id,
                        transfer_tx_hash: d.transfer_tx_hash,
                        signature: d.tx_signature,
                        wat: d.wat,
                    }
                }).collect::<Vec<UserWithdrawalData>>()
        )


    }

}