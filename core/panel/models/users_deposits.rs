

use actix::Addr;
use crate::*;
use crate::events::publishers::action::{SingleUserNotif, NotifData, ActionType};
use crate::misc::{Response, Limit};
use crate::schema::users::dsl::*;
use crate::schema::users_deposits;
use crate::constants::*;
use crate::models::users::{User, UserData, UserRole};
use crate::schema::users_deposits::dsl::*;
use super::users::UserWalletInfoResponse;
use super::users_nfts::{UserNft, UserNftData};




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
    pub nft_img_url: String,
    pub from_cid: String,
    pub recipient_screen_cid: String,
    pub is_claimed: bool,
    pub amount: i64,
    pub tx_signature: String,
    pub iat: chrono::NaiveDateTime
}

#[derive(Insertable, Clone, Debug, PartialEq)]
#[diesel(table_name=users_deposits)]
pub struct NewUserDeposit{
    pub from_cid: String,
    pub recipient_screen_cid: String,
    pub is_claimed: bool,
    pub amount: i64,
    pub nft_id: String,
    pub nft_img_url: String,
    pub mint_tx_hash: String,
    pub tx_signature: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NewUserDepositRequest{
    pub from_cid: String,
    pub recipient: String,
    pub amount: i64,
    pub nft_img_url: String,
    pub nft_name: String,
    pub nft_desc: String,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserDepositData{
    pub id: i32,
    pub from_cid: String,
    pub recipient_screen_cid: String,
    pub nft_id: String,
    pub nft_img_url: String,
    pub is_claimed: bool,
    pub amount: i64,
    pub mint_tx_hash: String,
    pub signature: String, 
    pub iat: String
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserDepositDataWithWalletInfo{
    pub id: i32,
    pub from_wallet_info: UserWalletInfoResponse,
    pub recipient_wallet_info: UserWalletInfoResponse,
    pub nft_data: UserNftData,
    pub nft_img_url: String,
    pub is_claimed: bool,
    pub amount: i64,
    pub mint_tx_hash: String,
    pub signature: String, 
    pub iat: String
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserDeposit{

    pub async fn insert(user_deposit_request: NewUserDepositRequest, succ_mint_tx_hash: String, token_id: String, 
        polygon_recipient_address: String, nft_url: String, redis_actor: Addr<RedisActor>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserDepositData, PanelHttpResponse>{

        let depositor_screen_cid = &walletreq::evm::get_keccak256_from(user_deposit_request.clone().from_cid);
        let get_user = User::find_by_screen_cid(&depositor_screen_cid.clone(), connection).await;
        let Ok(user) = get_user else{

            let resp_err = get_user.unwrap_err();
            return Err(resp_err);
        };
            
        let get_recipient = User::find_by_screen_cid(&polygon_recipient_address.clone(), connection).await;
        let Ok(recipient) = get_recipient else{

            let resp_err = get_recipient.unwrap_err();
            return Err(resp_err);
        };

        let new_user_deposit = NewUserDeposit{
            from_cid: user_deposit_request.from_cid,
            recipient_screen_cid: polygon_recipient_address,
            is_claimed: false,
            amount: user_deposit_request.amount,
            nft_id: token_id,
            nft_img_url: nft_url,
            mint_tx_hash: succ_mint_tx_hash,
            tx_signature: user_deposit_request.tx_signature,
        };

        match diesel::insert_into(users_deposits)
            .values(&new_user_deposit)
            .returning(UserDeposit::as_returning())
            .get_result::<UserDeposit>(connection)
            {
                Ok(user_deposit) => {

                    let ud = UserDepositData{ 
                        id: user_deposit.id, 
                        from_cid: user_deposit.from_cid, 
                        recipient_screen_cid: user_deposit.recipient_screen_cid,
                        nft_id: user_deposit.nft_id.to_string(),
                        nft_img_url: user_deposit.nft_img_url.to_string(),
                        is_claimed: user_deposit.is_claimed,
                        amount: user_deposit.amount, 
                        signature: user_deposit.tx_signature,
                        mint_tx_hash: user_deposit.mint_tx_hash,
                        iat: user_deposit.iat.to_string()
                    };

                    /** -------------------------------------------------------------------- */
                    /** ----------------- publish new event data to `on_user_action` channel */
                    /** -------------------------------------------------------------------- */
                    // if the actioner is the user himself we'll notify user with something like:
                    // u've just done that action!
                    let actioner_wallet_info = UserWalletInfoResponse{
                        username: user.username,
                        avatar: user.avatar,
                        bio: user.bio,
                        banner: user.banner,
                        mail: user.mail,
                        screen_cid: user.screen_cid,
                        extra: user.extra,
                        stars: user.stars,
                        created_at: user.created_at.to_string(),
                    };
                    let user_wallet_info = UserWalletInfoResponse{
                        username: recipient.username,
                        avatar: recipient.avatar,
                        bio: recipient.bio,
                        banner: recipient.banner,
                        mail: recipient.mail,
                        screen_cid: recipient.screen_cid,
                        extra: recipient.extra,
                        stars: recipient.stars,
                        created_at: recipient.created_at.to_string(),
                    };
                    let user_notif_info = SingleUserNotif{
                        wallet_info: user_wallet_info,
                        notif: NotifData{
                            actioner_wallet_info,
                            fired_at: Some(chrono::Local::now().timestamp()),
                            action_type: ActionType::DepositGiftCard,
                            action_data: serde_json::to_value(ud.clone()).unwrap()
                        }
                    };
                    let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                    events::publishers::action::emit(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;

                    Ok(
                        ud
                    )

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
                        status: 500,
                        is_error: true
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
                is_error: true
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
                nft_img_url: deposit.nft_img_url.to_string(),
                is_claimed: deposit.is_claimed,
                amount: deposit.amount, 
                mint_tx_hash: deposit.mint_tx_hash, 
                signature: deposit.tx_signature, 
                iat: deposit.iat.to_string()
            }
        )

    }

    pub async fn get_all_for(user_cid: String, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserDepositDataWithWalletInfo>, PanelHttpResponse>{

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

        let user_deposits = users_deposits
            .order(iat.desc())
            .filter(from_cid.eq(user_cid.clone()))
            .offset(from)
            .limit((to - from) + 1)
            .load::<UserDeposit>(connection);
            
        let Ok(deposits) = user_deposits else{
            let resp = Response{
                data: Some(user_cid.clone()),
                message: CID_HAS_NO_DEPOSIT_YET,
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
                    UserDepositDataWithWalletInfo{
                        id: d.id,
                        from_wallet_info: {
                            let from_screen_cid = walletreq::evm::get_keccak256_from(user_cid.clone());
                            let user = User::find_by_screen_cid_none_async(&from_screen_cid, connection).unwrap();
                            UserWalletInfoResponse{
                                username: user.username,
                                avatar: user.avatar,
                                bio: user.bio,
                                banner: user.banner,
                                mail: user.mail,
                                screen_cid: user.screen_cid,
                                extra: user.extra,
                                stars: user.stars,
                                created_at: user.created_at.to_string(),
                            }
                        },
                        recipient_wallet_info: {
                            {
                                let user = User::find_by_screen_cid_none_async(&d.recipient_screen_cid, connection).unwrap();
                                UserWalletInfoResponse{
                                    username: user.username,
                                    avatar: user.avatar,
                                    bio: user.bio,
                                    banner: user.banner,
                                    mail: user.mail,
                                    screen_cid: user.screen_cid,
                                    extra: user.extra,
                                    stars: user.stars,
                                    created_at: user.created_at.to_string(),
                                }
                            }
                        },
                        is_claimed: d.is_claimed,
                        amount: d.amount,
                        nft_data: {
                            let nft = UserNft::find_by_onchain_id_none_async(&d.nft_id, connection).unwrap();
                            nft
                        },
                        nft_img_url: d.nft_img_url.to_string(),
                        mint_tx_hash: d.mint_tx_hash,
                        signature: d.tx_signature,
                        iat: d.iat.to_string(),
                    }
                }).collect::<Vec<UserDepositDataWithWalletInfo>>()
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
                            nft_img_url: d.nft_img_url.to_string(),
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
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );
                }
            
            }
    }

    pub async fn get_all(limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserDepositDataWithWalletInfo>, PanelHttpResponse>{
        
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

        let user_deposits = users_deposits
            .order(iat.desc())
            .offset(from)
            .limit((to - from) + 1)
            .load::<UserDeposit>(connection);
            
        let Ok(deposits) = user_deposits else{
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: NO_DEPOSITS_YET,
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
                    UserDepositDataWithWalletInfo{
                        id: d.id,
                        from_wallet_info: {
                            let from_screen_cid = walletreq::evm::get_keccak256_from(d.from_cid.clone());
                            let user = User::find_by_screen_cid_none_async(&from_screen_cid, connection).unwrap();
                            UserWalletInfoResponse{
                                username: user.username,
                                avatar: user.avatar,
                                bio: user.bio,
                                banner: user.banner,
                                mail: user.mail,
                                screen_cid: user.screen_cid,
                                extra: user.extra,
                                stars: user.stars,
                                created_at: user.created_at.to_string(),
                            }
                        },
                        recipient_wallet_info: {
                            {
                                let user = User::find_by_screen_cid_none_async(&d.recipient_screen_cid, connection).unwrap();
                                UserWalletInfoResponse{
                                    username: user.username,
                                    avatar: user.avatar,
                                    bio: user.bio,
                                    banner: user.banner,
                                    mail: user.mail,
                                    screen_cid: user.screen_cid,
                                    extra: user.extra,
                                    stars: user.stars,
                                    created_at: user.created_at.to_string(),
                                }
                            }
                        },
                        is_claimed: d.is_claimed,
                        amount: d.amount,
                        nft_data: {
                            let nft = UserNft::find_by_onchain_id_none_async(&d.nft_id, connection).unwrap();
                            nft
                        },
                        nft_img_url: d.nft_img_url.to_string(),
                        mint_tx_hash: d.mint_tx_hash,
                        signature: d.tx_signature,
                        iat: d.iat.to_string(),
                    }
                }).collect::<Vec<UserDepositDataWithWalletInfo>>()
        )


    }


    pub async fn get_unclaimeds_for(user_screen_cid: String, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserDepositDataWithWalletInfo>, PanelHttpResponse>{
        
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

        let user_deposits = users_deposits
            .order(iat.desc())
            .filter(users_deposits::recipient_screen_cid.eq(user_screen_cid))
            .filter(users_deposits::is_claimed.eq(false))
            .offset(from)
            .limit((to - from) + 1)
            .load::<UserDeposit>(connection);
            
        let Ok(deposits) = user_deposits else{
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: RECIPIENT_HAS_NO_DEPOSIT_YET,
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
                    UserDepositDataWithWalletInfo{
                        id: d.id,
                        from_wallet_info: {
                            let from_screen_cid = walletreq::evm::get_keccak256_from(d.from_cid.clone());
                            let user = User::find_by_screen_cid_none_async(&from_screen_cid, connection).unwrap();
                            UserWalletInfoResponse{
                                username: user.username,
                                avatar: user.avatar,
                                bio: user.bio,
                                banner: user.banner,
                                mail: user.mail,
                                screen_cid: user.screen_cid,
                                extra: user.extra,
                                stars: user.stars,
                                created_at: user.created_at.to_string(),
                            }
                        },
                        recipient_wallet_info: {
                            {
                                let user = User::find_by_screen_cid_none_async(&d.recipient_screen_cid, connection).unwrap();
                                UserWalletInfoResponse{
                                    username: user.username,
                                    avatar: user.avatar,
                                    bio: user.bio,
                                    banner: user.banner,
                                    mail: user.mail,
                                    screen_cid: user.screen_cid,
                                    extra: user.extra,
                                    stars: user.stars,
                                    created_at: user.created_at.to_string(),
                                }
                            }
                        },
                        is_claimed: d.is_claimed,
                        amount: d.amount,
                        nft_data: {
                            let nft = UserNft::find_by_onchain_id_none_async(&d.nft_id, connection).unwrap();
                            nft
                        },
                        nft_img_url: d.nft_img_url.to_string(),
                        mint_tx_hash: d.mint_tx_hash,
                        signature: d.tx_signature,
                        iat: d.iat.to_string(),
                    }
                }).collect::<Vec<UserDepositDataWithWalletInfo>>()
        )


    }



}