


use std::time::{SystemTime, UNIX_EPOCH};
use actix::Addr;
use actix_web::web::Query;
use chrono::NaiveDateTime;
use crate::adapters::nftport;
use crate::constants::{COLLECTION_NOT_FOUND_FOR, INVALID_QUERY_LIMIT, GALLERY_NOT_OWNED_BY, CANT_GET_CONTRACT_ADDRESS, USER_NOT_FOUND, USER_SCREEN_CID_NOT_FOUND, COLLECTION_UPLOAD_PATH, UNSUPPORTED_FILE_TYPE, TOO_LARGE_FILE_SIZE, STORAGE_IO_ERROR_CODE, COLLECTION_NOT_OWNED_BY, CANT_CREATE_COLLECTION_ONCHAIN, INVALID_CONTRACT_TX_HASH, CANT_UPDATE_COLLECTION_ONCHAIN, COLLECTION_NOT_FOUND_FOR_CONTRACT, USER_CLP_EVENT_NOT_FOUND_ANY, USER_CLP_EVENT_NOT_FOUND};
use crate::helpers::misc::{Response, Limit};
use crate::{*, constants::COLLECTION_NOT_FOUND_OF};
use self::constants::CLP_EVENT_NOT_FOUND;

use super::clp_events::ClpEventData;
use super::users::{User, UserData, UserRole};
use super::users_galleries::{UserPrivateGalleryData, UserPrivateGallery, UpdateUserPrivateGallery, UpdateUserPrivateGalleryRequest};
use super::users_nfts::UserNftData;
use crate::schema::users_clps::dsl::*;
use crate::schema::clp_events::dsl::*;
use crate::schema::users::dsl::*;
use crate::schema::{users_clps, clp_events, users};
use crate::models::clp_events::ClpEvent;


/* 

    in order this table works correctly clp_events must be initialized first
    since there is a reference as fk to the pk of clp_events and users

    diesel migration generate users_clps        ---> create users_clps migration sql files
    diesel migration run                        ---> apply sql files to db 
    diesel migration redo                       ---> drop tables 

*/
#[derive(Identifiable, Selectable, Queryable, Associations, Debug, Serialize, Deserialize)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(ClpEvent))]
#[diesel(table_name=users_clps)]
pub struct UserClp{
    pub id: i32,
    pub clp_event_id: i32,
    pub user_id: i32,
    pub entry_amount: Option<i64>,
    pub registered_at: chrono::NaiveDateTime,
    pub joined_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct ClpEventsPerUser{
    pub user: UserData,
    pub clp_events: Vec<ClpEventData>
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct RegisterUserClpEventRequest{
    pub participant_cid: String,
    pub clp_event_id: i32,
    pub entry_amount: Option<i64>,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct CancelUserClpEventRequest{
    pub participant_cid: String,
    pub clp_event_id: i32,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
#[derive(Insertable)]
#[diesel(table_name=users_clps)]
pub struct InsertNewUserClp{
    pub clp_event_id: i32,
    pub user_id: i32,
    pub entry_amount: Option<i64>,
}


impl UserClp{

    pub async fn cancel_reservation(participant_id: i32, event_id: i32, redis_client: redis::Client,
        redis_actor: Addr<RedisActor>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>
        ) -> Result<UserData, PanelHttpResponse>{
        
        match Self::delete_by_participant_and_event_id(participant_id, event_id, connection).await{
            Ok(num_deleted) => {
                
                let get_user = User::find_by_id(participant_id, connection).await;
                let Ok(user) = get_user else{
                    let err_resp = get_user.unwrap_err();
                    return Err(err_resp);
                };

                // if we have a deleted row 
                if num_deleted > 0 {

                    // payback the participant with the entrance_fee
                    let get_user_clp_event = Self::find_by_participant_and_event_id(participant_id, event_id, connection).await;
                    let Ok(user_clp_event) = get_user_clp_event else{
                        let err_resp = get_user_clp_event.unwrap_err();
                        return Err(err_resp);
                    };

                    let new_balance = user.balance.unwrap() + user_clp_event.entry_amount.unwrap();
                    let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actor, connection).await;
                    let Ok(updated_user_data) = update_user_balance else{

                        let err_resp = update_user_balance.unwrap_err();
                        return Err(err_resp);
                        
                    };

                    Ok(updated_user_data)

                } else{

                    Ok(
                        UserData{ 
                            id: user.id, 
                            region: user.region.clone(),
                            username: user.username, 
                            bio: user.bio.clone(),
                            avatar: user.avatar.clone(),
                            banner: user.banner.clone(),
                            wallet_background: user.wallet_background.clone(),
                            activity_code: user.activity_code,
                            twitter_username: user.twitter_username, 
                            facebook_username: user.facebook_username, 
                            discord_username: user.discord_username, 
                            identifier: user.identifier, 
                            user_role: {
                                match user.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: user.token_time,
                            balance: user.balance,
                            last_login: { 
                                if user.last_login.is_some(){
                                    Some(user.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: user.created_at.to_string(),
                            updated_at: user.updated_at.to_string(),
                            mail: user.mail,
                            google_id: user.google_id,
                            microsoft_id: user.microsoft_id,
                            is_mail_verified: user.is_mail_verified,
                            is_phone_verified: user.is_phone_verified,
                            phone_number: user.phone_number,
                            paypal_id: user.paypal_id,
                            account_number: user.account_number,
                            device_id: user.device_id,
                            social_id: user.social_id,
                            cid: user.cid,
                            screen_cid: user.screen_cid,
                            snowflake_id: user.snowflake_id,
                            stars: user.stars,
                            extra: user.extra,
                        }
                    )
                }
            },
            Err(resp) => Err(resp)
        }

    }

    pub async fn insert(entrance_fee: i64, participant_id: i32, event_id: i32, redis_client: RedisClient, redis_actor: Addr<RedisActor>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<UserClp, PanelHttpResponse>{

        let get_user = User::find_by_id(participant_id, connection).await;
        let Ok(user) = get_user else{
            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };

        let new_balance = user.balance.unwrap() - entrance_fee;
        let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actor.clone(), connection).await;
        let Ok(updated_user_balance_data) = update_user_balance else{

            let err_resp = update_user_balance.unwrap_err();
            return Err(err_resp);
            
        };

        match diesel::insert_into(users_clps)
            .values(&InsertNewUserClp{
                clp_event_id: event_id,
                user_id: participant_id,
                entry_amount: Some(entrance_fee),
            })
            .returning(UserClp::as_returning())
            .get_result::<UserClp>(connection)
            {
                Ok(user_clp) => {

                    Ok(user_clp)

                },
                Err(e) => {

                    let new_balance = updated_user_balance_data.balance.unwrap() - entrance_fee;
                    let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actor, connection).await;
                    let Ok(updated_user_balance_data) = update_user_balance else{

                        let err_resp = update_user_balance.unwrap_err();
                        return Err(err_resp);
                        
                    };

                    let resp_err = &e.to_string();

                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserClp::insert");
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

    pub async fn get_all_users_clps(connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<UserClp>, PanelHttpResponse>{

        let users_clps_ = users_clps
            .order(users_clps::registered_at.desc())
            .load::<UserClp>(connection);
            
        let Ok(all_users_clps) = users_clps_ else{
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: USER_CLP_EVENT_NOT_FOUND_ANY,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };

        Ok(
            all_users_clps
        )

    }

    pub async fn update_joined_at(participant_id: i32, event_id: i32,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<UserClp, PanelHttpResponse>{

        let get_user_clp = Self::find_by_participant_and_event_id(participant_id, event_id, connection).await;
        let Ok(single_user_clp) = get_user_clp else{
            let err_resp = get_user_clp.unwrap_err();
            return Err(err_resp);
        };

        let now = chrono::Local::now().naive_local();
        match diesel::update(users_clps.find(single_user_clp.id))
            .set(joined_at.eq(now))
            .returning(UserClp::as_returning())
            .get_result(connection)
        {
            Ok(updated_user_clp) => Ok(updated_user_clp),
            Err(err) => {
                
                let resp_err = &err.to_string();

                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &err.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(err)), "UserClp::update_joined_at");
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

    pub async fn find_by_participant_and_event_id(participant_id: i32, event_id: i32,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<UserClp, PanelHttpResponse>{

        let single_user_clp = users_clps
            .filter(users_clps::user_id.eq(participant_id))
            .filter(users_clps::clp_event_id.eq(event_id))
            .first::<UserClp>(connection);
                        
        let Ok(user_clp) = single_user_clp else{
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: USER_CLP_EVENT_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user_clp)
        
    }

    pub async fn delete_by_participant_and_event_id(participant_id: i32, event_id: i32,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<usize, PanelHttpResponse>{

        match diesel::delete(users_clps
            .filter(users_clps::user_id.eq(participant_id)))
            .filter(users_clps::clp_event_id.eq(event_id))
            .execute(connection)
            {
                Ok(num_deleted) => Ok(num_deleted),
                Err(e) => {

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: &e.to_string(),
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(
                            HttpResponse::InternalServerError().json(resp)
                        )
                    );

                }
            }
        
    }
    
    pub async fn get_all_users_in_clp_event(event_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<User>, PanelHttpResponse>{

        
        let get_event = ClpEvent::find_by_id(event_id, connection).await;
        let Ok(event) = get_event else{
            let err_resp = get_event.unwrap_err();
            return Err(err_resp);
        };

        // trying to get all users in the found event
        match UserClp::belonging_to(&event)
            .inner_join(users::table)
            .select(User::as_select())
            .order(users::created_at.desc())
            .load(connection)
        {
            Ok(users_in_this_event) => Ok(users_in_this_event),
            Err(e) => {

                let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserClp::get_all_user_clp_events");
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

    pub async fn get_all_users_in_clp_event_without_actix_response(event_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<User>, String>{

        
        let get_event = ClpEvent::find_by_id_without_actix_response(event_id, connection).await;
        let Ok(event) = get_event else{
            let err_resp = CLP_EVENT_NOT_FOUND;
            return Err(err_resp.to_string());
        };

        // trying to get all users in the found event
        match UserClp::belonging_to(&event)
            .inner_join(users::table)
            .select(User::as_select())
            .order(users::created_at.desc())
            .load(connection)
        {
            Ok(users_in_this_event) => Ok(users_in_this_event),
            Err(e) => {

                let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserClp::get_all_user_clp_events");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    return Err(resp_err.to_owned());
            }
        }
    
    }

    pub async fn get_all_users_and_their_events(limit: Query<Limit>, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<ClpEventsPerUser>, PanelHttpResponse>{

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

            match users::table
                .order(users::created_at.desc())
                .offset(from)
                .limit((to - from) + 1)
                .load::<User>(connection)
            {
                Ok(all_users) => {

                    match UserClp::belonging_to(&all_users)
                        .inner_join(clp_events::table)
                        .select((UserClp::as_select(), ClpEventData::as_select()))
                        .offset(from)
                        .limit((to - from) + 1)
                        .order(users_clps::registered_at.desc())
                        .load(connection)
                    {
                        Ok(all_users_events) => {
                            
                            let events_per_user: Vec<ClpEventsPerUser> = all_users_events
                                .grouped_by(&all_users)
                                .into_iter()
                                .zip(all_users)
                                .map(|(clpevts, user)|{
                                    ClpEventsPerUser{
                                        user: UserData{
                                            id: user.id, 
                                            region: user.region.clone(),
                                            username: user.username, 
                                            bio: user.bio.clone(),
                                            avatar: user.avatar.clone(),
                                            banner: user.banner.clone(),
                                            wallet_background: user.wallet_background.clone(),
                                            activity_code: user.activity_code,
                                            twitter_username: user.twitter_username, 
                                            facebook_username: user.facebook_username, 
                                            discord_username: user.discord_username, 
                                            identifier: user.identifier, 
                                            user_role: {
                                                match user.user_role.clone(){
                                                    UserRole::Admin => "Admin".to_string(),
                                                    UserRole::User => "User".to_string(),
                                                    _ => "Dev".to_string(),
                                                }
                                            },
                                            token_time: user.token_time,
                                            balance: user.balance,
                                            last_login: { 
                                                if user.last_login.is_some(){
                                                    Some(user.last_login.unwrap().to_string())
                                                } else{
                                                    Some("".to_string())
                                                }
                                            },
                                            created_at: user.created_at.to_string(),
                                            updated_at: user.updated_at.to_string(),
                                            mail: user.mail,
                                            google_id: user.google_id,
                                            microsoft_id: user.microsoft_id,
                                            is_mail_verified: user.is_mail_verified,
                                            is_phone_verified: user.is_phone_verified,
                                            phone_number: user.phone_number,
                                            paypal_id: user.paypal_id,
                                            account_number: user.account_number,
                                            device_id: user.device_id,
                                            social_id: user.social_id,
                                            cid: user.cid,
                                            screen_cid: user.screen_cid,
                                            snowflake_id: user.snowflake_id,
                                            stars: user.stars,
                                            extra: user.extra,
                                        },
                                        clp_events: {
                                            clpevts
                                                .into_iter()
                                                .map(|(uclpevt, clpevt)| clpevt)
                                                .collect::<Vec<ClpEventData>>()
                                        },
                                    }
                                })  
                                .collect();

                            Ok(events_per_user)

                        },
                        Err(e) => {
            
                            let resp_err = &e.to_string();
            
                            /* custom error handler */
                            use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                            
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec();  
                            let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserClp::get_all_users_and_their_events");
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

                    
                },
                Err(e) => {

                    let resp_err = &e.to_string();

                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserClp::get_all_users_and_their_events");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );
                }
            }

    }

    pub async fn get_all_user_events(limit: Query<Limit>, participant_id: i32,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<ClpEventData>, PanelHttpResponse>{

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

            let get_user = User::find_by_id(participant_id, connection).await;
            let Ok(user) = get_user else{
                let err_resp = get_user.unwrap_err();
                return Err(err_resp);
            };
    
            // trying to get all events for the found user
            match UserClp::belonging_to(&user)
                .inner_join(clp_events::table)
                .select(ClpEventData::as_select())
                .offset(from)
                .limit((to - from) + 1)
                .order(users_clps::registered_at.desc())
                .load(connection)
            {
                Ok(events_for_this_user) => Ok(events_for_this_user),
                Err(e) => {
    
                    let resp_err = &e.to_string();
    
    
                        /* custom error handler */
                        use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                         
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserClp::get_all_user_events");
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