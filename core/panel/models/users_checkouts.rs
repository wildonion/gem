



use crate::*;
use crate::misc::{Response, Limit};
use crate::schema::users::dsl::*;
use crate::schema::users_checkouts;
use crate::constants::*;
use crate::models::users::{User, UserData, UserRole};
use crate::schema::users_checkouts::dsl::*;




/* 

    diesel migration generate users_checkouts ---> create users_checkouts migration sql files
    diesel migration run                     ---> apply sql files to db 
    diesel migration redo                    ---> drop tables 

*/


#[derive(Identifiable, Selectable, Queryable, Debug)]
#[diesel(table_name=users_checkouts)]
pub struct UserCheckout{
    pub id: i32,
    pub user_cid: String,
    pub product_id: String,
    pub price_id: String,
    pub payment_status: String,
    pub payment_intent: String,
    pub c_status: String,
    pub checkout_session_url: String,
    pub checkout_session_id: String,
    pub checkout_session_expires_at: i64,
    pub tokens: i64,
    pub usd_token_price: i64,
    pub tx_signature: String,
    pub iat: chrono::NaiveDateTime
}

#[derive(Insertable, Clone, Debug, ToSchema, PartialEq)]
#[diesel(table_name=users_checkouts)]
pub struct NewUserCheckout{
    pub user_cid: String,
    pub product_id: String,
    pub price_id: String,
    pub payment_status: String,
    pub payment_intent: String,
    pub c_status: String,
    pub checkout_session_url: String,
    pub checkout_session_id: String,
    pub checkout_session_expires_at: i64,
    pub tokens: i64,
    pub usd_token_price: i64,
    pub tx_signature: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct UserCheckoutData{
    pub id: i32,
    pub user_cid: String,
    pub product_id: String,
    pub price_id: String,
    pub payment_status: String,
    pub payment_intent: String,
    pub c_status: String,
    pub checkout_session_url: String,
    pub checkout_session_id: String,
    pub checkout_session_expires_at: i64,
    pub tokens: i64,
    pub usd_token_price: i64,
    pub tx_signature: String,
    pub iat: String
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserCheckout{

    pub async fn insert(new_user_checkout: NewUserCheckout, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserCheckoutData, PanelHttpResponse>{

        match diesel::insert_into(users_checkouts)
            .values(&new_user_checkout)
            .returning(UserCheckout::as_returning())
            .get_result::<UserCheckout>(connection)
            {
                Ok(user_checkout) => {

                    Ok(UserCheckoutData{
                        id: user_checkout.id,
                        user_cid: user_checkout.user_cid,
                        product_id: user_checkout.product_id,
                        price_id: user_checkout.price_id,
                        payment_status: user_checkout.payment_status,
                        payment_intent: user_checkout.payment_intent,
                        c_status: user_checkout.c_status,
                        checkout_session_url: user_checkout.checkout_session_url,
                        checkout_session_id: user_checkout.checkout_session_id,
                        checkout_session_expires_at: user_checkout.checkout_session_expires_at,
                        tokens: user_checkout.tokens,
                        usd_token_price: user_checkout.usd_token_price,
                        tx_signature: user_checkout.tx_signature,
                        iat: user_checkout.iat.to_string(),
                    })

                },
                Err(e) => {

                    let resp_err = &e.to_string();

                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserCheckout::insert");
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

    pub async fn get_all(connection: &mut PooledConnection<ConnectionManager<PgConnection>>, 
        limit: web::Query<Limit>) 
        -> Result<Vec<UserCheckoutData>, PanelHttpResponse>{

        
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

        let users_checkouts_data = users_checkouts
            .order(iat.desc())
            .offset(from)
            .limit((to - from) + 1)
            .load::<UserCheckout>(connection);
            
        let Ok(checkouts) = users_checkouts_data else{
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: NO_CHECKOUTS_YET,
                status: 404,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };

        Ok(
            checkouts
                .into_iter()
                .map(|c| {
                    UserCheckoutData{
                        id: c.id,
                        user_cid: c.user_cid,
                        product_id: c.product_id,
                        price_id: c.price_id,
                        payment_status: c.payment_status,
                        payment_intent: c.payment_intent,
                        c_status: c.c_status,
                        checkout_session_url: c.checkout_session_url,
                        checkout_session_id: c.checkout_session_id,
                        checkout_session_expires_at: c.checkout_session_expires_at,
                        tokens: c.tokens,
                        usd_token_price: c.usd_token_price,
                        tx_signature: c.tx_signature,
                        iat: c.iat.to_string(),
                    }
                }).collect::<Vec<UserCheckoutData>>()
        )

    }

    pub async fn get_all_unpaid_for(user_crypto_id: &str, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserCheckoutData>, PanelHttpResponse>{

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

        let users_checkouts_data = users_checkouts
            .order(iat.desc())
            .filter(user_cid.eq(user_crypto_id))
            .filter(payment_status.eq("unpaid"))
            .offset(from)
            .limit((to - from) + 1)
            .load::<UserCheckout>(connection);
            
        let Ok(checkouts) = users_checkouts_data else{
            let resp = Response{
                data: Some(user_crypto_id),
                message: CID_HAS_NO_PAID_CHECKOUT_YET,
                status: 404,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };

        Ok(
            checkouts
                .into_iter()
                .map(|c| {
                    UserCheckoutData{
                        id: c.id,
                        user_cid: c.user_cid,
                        product_id: c.product_id,
                        price_id: c.price_id,
                        payment_status: c.payment_status,
                        payment_intent: c.payment_intent,
                        c_status: c.c_status,
                        checkout_session_url: c.checkout_session_url,
                        checkout_session_id: c.checkout_session_id,
                        checkout_session_expires_at: c.checkout_session_expires_at,
                        tokens: c.tokens,
                        usd_token_price: c.usd_token_price,
                        tx_signature: c.tx_signature,
                        iat: c.iat.to_string(),
                    }
                }).collect::<Vec<UserCheckoutData>>()
        )

    }

    pub async fn get_all_paid_for(user_crypto_id: &str, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserCheckoutData>, PanelHttpResponse>{
        
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

        let users_checkouts_data = users_checkouts
            .order(iat.desc())
            .filter(user_cid.eq(user_crypto_id))
            .filter(payment_status.eq("paid"))
            .offset(from)
            .limit((to - from) + 1)
            .load::<UserCheckout>(connection);
            
        let Ok(checkouts) = users_checkouts_data else{
            let resp = Response{
                data: Some(user_crypto_id),
                message: CID_HAS_NO_UNPAID_CHECKOUT_YET,
                status: 404,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };

        Ok(
            checkouts
                .into_iter()
                .map(|c| {
                    UserCheckoutData{
                        id: c.id,
                        user_cid: c.user_cid,
                        product_id: c.product_id,
                        price_id: c.price_id,
                        payment_status: c.payment_status,
                        payment_intent: c.payment_intent,
                        c_status: c.c_status,
                        checkout_session_url: c.checkout_session_url,
                        checkout_session_id: c.checkout_session_id,
                        checkout_session_expires_at: c.checkout_session_expires_at,
                        tokens: c.tokens,
                        usd_token_price: c.usd_token_price,
                        tx_signature: c.tx_signature,
                        iat: c.iat.to_string(),
                    }
                }).collect::<Vec<UserCheckoutData>>()
        )

    }

    pub async fn update(session_id: &str, new_payment_intent: &str,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserCheckoutData, PanelHttpResponse>{
        
        let get_user_checkout = users_checkouts
            .filter(checkout_session_id.eq(session_id))
            .first::<UserCheckout>(connection);

        let Ok(user_checkout) = get_user_checkout else{
            let resp = Response{
                data: Some(session_id),
                message: NO_CHECKOUT_FOUND,
                status: 404
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };
        
        match diesel::update(users_checkouts.find(user_checkout.id))
            .set((c_status.eq("complete"), payment_intent.eq(new_payment_intent), payment_status.eq("paid")))
            .returning(UserCheckout::as_returning())
            .get_result(connection)
            {
            
                Ok(c) => {
                    Ok(
                        UserCheckoutData{
                            id: c.id,
                            user_cid: c.user_cid,
                            product_id: c.product_id,
                            price_id: c.price_id,
                            payment_status: c.payment_status,
                            payment_intent: c.payment_intent,
                            c_status: c.c_status,
                            checkout_session_url: c.checkout_session_url,
                            checkout_session_id: c.checkout_session_id,
                            checkout_session_expires_at: c.checkout_session_expires_at,
                            tokens: c.tokens,
                            usd_token_price: c.usd_token_price,
                            tx_signature: c.tx_signature,
                            iat: c.iat.to_string(),
                        }
                    )

                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();

                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserCheckout::update");
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