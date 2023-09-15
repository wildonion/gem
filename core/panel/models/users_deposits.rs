

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
    pub mint_tx_hash: String,
    pub nft_id: String,
    pub from_cid: String,
    pub recipient_screen_cid: String,
    pub is_claimed: bool,
    pub amount: i64,
    pub tx_signature: String,
    pub iat: chrono::NaiveDateTime
}

#[derive(Insertable, Clone, Debug, ToSchema, PartialEq)]
#[diesel(table_name=users_deposits)]
pub struct NewUserDeposit{
    pub from_cid: String,
    pub recipient_screen_cid: String,
    pub is_claimed: bool,
    pub amount: i64,
    pub nft_id: String,
    pub mint_tx_hash: String,
    /* 
        this must be generated inside the client by signing the whole 
        data body of this struct using the client private key 
    */
    pub tx_signature: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserDepositRequest{
    pub from_cid: String,
    pub recipient: String,
    pub amount: i64,
    /* 
        this must be generated inside the client by signing the whole 
        data body of this struct using the client private key 
    */
    pub tx_signature: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct DecodedSignedDepositData{
    pub from_cid: String,
    pub recipient_screen_cid: String,
    pub amount: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct UserDepositData{
    pub id: i32,
    pub from_cid: String,
    pub recipient_screen_cid: String,
    pub nft_id: String,
    pub is_claimed: bool,
    pub amount: i64,
    pub mint_tx_hash: String,
    pub signature: String, 
    pub iat: String
}

impl UserDeposit{

    pub async fn insert(user_deposit_request: NewUserDepositRequest, succ_mint_tx_hash: String, token_id: String, polygon_recipient_address: String, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserDepositData, PanelHttpResponse>{

        let new_user_deposit = NewUserDeposit{
            from_cid: user_deposit_request.from_cid,
            recipient_screen_cid: polygon_recipient_address,
            is_claimed: false,
            amount: user_deposit_request.amount,
            nft_id: token_id,
            mint_tx_hash: succ_mint_tx_hash,
            tx_signature: user_deposit_request.tx_signature,
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
                        recipient_screen_cid: user_deposit.recipient_screen_cid,
                        nft_id: user_deposit.nft_id.to_string(),
                        is_claimed: user_deposit.is_claimed,
                        amount: user_deposit.amount, 
                        signature: user_deposit.tx_signature,
                        mint_tx_hash: user_deposit.mint_tx_hash,
                        iat: user_deposit.iat.to_string()
                    })

                },
                Err(e) => {

                    let resp_err = &e.to_string();

                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserDeposit::insert");
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

    pub async fn find_by_id(deposit_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserDepositData, PanelHttpResponse>{

        let user_deposits = users_deposits
            .filter(users_deposits::id.eq(deposit_id))
            .first::<UserDeposit>(connection);
            
        let Ok(deposit) = user_deposits else{
            let resp = Response{
                data: Some(deposit_id),
                message: DEPOSIT_NOT_FOUND,
                status: 404,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };

        Ok(
            UserDepositData{ 
                id: deposit.id, 
                from_cid: deposit.from_cid, 
                recipient_screen_cid: deposit.recipient_screen_cid, 
                nft_id: deposit.nft_id.to_string(), 
                is_claimed: deposit.is_claimed,
                amount: deposit.amount, 
                mint_tx_hash: deposit.mint_tx_hash, 
                signature: deposit.tx_signature, 
                iat: deposit.iat.to_string()
            }
        )

    }

    pub async fn get_all_for(user_cid: String, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<UserDepositData>, PanelHttpResponse>{

        let user_deposits = users_deposits
            .filter(from_cid.eq(user_cid.clone()))
            .load::<UserDeposit>(connection);
            
        let Ok(deposits) = user_deposits else{
            let resp = Response{
                data: Some(user_cid.clone()),
                message: CID_HAS_NO_DEPOSIT_YET,
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
                        recipient_screen_cid: d.recipient_screen_cid,
                        is_claimed: d.is_claimed,
                        amount: d.amount,
                        nft_id: d.nft_id.to_string(),
                        mint_tx_hash: d.mint_tx_hash,
                        signature: d.tx_signature,
                        iat: d.iat.to_string(),
                    }
                }).collect::<Vec<UserDepositData>>()
        )


    }

    pub async fn set_claim(deposit_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserDepositData, PanelHttpResponse>{

        match diesel::update(users_deposits.find(deposit_id))
            .set(is_claimed.eq(true))
            .returning(UserDeposit::as_returning())
            .get_result(connection)
            {
            
                Ok(d) => {
                    Ok(
                        UserDepositData{
                            id: d.id,
                            from_cid: d.from_cid,
                            recipient_screen_cid: d.recipient_screen_cid,
                            is_claimed: d.is_claimed,
                            amount: d.amount,
                            nft_id: d.nft_id.to_string(),
                            mint_tx_hash: d.mint_tx_hash,
                            signature: d.tx_signature,
                            iat: d.iat.to_string(),
                        }
                    )

                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();

                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserDeposit::set_claim");
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

    pub async fn get_all(connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<UserDepositData>, PanelHttpResponse>{

        let user_deposits = users_deposits
            .load::<UserDeposit>(connection);
            
        let Ok(deposits) = user_deposits else{
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: CID_HAS_NO_DEPOSIT_YET,
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
                        recipient_screen_cid: d.recipient_screen_cid,
                        is_claimed: d.is_claimed,
                        amount: d.amount,
                        nft_id: d.nft_id.to_string(),
                        mint_tx_hash: d.mint_tx_hash,
                        signature: d.tx_signature,
                        iat: d.iat.to_string(),
                    }
                }).collect::<Vec<UserDepositData>>()
        )


    }


    pub async fn get_unclaimeds_for(user_screen_cid: String, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<UserDepositData>, PanelHttpResponse>{

        let user_deposits = users_deposits
            .filter(users_deposits::recipient_screen_cid.eq(user_screen_cid))
            .filter(users_deposits::is_claimed.eq(false))
            .load::<UserDeposit>(connection);
            
        let Ok(deposits) = user_deposits else{
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: RECIPIENT_HAS_NO_DEPOSIT_YET,
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
                        recipient_screen_cid: d.recipient_screen_cid,
                        is_claimed: d.is_claimed,
                        amount: d.amount,
                        nft_id: d.nft_id.to_string(),
                        mint_tx_hash: d.mint_tx_hash,
                        signature: d.tx_signature,
                        iat: d.iat.to_string(),
                    }
                }).collect::<Vec<UserDepositData>>()
        )


    }



}