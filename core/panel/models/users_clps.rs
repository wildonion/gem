




use std::time::{SystemTime, UNIX_EPOCH};
use actix_web::web::Query;
use chrono::NaiveDateTime;
use crate::adapters::nftport;
use crate::constants::{COLLECTION_NOT_FOUND_FOR, INVALID_QUERY_LIMIT, GALLERY_NOT_OWNED_BY, CANT_GET_CONTRACT_ADDRESS, USER_NOT_FOUND, USER_SCREEN_CID_NOT_FOUND, COLLECTION_UPLOAD_PATH, UNSUPPORTED_FILE_TYPE, TOO_LARGE_FILE_SIZE, STORAGE_IO_ERROR_CODE, COLLECTION_NOT_OWNED_BY, CANT_CREATE_COLLECTION_ONCHAIN, INVALID_CONTRACT_TX_HASH, CANT_UPDATE_COLLECTION_ONCHAIN, COLLECTION_NOT_FOUND_FOR_CONTRACT, CLP_EVENT_NO_FOUND_ANY, CLP_EVENT_NO_FOUND};
use crate::misc::{Response, Limit};
use crate::{*, constants::COLLECTION_NOT_FOUND_OF};
use super::clp_events::ClpEventData;
use super::users::{User, UserData};
use super::users_galleries::{UserPrivateGalleryData, UserPrivateGallery, UpdateUserPrivateGallery, UpdateUserPrivateGalleryRequest};
use super::users_nfts::UserNftData;
use crate::schema::users_clps::dsl::*;
use crate::schema::{users_clps, clp_events, users};
use crate::models::clp_events::ClpEvent;


/* 

    in order this table works correctly clp_events must be initialized first
    since there is a reference as fk to the pk of clp_events and users

    diesel migration generate users_clps        ---> create users_clps migration sql files
    diesel migration run                        ---> apply sql files to db 
    diesel migration redo                       ---> drop tables 

*/
#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
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


impl UserClp{

    pub async fn get_all_users_clps(connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<UserClp>, PanelHttpResponse>{

        let users_clps_ = users_clps
            .order(users_clps::registered_at.desc())
            .load::<UserClp>(connection);
            
        let Ok(all_users_clps) = users_clps_ else{
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: CLP_EVENT_NO_FOUND_ANY,
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
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
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
                        
        let Ok(user) = single_user_clp else{
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: CLP_EVENT_NO_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)
        
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
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
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

    pub async fn get_all_users_and_their_events(limit: Query<Limit>, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<ClpEventsPerUser>, PanelHttpResponse>{

            /* 
            
                let all_authors = authors::table
                    .select(Author::as_select())
                    .load(conn)?;

                let books = BookAuthor::belonging_to(&authors)
                    .inner_join(books::table)
                    .select((BookAuthor::as_select(), Book::as_select()))
                    .load(conn)?;

                let books_per_author: Vec<(Author, Vec<Book>)> = books
                    .grouped_by(&all_authors)
                    .into_iter()
                    .zip(authors)
                    .map(|(b, author)| (author, b.into_iter().map(|(_, book)| book).collect()))
                    .collect();

                println!("All authors including their books: {books_per_author:?}");
            */

            todo!()

    }

    pub async fn get_all_user_events(limit: Query<Limit>, participant_id: i32,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<ClpEventData>, PanelHttpResponse>{


        todo!()

    }


}