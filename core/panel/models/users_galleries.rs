


use wallexerr::Wallet;
use crate::constants::{GALLERY_NOT_FOUND, GALLERY_NOT_OWNED_BY, COLLECTION_NOT_FOUND_FOR, INVALID_QUERY_LIMIT};
use crate::misc::Limit;
use crate::schema::users_fans::friends;
use crate::{*, misc::Response, constants::STORAGE_IO_ERROR_CODE};
use super::users::UserWalletInfoResponse;
use super::users_collections::{UserCollection, UserCollectionData};
use super::users_fans::{InvitationRequestData, UserFan, InvitationRequestDataResponse};
use super::users_nfts::UserNftData;
use crate::schema::users_galleries::dsl::*;
use crate::schema::users_galleries;
use crate::models::users::User;


/* 

    diesel migration generate users_galleries ---> create users_galleries migration sql files
    diesel migration run                      ---> apply sql files to db 
    diesel migration redo                     ---> drop tables 

*/
#[derive(Queryable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=users_galleries)]
pub struct UserPrivateGallery{
    pub id: i32,
    pub owner_screen_cid: String,
    pub collections: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub gal_name: String,
    pub gal_description: String,
    pub invited_friends: Option<Vec<Option<String>>>,
    pub extra: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct UserPrivateGalleryData{
    pub id: i32,
    pub owner_screen_cid: String,
    pub collections: Option<serde_json::Value>,
    pub gal_name: String,
    pub gal_description: String,
    pub invited_friends: Option<Vec<Option<String>>>,
    pub extra: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateUserPrivateGalleryRequest{
    pub owner_cid: String,
    pub collections: Option<serde_json::Value>,
    pub gal_name: String,
    pub gal_description: String,
    pub invited_friends: Option<Vec<Option<String>>>,
    pub extra: Option<serde_json::Value>,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, AsChangeset)]
#[diesel(table_name=users_galleries)]
pub struct UpdateUserPrivateGallery{
    pub owner_screen_cid: String,
    pub collections: Option<serde_json::Value>,
    pub gal_name: String,
    pub gal_description: String,
    pub invited_friends: Option<Vec<Option<String>>>,
    pub extra: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NewUserPrivateGalleryRequest{
    pub owner_cid: String,
    pub gal_name: String,
    pub gal_description: String,
    pub extra: Option<serde_json::Value>,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Insertable)]
#[diesel(table_name=users_galleries)]
pub struct InsertNewUserPrivateGalleryRequest{
    pub owner_screen_cid: String,
    pub gal_name: String,
    pub gal_description: String,
    pub extra: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RemoveInvitedFriendFromPrivateGalleryRequest{
    pub gal_id: i32,
    pub caller_cid: String,
    pub friend_screen_cid: String,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SendInvitationRequest{
    pub gal_id: i32,
    pub gallery_owner_cid: String,
    pub to_screen_cid: String,
    pub tx_signature: String,
    pub hash_data: String,
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserPrivateGallery{

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= GALLERY OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn get_all_for(screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserPrivateGalleryData>, PanelHttpResponse>{
        
        let from = limit.from.unwrap_or(0);
        let to = limit.to.unwrap_or(10);

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        /* fetch all owner galleries */
        let user_galleries = users_galleries
            .order(created_at.desc())
            .offset(from)
            .limit((to - from) + 1)
            .filter(owner_screen_cid.eq(screen_cid))
            .load::<UserPrivateGallery>(connection);
        
        /* 
            the first process of verifying the galler owner is the process
            of matching the JWT id and the caller screen cid and the second 
            step is to find all those galleries belong to the caller
        */
        let Ok(galleries) = user_galleries else{
            let resp = Response{
                data: Some(screen_cid),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        };

        Ok(
            galleries
                .into_iter()
                /* 
                    map takes an FnMut closure so it captures env vars mutably and 
                    and since the prv_cols is moving into the closure we have to 
                    clone it in each iteration to not to lose ownership
                */
                .map(|g| {
            
                    UserPrivateGalleryData{
                        id: g.id,
                        owner_screen_cid: g.owner_screen_cid,
                        collections: {
                            let cols = g.collections;
                            let decoded_cols = if cols.is_some(){
                                serde_json::from_value::<Vec<UserCollectionData>>(cols.clone().unwrap()).unwrap()
                            } else{
                                vec![]
                            };
                            
                            let none_minted_cols = decoded_cols
                                .into_iter()
                                .map(|mut c|{
                                    
                                    /* return those none minted ones */
                                    if c.nfts.is_some(){
                                        let col_nfts = c.nfts;
                                        let decoded_nfts = if col_nfts.is_some(){
                                            serde_json::from_value::<Vec<UserNftData>>(col_nfts.unwrap()).unwrap()
                                        } else{
                                            vec![]
                                        };
                                        
                                        let none_minted_nfts = decoded_nfts
                                            .into_iter()
                                            .map(|nft|{
                                                if nft.is_minted == false{
                                                    Some(nft)
                                                } else{
                                                    None
                                                }
                                            }).collect::<Vec<Option<UserNftData>>>();
                                        
                                        c.nfts = Some(serde_json::to_value(none_minted_nfts).unwrap());
                                        
                                        c
                
                                    } else{
                                        c
                                    }
                                })
                                .collect::<Vec<UserCollectionData>>();

                            Some(serde_json::to_value(none_minted_cols).unwrap())

                        },
                        gal_name: g.gal_name,
                        gal_description: g.gal_description,
                        invited_friends: g.invited_friends,
                        extra: g.extra,
                        created_at: g.created_at.to_string(),
                        updated_at: g.updated_at.to_string(),
                    }
        
                }).collect::<Vec<UserPrivateGalleryData>>()
        )

    }

    pub async fn get_all_galleries_invited_to(caller_screen_cid: &str, gal_owner_screen_cid: &str, 
        limit: web::Query<Limit>, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<Option<UserPrivateGalleryData>>, PanelHttpResponse>{

        let from = limit.from.unwrap_or(0);
        let to = limit.to.unwrap_or(10);

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        /* fetch all owner galleries */
        let user_galleries = users_galleries
            .order(created_at.desc())
            .offset(from)
            .limit((to - from) + 1)
            .filter(owner_screen_cid.eq(gal_owner_screen_cid))
            .load::<UserPrivateGallery>(connection);
            
        let Ok(galleries) = user_galleries else{
            let resp = Response{
                data: Some(gal_owner_screen_cid),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        };

        let gals = galleries
            .into_iter()
            .map(|g|{

                let inv_frds = g.invited_friends;
                if inv_frds.is_some(){
                    let friends_scid = inv_frds.as_ref().unwrap();
                    if friends_scid.contains(&Some(caller_screen_cid.to_string())){
                        /* caller must be invited to this gallery before */
                        Some(
                            UserPrivateGalleryData{
                                id: g.id,
                                owner_screen_cid: g.owner_screen_cid,
                                collections: {
                                    let cols = g.collections;
                                    let decoded_cols = if cols.is_some(){
                                        serde_json::from_value::<Vec<UserCollectionData>>(cols.clone().unwrap()).unwrap()
                                    } else{
                                        vec![]
                                    };
                                    
                                    let none_minted_cols = decoded_cols
                                        .into_iter()
                                        .map(|mut c|{
                                            
                                            /* return those none minted ones */
                                            if c.nfts.is_some(){
                                                let col_nfts = c.nfts;
                                                let decoded_nfts = if col_nfts.is_some(){
                                                    serde_json::from_value::<Vec<UserNftData>>(col_nfts.unwrap()).unwrap()
                                                } else{
                                                    vec![]
                                                };
                                                
                                                let none_minted_nfts = decoded_nfts
                                                    .into_iter()
                                                    .map(|nft|{
                                                        if nft.is_minted == false{
                                                            Some(nft)
                                                        } else{
                                                            None
                                                        }
                                                    }).collect::<Vec<Option<UserNftData>>>();
                                                
                                                c.nfts = Some(serde_json::to_value(none_minted_nfts).unwrap());
                                                
                                                c
                        
                                            } else{
                                                c
                                            }
                                        })
                                        .collect::<Vec<UserCollectionData>>();
        
                                    Some(serde_json::to_value(none_minted_cols).unwrap())
        
                                },
                                gal_name: g.gal_name,
                                gal_description: g.gal_description,
                                invited_friends: inv_frds.clone(),
                                extra: g.extra,
                                created_at: g.created_at.to_string(),
                                updated_at: g.updated_at.to_string(),
                            }
                        ) 
                    } else{
                        None
                    }
                } else{
                    None
                }

            })
            .collect::<Vec<Option<UserPrivateGalleryData>>>();
        
        

        Ok(
            if gals.contains(&None){
                vec![]
            } else{
                gals
            }
        )

    }

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= GALLERY OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn get_invited_friends_wallet_data_of_gallery(caller_screen_cid: &str, gal_id: i32, 
        limit: web::Query<Limit>, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<Option<UserWalletInfoResponse>>, PanelHttpResponse>{


        let from = limit.from.unwrap_or(0) as usize;
        let to = limit.to.unwrap_or(10) as usize;

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        let get_gallery_info = Self::find_by_id(gal_id, connection).await;
        let Ok(gallery) = get_gallery_info else{
            let resp_err = get_gallery_info.unwrap_err();
            return Err(resp_err);
        };

        if caller_screen_cid != gallery.owner_screen_cid{
            
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )

        }

        let inv_frds = gallery.invited_friends;
        let friends_wallet_data = if inv_frds.is_some(){
            let friends_ = inv_frds.unwrap();
            friends_
                .into_iter()
                .map(|f_scid|{
                    
                    if f_scid.is_some(){
                        let user_data = User::find_by_screen_cid_none_async(&f_scid.unwrap(), connection).unwrap();
                        Some(
                            UserWalletInfoResponse{
                                username: user_data.username,
                                mail: user_data.mail,
                                screen_cid: user_data.screen_cid,
                                stars: user_data.stars,
                                created_at: user_data.created_at.to_string(),
                            }
                        )
                    } else{
                        None
                    }

                })
                .collect::<Vec<Option<UserWalletInfoResponse>>>()

        } else{
            vec![]
        };

        let sliced = if friends_wallet_data.len() > to{
            let data = &friends_wallet_data[from..to+1];
            data.to_vec()
        } else{
            let data = &friends_wallet_data[from..friends_wallet_data.len()];
            data.to_vec()
        };
        
        Ok(
            if sliced.contains(&None){
                vec![]
            } else{
                sliced.to_owned()
            }
        )


    }

    pub async fn find_by_id(gallery_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<UserPrivateGalleryData, PanelHttpResponse>{

        let user_gallery = users_galleries
            .filter(users_galleries::id.eq(gallery_id))
            .first::<UserPrivateGallery>(connection);

        let Ok(gallery_info) = user_gallery else{

            let resp = Response{
                data: Some(gallery_id),
                message: GALLERY_NOT_FOUND,
                status: 404,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };


        Ok(
            UserPrivateGalleryData{ 
                id: gallery_info.id, 
                owner_screen_cid: gallery_info.owner_screen_cid, 
                collections: gallery_info.collections, 
                gal_name: gallery_info.gal_name, 
                gal_description: gallery_info.gal_description, 
                invited_friends: gallery_info.invited_friends, 
                extra: gallery_info.extra, 
                created_at: gallery_info.created_at.to_string(), 
                updated_at: gallery_info.updated_at.to_string() 
            }
        )

    }

}

impl UserPrivateGallery{

    pub async fn insert(new_gallery_info: NewUserPrivateGalleryRequest, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserPrivateGalleryData, PanelHttpResponse>{


            let new_gal_info = InsertNewUserPrivateGalleryRequest{
                owner_screen_cid: Wallet::generate_keccak256_from(new_gallery_info.owner_cid),
                gal_name: new_gallery_info.gal_name,
                gal_description: new_gallery_info.gal_description,
                extra: new_gallery_info.extra,
            };
        
            match diesel::insert_into(users_galleries)
            .values(&new_gal_info)
            .returning(UserPrivateGallery::as_returning())
            .get_result::<UserPrivateGallery>(connection)
            {
                Ok(fetched_gallery_data) => {

                    let user_private_gallery_data = UserPrivateGalleryData{
                        id: fetched_gallery_data.id,
                        owner_screen_cid: fetched_gallery_data.owner_screen_cid,
                        collections: fetched_gallery_data.collections,
                        gal_name: fetched_gallery_data.gal_name,
                        gal_description: fetched_gallery_data.gal_description,
                        invited_friends: fetched_gallery_data.invited_friends,
                        extra: fetched_gallery_data.extra,
                        created_at: fetched_gallery_data.created_at.to_string(),
                        updated_at: fetched_gallery_data.updated_at.to_string(),
                    };

                    Ok(user_private_gallery_data)

                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserGallery::insert");
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

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= GALLERY OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn send_invitation_request_to(send_invitation_request: SendInvitationRequest,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<InvitationRequestDataResponse, PanelHttpResponse>{
        
        let SendInvitationRequest{ gal_id, gallery_owner_cid, to_screen_cid, tx_signature, hash_data } = 
            send_invitation_request;

        let get_gallery_data = Self::find_by_id(gal_id, connection).await;
        let Ok(gallery_data) = get_gallery_data else{
            let error_resp = get_gallery_data.unwrap_err();
            return Err(error_resp);
        };

        let gallery_owner_screen_cid = Wallet::generate_keccak256_from(gallery_owner_cid);
        if gallery_data.owner_screen_cid != gallery_owner_screen_cid{
            
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }


        let invitation_request_data = InvitationRequestData{
            from_screen_cid: gallery_owner_screen_cid,
            requested_at: chrono::Local::now().timestamp(),
            gallery_id: gal_id,
            is_accepted: false,
        };

        /* note that gallery_owner_screen_cid and to_screen_cid must be each other's friends */
        UserFan::push_invitation_request_for(&to_screen_cid, invitation_request_data, connection).await


    }

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= GALLERY OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn remove_invited_friend_from(remove_invited_friend_request: RemoveInvitedFriendFromPrivateGalleryRequest,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserPrivateGalleryData, PanelHttpResponse>{

        let RemoveInvitedFriendFromPrivateGalleryRequest{ gal_id, caller_cid, friend_screen_cid, tx_signature, hash_data } = 
            remove_invited_friend_request;
        
        let get_gallery_data = Self::find_by_id(gal_id, connection).await;
        let Ok(gallery_data) = get_gallery_data else{
            let error_resp = get_gallery_data.unwrap_err();
            return Err(error_resp);
        };

        let caller_screen_cid = Wallet::generate_keccak256_from(caller_cid.clone());
        if gallery_data.owner_screen_cid != caller_screen_cid{
            
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }

        /* 
            since we've moved the gallery_data.invited_friends into inv_frds 
            thus to return the old data we'll use inv_frds 
        */
        let inv_frds = gallery_data.invited_friends;
        if inv_frds.is_some(){
            let mut friends_ = inv_frds.unwrap();
            if friends_.contains(&Some(friend_screen_cid.to_string())){
                let scid_idx = friends_.iter().position(|scid| *scid == Some(friend_screen_cid.to_string())).unwrap();
                friends_.remove(scid_idx);
            }

            let updated_gal_data = UpdateUserPrivateGalleryRequest{
                owner_cid: caller_cid,
                collections: gallery_data.collections,
                gal_name: gallery_data.gal_name,
                gal_description: gallery_data.gal_description,
                invited_friends: Some(friends_), /* updated */
                extra: gallery_data.extra,
                tx_signature,
                hash_data,
            };

            Self::update(&caller_screen_cid, updated_gal_data, gal_id, connection).await

        } else{

            /* just return the old one */
            Ok(
                UserPrivateGalleryData{ 
                    id: gallery_data.id, 
                    owner_screen_cid: gallery_data.owner_screen_cid, 
                    collections: gallery_data.collections, 
                    gal_name: gallery_data.gal_name, 
                    gal_description: gallery_data.gal_description, 
                    invited_friends: inv_frds, 
                    extra: gallery_data.extra, 
                    created_at: gallery_data.created_at.to_string(), 
                    updated_at: gallery_data.updated_at.to_string() 
                }
            )
        }

    }

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= GALLERY OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn update(caller_screen_cid: &str, new_gallery_info: UpdateUserPrivateGalleryRequest, 
        gal_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserPrivateGalleryData, PanelHttpResponse>{
        
        let get_gallery_info = Self::find_by_id(gal_id, connection).await;
        let Ok(gallery_data) = get_gallery_info else{
            let error_resp = get_gallery_info.unwrap_err();
            return Err(error_resp);
        };

        if gallery_data.owner_screen_cid == caller_screen_cid{

            let update_gal_data = UpdateUserPrivateGallery{
                owner_screen_cid: caller_screen_cid.to_string(),
                collections: new_gallery_info.collections,
                gal_name: new_gallery_info.gal_name,
                gal_description: new_gallery_info.gal_description,
                invited_friends: new_gallery_info.invited_friends,
                extra: new_gallery_info.extra,
            };
            
            match diesel::update(users_galleries.find(gallery_data.id))
                .set(&update_gal_data)
                .returning(UserPrivateGallery::as_returning())
                .get_result(connection)
                {
                
                    Ok(g) => {
                        Ok(
                            UserPrivateGalleryData{
                                id: g.id,
                                owner_screen_cid: g.owner_screen_cid,
                                collections: g.collections,
                                gal_name: g.gal_name,
                                gal_description: g.gal_description,
                                invited_friends: g.invited_friends,
                                extra: g.extra,
                                created_at: g.created_at.to_string(),
                                updated_at: g.updated_at.to_string(),
                            }
                        )

                    },
                    Err(e) => {
                        
                        let resp_err = &e.to_string();

                        /* custom error handler */
                        use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                            
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserGallery::update");
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
            
        } else{

            let resp = Response::<'_, &str>{
                data: Some(caller_screen_cid),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
            };

            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }

    }

}