

use crate::{*, models::users::User, helpers::misc::Response, constants::{NOT_VERIFIED_PHONE, NOT_VERIFIED_MAIL, INSUFFICIENT_FUNDS, USER_SCREEN_CID_NOT_FOUND, INVALID_SIGNATURE, CALLER_IS_NOT_THE_OWNER, EMPTY_CRYPTO_DATA}};


/*  > ------------------------------------------------------------------------------------------------
    | user must pay for the following calls and spend in-app token by signing the request body 
    | of each api after that if the request was kyced-verified then the rest of the api logic 
    | will be executed otherwise rejected, the following is the process of kycing the request:
    |    🆔 there must be a user with the passed in id
    |    📮 there must be a verified mail related to the found user
    |    💰 user must have enough balance to execute the call
    |    🏷 there must be an screen cid related to the passed in cid (keccak256 of cid must be in db)
    |    🔑 the secp256k1 signature inside the api body must be valid
    |
    |
*/
pub async fn verify_request(
    the_user_id: i32, from_cid: &str, tx_signature: &str, 
    hash_data: &str, deposited_amount: Option<i64>,
    connection: &mut DbPoolConnection
) -> Result<User, PanelHttpResponse>{

    /* find user info with this id */
    let get_user = User::find_by_id(the_user_id, connection).await;
    let Ok(user) = get_user else{
        let error_resp = get_user.unwrap_err();
        return Err(error_resp);
    };

    /* crypto data the signature, cid and hash data must not be empty */
    if tx_signature == "" || 
        from_cid == "" ||
        hash_data == ""{

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: EMPTY_CRYPTO_DATA,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        }

    /* ------------------------------------------------------------ */
    /* ------------------ NO NEED TO BE VERIFIED ------------------ */
    /* ------------------------------------------------------------ */
    /* if the phone wasn't verified user can't deposit */
    // if user.phone_number.is_none() || 
    // !user.is_phone_verified{

    //     let resp = Response::<&[u8]>{
    //         data: Some(&[]),
    //         message: NOT_VERIFIED_PHONE,
    //         status: 406,
    //         is_error: true
    //     };
    //     return Err(
    //         Ok(HttpResponse::NotAcceptable().json(resp))
    //     );

    // }
    /* ------------------------------------------------------------ */
    /* ------------------------------------------------------------ */
    /* ------------------------------------------------------------ */

    /* if the mail wasn't verified user can't deposit */
    if user.mail.is_none() || 
    !user.is_mail_verified{

        let resp = Response::<&[u8]>{
            data: Some(&[]),
            message: NOT_VERIFIED_MAIL,
            status: 406,
            is_error: true
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
            status: 406,
            is_error: true
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
        &walletreq::evm::get_keccak256_from(from_cid.to_string()), connection
    ).await;
    let Ok(user_info) = find_user_screen_cid else{
        
        let resp = Response{
            data: Some(from_cid.to_string()),
            message: USER_SCREEN_CID_NOT_FOUND,
            status: 404,
            is_error: true
        };
        return Err(
            Ok(HttpResponse::NotAcceptable().json(resp))
        );

    };

    /* caller of the method must owns the crypto id */
    if user.clone().cid.unwrap() != from_cid.to_string(){
        
        let resp = Response{
            data: Some(from_cid.to_string()),
            message: CALLER_IS_NOT_THE_OWNER,
            status: 403,
            is_error: true
        };
        return Err(
            Ok(HttpResponse::Forbidden().json(resp))
        );
    }
    
    /* verifying signature */
    let verification_sig_res = walletreq::evm::verify_signature(
        user_info.screen_cid.unwrap(),
        &tx_signature,
        &hash_data
    ).await;
    if verification_sig_res.is_err(){

        let resp = Response::<&[u8]>{
            data: Some(&[]),
            message: INVALID_SIGNATURE,
            status: 406,
            is_error: true
        };
        return Err(
            Ok(HttpResponse::NotAcceptable().json(resp))
        );

    }

    Ok(user)
    
}