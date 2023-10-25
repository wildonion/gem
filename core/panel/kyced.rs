

use wallexerr::Wallet;
use crate::{*, models::users::User, misc::Response, constants::{NOT_VERIFIED_PHONE, NOT_VERIFIED_MAIL, INSUFFICIENT_FUNDS, USER_SCREEN_CID_NOT_FOUND, INVALID_SIGNATURE}};


/*   ------------------------------------------------------------------------------------------------
    | user must pay for the following calls and spend in-app token by signing the request body 
    | of each api after that if the request was kyced-verified then the rest of the api logic 
    | will be executed otherwise rejected, the following is the process of kycing the request:
    |    🆔 there must be a user with the passed in id
    |    📮 there must be a verified mail and phone number related to the found user
    |    💰 user must have enough balance to execute the call
    |    🏷 there must be an screen cid related to the passed in cid (keccak256 of cid must be in db)
    |    🔑 the secp256k1 signature inside the api body must be valid
    |
    |
*/
pub async fn verify_request(
    the_user_id: i32, from_cid: &str, tx_signature: &str, 
    hash_data: &str, deposited_amount: Option<i64>,
    connection: &mut PooledConnection<ConnectionManager<PgConnection>>
) -> Result<User, PanelHttpResponse>{

    /* find user info with this id */
    let get_user = User::find_by_id(the_user_id, connection).await;
    let Ok(user) = get_user else{
        let error_resp = get_user.unwrap_err();
        return Err(error_resp);
    };

    /* if the phone wasn't verified user can't deposit */
    if user.phone_number.is_none() || 
    !user.is_phone_verified{

        let resp = Response::<&[u8]>{
            data: Some(&[]),
            message: NOT_VERIFIED_PHONE,
            status: 406
        };
        return Err(
            Ok(HttpResponse::NotAcceptable().json(resp))
        );

    }

    /* if the mail wasn't verified user can't deposit */
    if user.mail.is_none() || 
    !user.is_mail_verified{

        let resp = Response::<&[u8]>{
            data: Some(&[]),
            message: NOT_VERIFIED_MAIL,
            status: 406
        };
        return Err(
            Ok(HttpResponse::NotAcceptable().json(resp))
        );

    }

    /* check that the user has enough balance */
    if deposited_amount.is_some() 
        && (user.balance.is_none() || 
            user.balance.unwrap() < 0 || 
            user.balance.unwrap() < deposited_amount.unwrap()){

        let resp = Response::<&[u8]>{
            data: Some(&[]),
            message: INSUFFICIENT_FUNDS,
            status: 406
        };
        return Err(
            Ok(HttpResponse::NotAcceptable().json(resp))
        );
    }

    /* 
        first we'll try to find the a user with the passed in screen_cid 
        generated from keccak256 of cid then we'll go for the signature 
        verification process 
    */
    let find_user_screen_cid = User::find_by_screen_cid(
        &Wallet::generate_keccak256_from(from_cid.to_string()), connection
    ).await;
    let Ok(user_info) = find_user_screen_cid else{
        
        let resp = Response{
            data: Some(from_cid.to_string()),
            message: USER_SCREEN_CID_NOT_FOUND,
            status: 404
        };
        return Err(
            Ok(HttpResponse::NotAcceptable().json(resp))
        );

    };
    
    /* verifying signature */
    let verification_sig_res = wallet::evm::verify_signature(
        user_info.screen_cid.unwrap(),
        &tx_signature,
        &hash_data
    ).await;
    if verification_sig_res.is_err(){

        let resp = Response::<&[u8]>{
            data: Some(&[]),
            message: INVALID_SIGNATURE,
            status: 406
        };
        return Err(
            Ok(HttpResponse::NotAcceptable().json(resp))
        );

    }

    Ok(user)
    
}