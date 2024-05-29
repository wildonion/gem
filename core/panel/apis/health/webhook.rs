



pub use super::*;

#[post("/cid/wallet/stripe/update/balance/webhook/{session_id}/{payment_intent}")]
#[passport(admin, user, dev)]
pub(self) async fn update_user_balance_webhook(
        req: HttpRequest,
        params: web::Path<(String, String)>,
        app_state: web::Data<AppState>,
    ) -> PanelHttpResponse{

    /* 
        stripe event handler and webhook subscriber to the new success checkout session event
        webhook means once an event gets triggered an api call will be invoked to notify (it's 
        like a notification to the server) server about the event happend as a result of handling 
        another process in some where like a payment result in which server subscribes to incoming 
        event type and can publish it to redispubsub so other app, threads and scopes can also 
        subscribe to it or charge an in-app token balance of a user like the following logic
    */

    /* extracting shared state data */
    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().unwrap().get_redis().await.unwrap();
    let async_redis_client = storage.as_ref().unwrap().get_async_redis_pubsub_conn().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();

            let session_id = params.clone().0;
            let payment_intent = params.clone().1;
            let stripe_webhook_signature = env::var("STRIPE_WEBHOOK_SIGNATURE").unwrap();
            let webhook_event_signature = req.headers().get("stripe-signature").unwrap().to_str().unwrap();
            if &stripe_webhook_signature != webhook_event_signature{

                resp!{
                    &[u8], // the data type
                    &[], // response data
                    STRIPE_INVALID_WEBHOOK_SIGNATURE, // response message
                    StatusCode::EXPECTATION_FAILED, // status code
                    None::<Cookie<'_>>, // cookie
                }
            }

            match UserCheckout::update(&session_id, &payment_intent, connection).await{
                Ok(updated_user_checkout) => {
                    
                    /* update the user balance */
                    let find_user_screen_cid = User::find_by_screen_cid(&walletreq::evm::get_keccak256_from(updated_user_checkout.user_cid.clone()), connection).await;
                        let Ok(user_info) = find_user_screen_cid else{
                            
                            resp!{
                                String, // the data type
                                updated_user_checkout.user_cid, // response data
                                &USER_SCREEN_CID_NOT_FOUND, // response message
                                StatusCode::NOT_FOUND, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        };

                    let new_balance = if user_info.balance.is_none(){0 + updated_user_checkout.tokens} else{user_info.balance.unwrap() + updated_user_checkout.tokens};
                    match User::update_balance(user_info.id, "BuyToken", "Credit", new_balance, redis_client.to_owned(), redis_actix_actor, connection).await{

                        Ok(updated_user_data) => {

                            resp!{
                                UserData, // the data type
                                updated_user_data, // response data
                                PAID_SUCCESSFULLY, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    }

                },
                Err(resp) => {
                    resp
                }
            }

        },
        None => {
            
            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}


pub mod exports{
    pub use super::update_user_balance_webhook;
}