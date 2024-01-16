


use actix::Addr;
use chrono::NaiveDateTime;
 
use crate::constants::{GALLERY_NOT_FOUND, GALLERY_NOT_OWNED_BY, COLLECTION_NOT_FOUND_FOR, INVALID_QUERY_LIMIT, NO_GALLERY_FOUND, NO_GALLERY_FOUND_FOR, NO_GALLERY_FOUND_FOR_COL_OWNER, GALLERY_UPLOAD_PATH};
use crate::events::publishers::user::{SingleUserNotif, NotifData, ActionType};
use crate::misc::Limit;
use crate::schema::users_collections::contract_address;
use crate::schema::users_fans::friends;
use crate::{*, misc::Response, constants::STORAGE_IO_ERROR_CODE};
use super::users::{UserWalletInfoResponse, UserData};
use super::users_collections::{UserCollection, UserCollectionData, CollectionInfoResponse};
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
#[derive(Queryable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
#[diesel(table_name=users_galleries)]
pub struct UserPrivateGallery{
    pub id: i32,
    pub owner_screen_cid: String,
    pub collections: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub gal_name: String,
    pub gal_description: String,
    pub invited_friends: Option<Vec<Option<String>>>,
    pub extra: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub gallery_background: String,
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
    pub gallery_background: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct UserPrivateGalleryInfoData{
    pub id: i32,
    pub owner_screen_cid: String,
    pub collections: u64,
    pub gal_name: String,
    pub gal_description: String,
    pub invited_friends: Vec<UserWalletInfoResponse>,
    pub extra: Option<serde_json::Value>,
    pub gallery_background: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct UserPrivateGalleryInfoDataInvited{
    pub id: i32,
    pub owner_screen_cid: String,
    pub owner_username: String,
    pub collections: u64,
    pub gal_name: String,
    pub gal_description: String,
    pub invited_friends: Vec<UserWalletInfoResponse>,
    pub extra: Option<serde_json::Value>,
    pub gallery_background: String,
    pub created_at: String,
    pub updated_at: String,
    pub requested_at: i64,
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
    pub gallery_background: String,
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
pub struct ExitFromPrivateGalleryRequest{
    pub gal_id: i32,
    pub caller_cid: String,
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GalleryExtraObjWithPriceAndImgPath{
    pub entry_price: i64,
    pub img_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct GalleryOwnerCount{
    pub owner_wallet_info: UserWalletInfoResponse,
    pub galleries_count: usize,
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserPrivateGallery{


    pub async fn get_owners_with_lots_of_galleries(owners: Vec<UserData>, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<GalleryOwnerCount>, PanelHttpResponse>{

            let mut galleries_owner_map = vec![];
            for owner in owners{
                
                if owner.screen_cid.is_none(){
                    continue;
                }
                
                let owner_screen_cid_ = owner.screen_cid.unwrap();
                let get_all_galleries_owned_by = UserPrivateGallery::get_all_for_without_limit(&owner_screen_cid_, connection);
                let galleries_owned_by = if get_all_galleries_owned_by.is_ok(){
                    get_all_galleries_owned_by.unwrap()
                } else{
                    vec![]
                };
    
                let user = User::find_by_screen_cid(&owner_screen_cid_, connection).await.unwrap();
                let user_wallet_info = UserWalletInfoResponse{
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
    
                galleries_owner_map.push(
                    GalleryOwnerCount{
                        owner_wallet_info: user_wallet_info,
                        galleries_count: galleries_owned_by.len()
                    }
                )
            }
    
            galleries_owner_map.sort_by(|gal1, gal2|{
    
                let gal1_count = gal1.galleries_count;
                let gal2_count = gal2.galleries_count;
    
                gal2_count.cmp(&gal1_count)
    
            });
            
        Ok(galleries_owner_map)
                
    }

    pub async fn upload_background(
        gal_id: i32, 
        screen_cid: &str,
        mut img: Multipart, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserPrivateGalleryData, PanelHttpResponse>{
        
            
        let Ok(gallery) = Self::find_by_id(gal_id, connection).await else{
            let resp = Response{
                data: Some(gal_id),
                message: GALLERY_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        if gallery.owner_screen_cid != screen_cid.to_string(){
            let resp = Response{
                data: Some(gal_id),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            );
        }

        let img = std::sync::Arc::new(tokio::sync::Mutex::new(img));
        let get_gallery_img_path = multipartreq::store_file(
            GALLERY_UPLOAD_PATH, &format!("{}", gal_id), 
            "gallback", 
            img).await;
        let Ok(gallery_img_path) = get_gallery_img_path else{

            let err_res = get_gallery_img_path.unwrap_err();
            return Err(err_res);
        };

        /* update the avatar field in db */
        match diesel::update(users_galleries.find(gallery.id))
            .set(gallery_background.eq(gallery_img_path))
            .returning(UserPrivateGallery::as_returning())
            .get_result(connection)
            {
                Ok(updated_gallery) => {
                    
                    Ok(
                        UserPrivateGalleryData{
                            id: updated_gallery.id,
                            owner_screen_cid: updated_gallery.owner_screen_cid,
                            collections: updated_gallery.collections,
                            gal_name: updated_gallery.gal_name,
                            gal_description: updated_gallery.gal_description,
                            invited_friends: updated_gallery.invited_friends,
                            extra: updated_gallery.extra,
                            gallery_background: updated_gallery.gallery_background,
                            created_at: updated_gallery.created_at.to_string(),
                            updated_at: updated_gallery.updated_at.to_string(),
                        }
                    )
                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserPrivateGallery::update_wallet_back");
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

    pub async fn get_all_general_info_for(screen_cid: &str, caller_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserPrivateGalleryInfoData>, PanelHttpResponse>{
        
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

        let check_we_are_friend = UserFan::are_we_friends(
            &screen_cid, 
            caller_screen_cid, connection).await;
        
        if check_we_are_friend.is_ok() && *check_we_are_friend.as_ref().unwrap(){

            /* fetch all owner galleries */
            let user_galleries = users_galleries
                .order(created_at.desc())
                .offset(from)
                .limit((to - from) + 1)
                .filter(owner_screen_cid.eq(screen_cid))
                .load::<UserPrivateGallery>(connection);
            
            let Ok(galleries) = user_galleries else{
                let resp = Response{
                    data: Some(screen_cid),
                    message: NO_GALLERY_FOUND_FOR,
                    status: 404,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
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
                
                        UserPrivateGalleryInfoData{
                            id: g.id,
                            owner_screen_cid: g.owner_screen_cid,
                            collections: {
                                let cols = g.collections;
                                let decoded_cols = if cols.is_some(){
                                    serde_json::from_value::<Vec<UserCollectionData>>(cols.clone().unwrap()).unwrap()
                                } else{
                                    vec![]
                                };

                                decoded_cols.len() as u64
                            },
                            gal_name: g.gal_name,
                            gal_description: g.gal_description,
                            invited_friends: {
                                let g_invf = g.invited_friends;
                                let mut invfs = if g_invf.is_some(){
                                    g_invf.unwrap()
                                } else{
                                    vec![]
                                };
                                
                                invfs.retain(|scid| scid.is_some());

                                invfs
                                    .into_iter()
                                    .map(|scid|{
                                        let user = User::find_by_screen_cid_none_async(&scid.unwrap(), connection).unwrap();
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
                                    })
                                    .collect::<Vec<UserWalletInfoResponse>>()
                                
                            },
                            extra: g.extra,
                            gallery_background: g.gallery_background,
                            created_at: g.created_at.to_string(),
                            updated_at: g.updated_at.to_string(),
                        }
            
                    }).collect::<Vec<UserPrivateGalleryInfoData>>()
            )
        } else{
            
            let resp_msg = format!("{caller_screen_cid:} Is Not A Friend Of {screen_cid:}");
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: &resp_msg,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        

    }

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
                is_error: true
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
                is_error: true
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
                                        
                                        let mut none_minted_nfts = decoded_nfts
                                            .into_iter()
                                            .map(|nft|{
                                                /* if we couldn't unwrap the is_minted means it's not minted yet and it's false */
                                                if nft.is_minted.unwrap_or(false) == false{
                                                    Some(nft)
                                                } else{
                                                    None
                                                }
                                            }).collect::<Vec<Option<UserNftData>>>();

                                        
                                        none_minted_nfts.retain(|nft| nft.is_some());
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
                        gallery_background: g.gallery_background,
                        created_at: g.created_at.to_string(),
                        updated_at: g.updated_at.to_string(),
                    }
        
                }).collect::<Vec<UserPrivateGalleryData>>()
        )

    }

    pub fn get_all_for_without_limit(screen_cid: &str,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserPrivateGalleryData>, PanelHttpResponse>{
        
        /* fetch all owner galleries */
        let user_galleries = users_galleries
            .order(created_at.desc())
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
                is_error: true
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
                                        
                                        let mut none_minted_nfts = decoded_nfts
                                            .into_iter()
                                            .map(|nft|{
                                                /* if we couldn't unwrap the is_minted means it's not minted yet and it's false */
                                                if nft.is_minted.unwrap_or(false) == false{
                                                    Some(nft)
                                                } else{
                                                    None
                                                }
                                            }).collect::<Vec<Option<UserNftData>>>();

                                        
                                        none_minted_nfts.retain(|nft| nft.is_some());
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
                        gallery_background: g.gallery_background,
                        created_at: g.created_at.to_string(),
                        updated_at: g.updated_at.to_string(),
                    }
        
                }).collect::<Vec<UserPrivateGalleryData>>()
        )

    }

    pub async fn get_all_galleries_invited_to(caller_screen_cid: &str, 
        limit: web::Query<Limit>, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<Option<UserPrivateGalleryInfoDataInvited>>, PanelHttpResponse>{

        let from = limit.from.unwrap_or(0) as usize;
        let to = limit.to.unwrap_or(10) as usize;

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

        let get_user_fan = UserFan::get_user_fans_data_for(&caller_screen_cid, connection).await;
        let Ok(user_fan_data) = get_user_fan else{
            let resp_error = get_user_fan.unwrap_err();
            return Err(resp_error);
        };

        let user_invitation_request_data = user_fan_data.invitation_requests;
        let mut decoded_invitation_request_data = if user_invitation_request_data.is_some(){
            serde_json::from_value::<Vec<InvitationRequestData>>(user_invitation_request_data.unwrap()).unwrap()
        } else{
            vec![]
        };

        let mut gals = decoded_invitation_request_data
            .into_iter()
            .map(|invrd|{

                if !invrd.is_accepted{
                    
                    let gal_id = invrd.gallery_id;
                    let gallery = Self::find_by_id_none_sync(gal_id, connection).unwrap();
                    Some(
                        UserPrivateGalleryInfoDataInvited{
                            id: gallery.id,
                            owner_username: User::find_by_screen_cid_none_async(&gallery.owner_screen_cid, connection).unwrap().username,
                            owner_screen_cid: gallery.owner_screen_cid,
                            collections: {
                                let cols = gallery.collections;
                                let decoded_cols = if cols.is_some(){
                                    serde_json::from_value::<Vec<UserCollectionData>>(cols.clone().unwrap()).unwrap()
                                } else{
                                    vec![]
                                };

                                decoded_cols.len() as u64
                            },
                            gal_name: gallery.gal_name,
                            gal_description: gallery.gal_description,
                            invited_friends: {
                                let g_invf = gallery.invited_friends;
                                let mut invfs = if g_invf.is_some(){
                                    g_invf.unwrap()
                                } else{
                                    vec![]
                                };
                                
                                invfs.retain(|scid| scid.is_some());

                                invfs
                                    .into_iter()
                                    .map(|scid|{
                                        let user = User::find_by_screen_cid_none_async(&scid.unwrap(), connection).unwrap();
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
                                    })
                                    .collect::<Vec<UserWalletInfoResponse>>()
                                
                            },
                            extra: gallery.extra,
                            gallery_background: gallery.gallery_background,
                            created_at: gallery.created_at.to_string(),
                            updated_at: gallery.updated_at.to_string(),
                            requested_at: invrd.requested_at
                        }
                    )
                } else{
                    None
                }

            })
            .collect::<Vec<Option<UserPrivateGalleryInfoDataInvited>>>();


        /* sorting wallet data in desc order */
        gals.sort_by(|g1, g2|{

            let g1_default = UserPrivateGalleryInfoDataInvited::default();
            let g2_default = UserPrivateGalleryInfoDataInvited::default();
            let g1 = g1.as_ref().unwrap_or(&g1_default);
            let g2 = g2.as_ref().unwrap_or(&g2_default);

            let g1_created_at = NaiveDateTime
                ::parse_from_str(&g1.created_at, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap();

            let g2_created_at = NaiveDateTime
                ::parse_from_str(&g2.created_at, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap();

            g2_created_at.cmp(&g1_created_at)

        });

        gals.retain(|upgig| upgig.is_some());

        let sliced = if gals.len() > to{
            let data = &gals[from..to+1];
            data.to_vec()
        } else{
            let data = &gals[from..gals.len()];
            data.to_vec()
        };
        

        Ok(
            sliced   
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
                is_error: true
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
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )

        }

        let inv_frds = gallery.invited_friends;
        let friends_wallet_data = if inv_frds.is_some(){
            let friends_ = inv_frds.unwrap();
            let mut friends_wallets = friends_
                .into_iter()
                .map(|f_scid|{
                    
                    if f_scid.is_some(){
                        let user_data = User::find_by_screen_cid_none_async(&f_scid.unwrap(), connection).unwrap();
                        Some(
                            UserWalletInfoResponse{
                                username: user_data.username,
                                avatar: user_data.avatar,
                                mail: user_data.mail,
                                screen_cid: user_data.screen_cid,
                                stars: user_data.stars,
                                created_at: user_data.created_at.to_string(),
                                bio: user_data.bio,
                                banner: user_data.banner,
                                extra: user_data.extra,
                            }
                        )
                    } else{
                        None
                    }

                })
                .collect::<Vec<Option<UserWalletInfoResponse>>>();

            /* sorting wallet data in desc order */
            friends_wallets.sort_by(|fw1, fw2|{
                /* 
                    cannot move out of `*fw1` which is behind a shared reference
                    move occurs because `*fw1` has type `std::option::Option<UserWalletInfoResponse>`, 
                    which does not implement the `Copy` trait and unwrap() takes the 
                    ownership of the instance.
                    also we must create a longer lifetime for `UserWalletInfoResponse::default()` by 
                    putting it inside a type so we can take a reference to it and pass the 
                    reference to the `unwrap_or()`, cause &UserWalletInfoResponse::default() will be dropped 
                    at the end of the `unwrap_or()` statement while we're borrowing it.
                */
                let fw1_default = UserWalletInfoResponse::default();
                let fw2_default = UserWalletInfoResponse::default();
                let fw1 = fw1.as_ref().unwrap_or(&fw1_default);
                let fw2 = fw2.as_ref().unwrap_or(&fw2_default);

                let fw1_created_at = NaiveDateTime
                    ::parse_from_str(&fw1.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                let fw2_created_at = NaiveDateTime
                    ::parse_from_str(&fw2.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                fw2_created_at.cmp(&fw1_created_at)

            });

            friends_wallets // sorted

        } else{
            vec![]
        };

        /*  
            first we need to slice the current vector convert that type into 
            another vector, the reason behind doing this is becasue we can't
            call to_vec() on the slice directly since the lifetime fo the slice
            will be dropped while is getting used we have to create a longer 
            lifetime then call to_vec() on that type
        */
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
                is_error: true
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
                gallery_background: gallery_info.gallery_background,
                created_at: gallery_info.created_at.to_string(), 
                updated_at: gallery_info.updated_at.to_string() 
            }
        )

    }

    pub fn find_by_id_none_sync(gallery_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<UserPrivateGalleryData, PanelHttpResponse>{

        let user_gallery = users_galleries
            .filter(users_galleries::id.eq(gallery_id))
            .first::<UserPrivateGallery>(connection);

        let Ok(gallery_info) = user_gallery else{

            let resp = Response{
                data: Some(gallery_id),
                message: GALLERY_NOT_FOUND,
                status: 404,
                is_error: true
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
                gallery_background: gallery_info.gallery_background,
                created_at: gallery_info.created_at.to_string(), 
                updated_at: gallery_info.updated_at.to_string() 
            }
        )

    }

    pub async fn find_by_owner_and_contract_address(gallery_owner: &str, col_contract_address: &str,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<UserPrivateGalleryData, PanelHttpResponse>{

        let user_galleries_data = users_galleries
            .filter(users_galleries::owner_screen_cid.eq(gallery_owner))
            .load::<UserPrivateGallery>(connection);

        let Ok(galleries_info) = user_galleries_data else{

            let resp = Response{
                data: Some(gallery_owner),
                message: NO_GALLERY_FOUND_FOR_COL_OWNER,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };


        for gallery in galleries_info{
            
            let cols = gallery.collections.clone();
            let decoded_cols = if cols.is_some(){
                serde_json::from_value::<Vec<UserCollectionData>>(cols.clone().unwrap()).unwrap()
            } else{
                vec![]
            };

            for col in decoded_cols{
                if col.contract_address == col_contract_address.to_string(){

                    /* terminate the caller with the found gallery data */
                    return Ok(
                        UserPrivateGalleryData{ 
                            id: gallery.id, 
                            owner_screen_cid: gallery.owner_screen_cid, 
                            collections: gallery.collections, 
                            gal_name: gallery.gal_name, 
                            gal_description: gallery.gal_description, 
                            invited_friends: gallery.invited_friends, 
                            extra: gallery.extra, 
                            gallery_background: gallery.gallery_background,
                            created_at: gallery.created_at.to_string(), 
                            updated_at: gallery.updated_at.to_string() 
                        }
                    )
                }
            }

        }

        Ok(UserPrivateGalleryData::default()) /* terminate the caller with a default gallery data */

    }

    pub async fn find_by_owner_and_collection_id(gallery_owner: &str, col_id: i32,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<UserPrivateGalleryData, PanelHttpResponse>{

        let user_galleries_data = users_galleries
            .filter(users_galleries::owner_screen_cid.eq(gallery_owner))
            .load::<UserPrivateGallery>(connection);

        let Ok(galleries_info) = user_galleries_data else{

            let resp = Response{
                data: Some(gallery_owner),
                message: NO_GALLERY_FOUND_FOR_COL_OWNER,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };


        for gallery in galleries_info{
            
            let cols = gallery.collections.clone();
            let decoded_cols = if cols.is_some(){
                serde_json::from_value::<Vec<UserCollectionData>>(cols.clone().unwrap()).unwrap()
            } else{
                vec![]
            };

            for col in decoded_cols{
                if col.id == col_id{

                    /* terminate the caller with the found gallery data */
                    return Ok(
                        UserPrivateGalleryData{ 
                            id: gallery.id, 
                            owner_screen_cid: gallery.owner_screen_cid, 
                            collections: gallery.collections, 
                            gal_name: gallery.gal_name, 
                            gal_description: gallery.gal_description, 
                            invited_friends: gallery.invited_friends, 
                            extra: gallery.extra, 
                            gallery_background: gallery.gallery_background,
                            created_at: gallery.created_at.to_string(), 
                            updated_at: gallery.updated_at.to_string() 
                        }
                    )
                }
            }

        }

        Ok(UserPrivateGalleryData::default()) /* terminate the caller with a default gallery data */

    }

    pub async fn find_by_contract_address(col_contract_address: &str,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<UserPrivateGalleryData, PanelHttpResponse>{

        let galleries_data = users_galleries
            .load::<UserPrivateGallery>(connection);

        let Ok(galleries_info) = galleries_data else{

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: NO_GALLERY_FOUND,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        for gallery in galleries_info{
            
            let cols = gallery.collections.clone();
            let decoded_cols = if cols.is_some(){
                serde_json::from_value::<Vec<UserCollectionData>>(cols.clone().unwrap()).unwrap()
            } else{
                vec![]
            };

            for col in decoded_cols{
                if col.contract_address == col_contract_address.to_string(){

                    return Ok(
                        UserPrivateGalleryData{ 
                            id: gallery.id, 
                            owner_screen_cid: gallery.owner_screen_cid, 
                            collections: gallery.collections, 
                            gal_name: gallery.gal_name, 
                            gal_description: gallery.gal_description, 
                            invited_friends: gallery.invited_friends, 
                            extra: gallery.extra, 
                            gallery_background: gallery.gallery_background,
                            created_at: gallery.created_at.to_string(), 
                            updated_at: gallery.updated_at.to_string() 
                        }
                    )
                }
            }

        }

        Ok(UserPrivateGalleryData::default())

    }

}

impl UserPrivateGallery{

    pub async fn exit_from_private_gallery(exit_from_private_gallery: ExitFromPrivateGalleryRequest, redis_actor: Addr<RedisActor>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserPrivateGalleryData, PanelHttpResponse>{

        let ExitFromPrivateGalleryRequest{ gal_id, caller_cid, tx_signature, hash_data } = 
            exit_from_private_gallery;
        
        let get_gallery_data = Self::find_by_id(gal_id, connection).await;
        let Ok(gallery_data) = get_gallery_data else{
            let error_resp = get_gallery_data.unwrap_err();
            return Err(error_resp);
        };
        

        let caller_screen_cid = walletreq::evm::get_keccak256_from(caller_cid.clone());
        let get_user = User::find_by_screen_cid(&caller_screen_cid, connection).await;
        let Ok(user) = get_user else{
            let resp_err = get_user.unwrap_err();
            return Err(resp_err);
        };

        let get_gal_owner = User::find_by_screen_cid(&gallery_data.owner_screen_cid, connection).await;
        let Ok(gal_owner) = get_gal_owner else{
            let resp_err = get_gal_owner.unwrap_err();
            return Err(resp_err);
        };

        let inv_frds = gallery_data.invited_friends;
        if inv_frds.is_some(){
            let mut friends_ = inv_frds.unwrap();
            if friends_.contains(&Some(caller_screen_cid.to_string())){
                let scid_idx = friends_.iter().position(|scid| *scid == Some(caller_screen_cid.to_string())).unwrap();
                friends_.remove(scid_idx);
            }

            let updated_gal_data = UpdateUserPrivateGalleryRequest{
                owner_cid: gallery_data.owner_screen_cid.clone(),
                collections: gallery_data.collections,
                gal_name: gallery_data.gal_name,
                gal_description: gallery_data.gal_description,
                invited_friends: Some(friends_), /* updated */
                extra: gallery_data.extra,
                tx_signature,
                hash_data,
            };

            // only owner can update
            match Self::update(&gallery_data.owner_screen_cid, updated_gal_data, redis_actor.clone(), gal_id, connection).await{
                Ok(updated_gallery_data) => {

                    /** -------------------------------------------------------------------- */
                    /** ----------------- publish new event data to `on_user_action` channel */
                    /** -------------------------------------------------------------------- */
                    // if the actioner is the user himself we'll notify user with something like:
                    // u've just done that action!
                    let actioner_wallet_info = UserWalletInfoResponse{
                        username: user.clone().username,
                        avatar: user.clone().avatar,
                        bio: user.clone().bio,
                        banner: user.clone().banner,
                        mail: user.clone().mail,
                        screen_cid: user.clone().screen_cid,
                        extra: user.clone().extra,
                        stars: user.clone().stars,
                        created_at: user.clone().created_at.to_string(),
                    };
                    let user_wallet_info = UserWalletInfoResponse{
                        username: gal_owner.username,
                        avatar: gal_owner.avatar,
                        bio: gal_owner.bio,
                        banner: gal_owner.banner,
                        mail: gal_owner.mail,
                        screen_cid: gal_owner.screen_cid,
                        extra: gal_owner.extra,
                        stars: gal_owner.stars,
                        created_at: gal_owner.created_at.to_string(),
                    };
                    let user_notif_info = SingleUserNotif{
                        wallet_info: user_wallet_info,
                        notif: NotifData{
                            actioner_wallet_info,
                            fired_at: Some(chrono::Local::now().timestamp()),
                            action_type: ActionType::ExitFromPrivateGalleryRequest,
                            action_data: serde_json::to_value(updated_gallery_data.clone()).unwrap()
                        }
                    };
                    let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                    events::publishers::user::publish(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;

                    Ok(updated_gallery_data)
                },
                Err(err) => Err(err)
            }

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
                    gallery_background: gallery_data.gallery_background,
                    created_at: gallery_data.created_at.to_string(), 
                    updated_at: gallery_data.updated_at.to_string() 
                }
            )
        }

    }

    pub async fn insert(new_gallery_info: NewUserPrivateGalleryRequest, redis_actor: Addr<RedisActor>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserPrivateGalleryData, PanelHttpResponse>{

            let gal_owner = walletreq::evm::get_keccak256_from(new_gallery_info.owner_cid);
            let get_user = User::find_by_screen_cid(&gal_owner, connection).await;
            let Ok(user) = get_user else{

                let resp_err = get_user.unwrap_err();
                return Err(resp_err);
            };

            let new_gal_info = InsertNewUserPrivateGalleryRequest{
                owner_screen_cid: gal_owner,
                gal_name: new_gallery_info.gal_name,
                gal_description: new_gallery_info.gal_description,
                gallery_background: String::from(""),
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
                        gallery_background: fetched_gallery_data.gallery_background,
                        created_at: fetched_gallery_data.created_at.to_string(),
                        updated_at: fetched_gallery_data.updated_at.to_string(),
                    };

                    /** -------------------------------------------------------------------- */
                    /** ----------------- publish new event data to `on_user_action` channel */
                    /** -------------------------------------------------------------------- */
                    // if the actioner is the user himself we'll notify user with something like:
                    // u've just done that action!
                    let actioner_wallet_info = UserWalletInfoResponse{
                        username: user.clone().username,
                        avatar: user.clone().avatar,
                        bio: user.clone().bio,
                        banner: user.clone().banner,
                        mail: user.clone().mail,
                        screen_cid: user.clone().screen_cid,
                        extra: user.clone().extra,
                        stars: user.clone().stars,
                        created_at: user.clone().created_at.to_string(),
                    };
                    let user_wallet_info = UserWalletInfoResponse{
                        username: user.clone().username,
                        avatar: user.clone().avatar,
                        bio: user.clone().bio,
                        banner: user.clone().banner,
                        mail: user.clone().mail,
                        screen_cid: user.clone().screen_cid,
                        extra: user.clone().extra,
                        stars: user.clone().stars,
                        created_at: user.clone().created_at.to_string(),
                    };
                    let user_notif_info = SingleUserNotif{
                        wallet_info: user_wallet_info,
                        notif: NotifData{
                            actioner_wallet_info,
                            fired_at: Some(chrono::Local::now().timestamp()),
                            action_type: ActionType::CreatePrivateGallery,
                            action_data: serde_json::to_value(user_private_gallery_data.clone()).unwrap()
                        }
                    };
                    let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                    events::publishers::user::publish(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;

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
                        status: 500,
                        is_error: true
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
    pub async fn send_invitation_request_to(send_invitation_request: SendInvitationRequest, redis_actor: Addr<RedisActor>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<InvitationRequestDataResponse, PanelHttpResponse>{
        
        let SendInvitationRequest{ gal_id, gallery_owner_cid, to_screen_cid, tx_signature, hash_data } = 
            send_invitation_request;

        let gallery_owner_screen_cid = walletreq::evm::get_keccak256_from(gallery_owner_cid.clone());
        let check_we_are_friend = UserFan::are_we_friends(
            &gallery_owner_screen_cid, 
            &to_screen_cid, connection).await;
        
        let Ok(are_we_friend) = check_we_are_friend else{
            let err_resp = check_we_are_friend.unwrap_err();
            return Err(err_resp);
        };
        
        if are_we_friend{

            let get_gallery_data = Self::find_by_id(gal_id, connection).await;
            let Ok(gallery_data) = get_gallery_data else{
                let error_resp = get_gallery_data.unwrap_err();
                return Err(error_resp);
            };

            
            if gallery_data.owner_screen_cid != gallery_owner_screen_cid{
                
                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: GALLERY_NOT_OWNED_BY,
                    status: 403,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::Forbidden().json(resp))
                )
            }

            
            let get_user = User::find_by_screen_cid(&gallery_owner_screen_cid.clone(), connection).await;
            let Ok(user) = get_user else{

                let resp_err = get_user.unwrap_err();
                return Err(resp_err);
            };


            let invitation_request_data = InvitationRequestData{
                from_screen_cid: gallery_owner_screen_cid,
                requested_at: chrono::Local::now().timestamp(),
                gallery_id: gal_id,
                is_accepted: false,
                username: user.username,
                user_avatar: user.avatar,
            };

            /* note that gallery_owner_screen_cid and to_screen_cid must be each other's friends */
            UserFan::push_invitation_request_for(&to_screen_cid, redis_actor.clone(), invitation_request_data, connection).await

        } else{
            let resp_msg = format!("{gallery_owner_cid:} Is Not A Friend Of {to_screen_cid:}");
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: &resp_msg,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

    }

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= GALLERY OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn remove_invited_friend_from(remove_invited_friend_request: RemoveInvitedFriendFromPrivateGalleryRequest,
        redis_client: RedisClient, redis_actor: Addr<RedisActor>, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserPrivateGalleryData, PanelHttpResponse>{

        let RemoveInvitedFriendFromPrivateGalleryRequest{ gal_id, caller_cid, friend_screen_cid, tx_signature, hash_data } = 
            remove_invited_friend_request;
        
        let get_gallery_data = Self::find_by_id(gal_id, connection).await;
        let Ok(gallery_data) = get_gallery_data else{
            let error_resp = get_gallery_data.unwrap_err();
            return Err(error_resp);
        };
        
        let caller_screen_cid = walletreq::evm::get_keccak256_from(caller_cid.clone());
        if gallery_data.owner_screen_cid != caller_screen_cid{
            
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }

        let get_friend_info = User::find_by_screen_cid(&friend_screen_cid, connection).await;
        let Ok(friend_info) = get_friend_info else{
            let err_resp = get_friend_info.unwrap_err();
            return Err(err_resp);
        };

        let get_gal_owner = User::find_by_screen_cid(&caller_screen_cid, connection).await;
        let Ok(gal_owner) = get_gal_owner else{
            let err_resp = get_gal_owner.unwrap_err();
            return Err(err_resp);
        };

        let mut updated_friend_balance = None;
        let mut global_gal_price = 0;

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

            // also remove from redis and pyback the friend_screen_cid with enterance fee 
            let mut conn = redis_client.get_async_connection().await.unwrap();
            let get_galleries_with_entrance_fee: redis::RedisResult<String> = conn.get("galleries_with_entrance_fee").await;
            if get_galleries_with_entrance_fee.is_ok(){
                let mut galleries_with_entrance_fee = serde_json::from_str::<HashMap<i32, (Vec<String>, i64)>>(&get_galleries_with_entrance_fee.unwrap()).unwrap();
                let get_friend_scids = galleries_with_entrance_fee.get(&gal_id);
                let mut gprice = 0 as i64;
                if get_friend_scids.is_some(){
                    let friend_scids_gprice = get_friend_scids.unwrap();
                    let mut friend_scids = friend_scids_gprice.0.to_owned();
                    gprice = friend_scids_gprice.1;
                    if friend_scids.contains(&friend_screen_cid.to_string()){
                        let scid_idx = friend_scids.iter().position(|scid| *scid == friend_screen_cid.to_string()).unwrap();
                        friend_scids.remove(scid_idx);
                        galleries_with_entrance_fee.insert(gal_id, (friend_scids.to_owned(), gprice));
                    }
                }

                let stringified_ = serde_json::to_string_pretty(&galleries_with_entrance_fee).unwrap();
                let ـ : RedisResult<String> = conn.set("galleries_with_entrance_fee", stringified_).await;
                
                // payback friend_screen_cid with the gallery price
                let new_balance = friend_info.balance.unwrap() + gprice;
                updated_friend_balance = Some(User::update_balance(friend_info.id, new_balance, redis_client.clone(), redis_actor.clone(), connection).await.unwrap());
                global_gal_price = gprice;
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

            match Self::update(&caller_screen_cid, updated_gal_data, redis_actor.clone(), gal_id, connection).await{
                Ok(update_gallery_data) => {

                    /** -------------------------------------------------------------------- */
                    /** ----------------- publish new event data to `on_user_action` channel */
                    /** -------------------------------------------------------------------- */
                    // if the actioner is the user himself we'll notify user with something like:
                    // u've just done that action!
                    let actioner_wallet_info = UserWalletInfoResponse{
                        username: gal_owner.username,
                        avatar: gal_owner.avatar,
                        bio: gal_owner.bio,
                        banner: gal_owner.banner,
                        mail: gal_owner.mail,
                        screen_cid: gal_owner.screen_cid,
                        extra: gal_owner.extra,
                        stars: gal_owner.stars,
                        created_at: gal_owner.created_at.to_string(),
                    };
                    let user_wallet_info = UserWalletInfoResponse{
                        username: friend_info.username,
                        avatar: friend_info.avatar,
                        bio: friend_info.bio,
                        banner: friend_info.banner,
                        mail: friend_info.mail,
                        screen_cid: friend_info.screen_cid,
                        extra: friend_info.extra,
                        stars: friend_info.stars,
                        created_at: friend_info.created_at.to_string(),
                    };
                    let user_notif_info = SingleUserNotif{
                        wallet_info: user_wallet_info,
                        notif: NotifData{
                            actioner_wallet_info,
                            fired_at: Some(chrono::Local::now().timestamp()),
                            action_type: ActionType::RemoveInvitedFriendFrom,
                            action_data: serde_json::to_value(update_gallery_data.clone()).unwrap()
                        }
                    };
                    let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                    events::publishers::user::publish(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;

                    Ok(update_gallery_data)
                },
                Err(err) => {
                    
                    if updated_friend_balance.is_some(){
                        let new_balance = updated_friend_balance.unwrap().balance.unwrap() - global_gal_price;
                        let updated_friend_balance = Some(User::update_balance(friend_info.id, new_balance, redis_client.clone(), redis_actor.clone(), connection).await.unwrap());
                    }

                    Err(err)
                },
            }

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
                    gallery_background: gallery_data.gallery_background,
                    created_at: gallery_data.created_at.to_string(), 
                    updated_at: gallery_data.updated_at.to_string() 
                }
            )
        }

    }

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= GALLERY OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn update(caller_screen_cid: &str, new_gallery_info: UpdateUserPrivateGalleryRequest, redis_actor: Addr<RedisActor>,
        gal_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserPrivateGalleryData, PanelHttpResponse>{
        
        let get_gallery_info = Self::find_by_id(gal_id, connection).await;
        let Ok(gallery_data) = get_gallery_info else{
            let error_resp = get_gallery_info.unwrap_err();
            return Err(error_resp);
        };

        if gallery_data.owner_screen_cid == caller_screen_cid{

            let get_user = User::find_by_screen_cid(&caller_screen_cid, connection).await;
            let Ok(user) = get_user else{
                let err_resp = get_user.unwrap_err();
                return Err(err_resp);
            };

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

                        let gallery_data = UserPrivateGalleryData{
                            id: g.id,
                            owner_screen_cid: g.owner_screen_cid,
                            collections: g.collections,
                            gal_name: g.gal_name,
                            gal_description: g.gal_description,
                            invited_friends: g.invited_friends,
                            extra: g.extra,
                            gallery_background: g.gallery_background,
                            created_at: g.created_at.to_string(),
                            updated_at: g.updated_at.to_string(),
                        };

                        /** -------------------------------------------------------------------- */
                        /** ----------------- publish new event data to `on_user_action` channel */
                        /** -------------------------------------------------------------------- */
                        // if the actioner is the user himself we'll notify user with something like:
                        // u've just done that action!
                        let actioner_wallet_info = UserWalletInfoResponse{
                            username: user.clone().username,
                            avatar: user.clone().avatar,
                            bio: user.clone().bio,
                            banner: user.clone().banner,
                            mail: user.clone().mail,
                            screen_cid: user.clone().screen_cid,
                            extra: user.clone().extra,
                            stars: user.clone().stars,
                            created_at: user.clone().created_at.to_string(),
                        };
                        let user_wallet_info = UserWalletInfoResponse{
                            username: user.clone().username,
                            avatar: user.clone().avatar,
                            bio: user.clone().bio,
                            banner: user.clone().banner,
                            mail: user.clone().mail,
                            screen_cid: user.clone().screen_cid,
                            extra: user.clone().extra,
                            stars: user.clone().stars,
                            created_at: user.clone().created_at.to_string(),
                        };
                        let user_notif_info = SingleUserNotif{
                            wallet_info: user_wallet_info,
                            notif: NotifData{
                                actioner_wallet_info,
                                fired_at: Some(chrono::Local::now().timestamp()),
                                action_type: ActionType::UpdatePrivateGallery,
                                action_data: serde_json::to_value(gallery_data.clone()).unwrap()
                            }
                        };
                        let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                        events::publishers::user::publish(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;
                        
                        Ok(
                            gallery_data
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
                            status: 500,
                            is_error: true
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
                is_error: true
            };

            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }

    }


}