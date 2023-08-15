

use crate::*;
use crate::misc::Response;
use crate::schema::users::dsl::*;
use crate::schema::users_deposits;
use crate::constants::*;
use crate::models::users::{User, UserData, UserRole};
use crate::schema::users_deposits::dsl::*;





/* 

    diesel migration generate users_deposits ---> create users_deposits migration sql files
    diesel migration run                     ---> apply sql files to db 
    diesel migration redo                    ---> drop tables 

*/


#[derive(Identifiable, Selectable, Queryable, Debug)]
#[diesel(table_name=users_deposits)]
pub struct UserDeposit { /* note that the ordering of fields must be the same as the table fields in up.sql */
    pub id: i32,
    pub payment_id: String,
    pub from_cid: String,
    pub recipient_cid: String,
    pub amount: i64,
    pub tx_signature: String,
    pub iat: chrono::NaiveDateTime
}

#[derive(Insertable, Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
#[diesel(table_name=users_deposits)]
pub struct NewUserDeposit{
    pub from_cid: String,
    pub recipient_cid: String,
    pub amount: i64,
    pub payment_id: String,
    pub tx_signature: String, /* this must be generated inside the client by signing the operation using the client private key */
    pub iat: chrono::NaiveDateTime // deposited at
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserDepositRequest{
    pub from_cid: String,
    pub recipient_cid: String,
    pub amount: i64,
    pub tx_signature: String, /* this must be generated inside the client by signing the operation using the client private key */
    pub iat: chrono::NaiveDateTime // deposited at
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct UserDepositData{
    pub id: i32,
    pub from_cid: String,
    pub recipient_cid: String,
    pub amount: i64,
    pub payment_id: String,
    pub signature: String, 
    pub iat: chrono::NaiveDateTime 
}

impl UserDeposit{

    pub async fn insert(user_deposit_request: NewUserDepositRequest, succ_payment_id: String, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserDepositData, PanelHttpResponse>{

        let new_user_deposit = NewUserDeposit{
            from_cid: user_deposit_request.from_cid,
            recipient_cid: user_deposit_request.recipient_cid,
            amount: user_deposit_request.amount,
            payment_id: succ_payment_id,
            tx_signature: user_deposit_request.tx_signature,
            iat: user_deposit_request.iat,
        };

        match diesel::insert_into(users_deposits)
            .values(&new_user_deposit)
            .returning(UserDeposit::as_returning())
            .get_result::<UserDeposit>(connection)
            {
                Ok(user_deposit) => {

                    Ok(UserDepositData{ 
                        id: user_deposit.id, 
                        from_cid: user_deposit.from_cid, 
                        recipient_cid: user_deposit.recipient_cid, 
                        amount: user_deposit.amount, 
                        signature: user_deposit.tx_signature,
                        payment_id: user_deposit.payment_id,
                        iat: user_deposit.iat 
                    })

                },
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

    pub async fn get_all_for(user_cid: String, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<UserDepositData>, PanelHttpResponse>{

        let user_deposits = users_deposits
            .filter(from_cid.eq(user_cid.clone()))
            .load::<UserDeposit>(connection);
            
        let Ok(deposits) = user_deposits else{
            let resp = Response{
                data: Some(user_cid.clone()),
                message: CID_HAS_NOT_DEPOSIT_YET,
                status: 404,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };

        Ok(
            deposits
                .into_iter()
                .map(|d| {
                    UserDepositData{
                        id: d.id,
                        from_cid: d.from_cid,
                        recipient_cid: d.recipient_cid,
                        amount: d.amount,
                        payment_id: d.payment_id,
                        signature: d.tx_signature,
                        iat: d.iat,
                    }
                }).collect::<Vec<UserDepositData>>()
        )


    }

    pub async fn get_all(connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<UserDepositData>, PanelHttpResponse>{

        let user_deposits = users_deposits
            .load::<UserDeposit>(connection);
            
        let Ok(deposits) = user_deposits else{
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: CID_HAS_NOT_DEPOSIT_YET,
                status: 404,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };

        Ok(
            deposits
                .into_iter()
                .map(|d| {
                    UserDepositData{
                        id: d.id,
                        from_cid: d.from_cid,
                        recipient_cid: d.recipient_cid,
                        amount: d.amount,
                        payment_id: d.payment_id,
                        signature: d.tx_signature,
                        iat: d.iat,
                    }
                }).collect::<Vec<UserDepositData>>()
        )


    }

}