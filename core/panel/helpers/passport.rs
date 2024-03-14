


use actix::Addr;
use crate::{constants::{EXPIRED_JWT, EXPIRED_REF_JWT, FETCHED, IDENTIFIER_ALREADY_EXISTS, INVALID_REF_JWT, REGISTERED, WRONG_IDENTIFIER, WRONG_PASSWORD}, helpers::misc::Response, models::users::{JWTClaims, User, UserData, UserRole}, *
};

use self::models::users::UserLoginInfoRequest;



/* 

    read more: https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html

    #[trait_variant::make(PassportSend: Send)] creates two versions of the trait: 
    Passport for single-threaded executors and PassportSend for multithreaded 
    work-stealing executors like so:
    pub trait PassportSend: Send {
        type Request;
        async fn get_user(&self, role: Option<UserRole>, 
            connection: &mut DbPoolConnection) 
            -> Result<JWTClaims, PanelHttpResponse>;

        fn check_refresh_token(&self, connection: &mut DbPoolConnection) 
            -> Result<User, PanelHttpResponse>;
    }

    in the future this would be like:
    trait PassportSend = Passport<get_user(): Send> + Send;

*/
#[trait_variant::make(PassportSend: Send)] // Passport trait must be Send so we can call its async method in tokio runtime threads
pub trait Passport{

    type Request;
    async fn get_user(&self, role: Option<UserRole>, 
        connection: &mut DbPoolConnection) 
        -> Result<JWTClaims, PanelHttpResponse>;

    async fn get_passport<T: UserDataExt>(&self, login_info: T, redis_client: redis::Client, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
        -> Result<PanelHttpResponse, PanelHttpResponse>;

    async fn create_passport<T: UserDataExt>(&self, new_user_info: T,
        connection: &mut DbPoolConnection) 
        -> Result<PanelHttpResponse, PanelHttpResponse>;

    fn check_refresh_token(&self, connection: &mut DbPoolConnection) 
        -> Result<User, PanelHttpResponse>;

    async fn stream() -> impl Iterator<Item = String>; // the default type param of the Iterator trait has been set to String

}

impl Passport for HttpRequest{

    /* 
        HttpRequest is not Send because compiler says:
            captured value is not `Send` because `&` references cannot be sent unless their referent is `Sync`
        this is because we're casting the &self into &Self::Request 
        and since User::passport is a future object all the types which 
        are being passed into the method must be Send so we can share them
        between threads which in our case `req` is not Send 
    */
    type Request = HttpRequest;

    fn check_refresh_token(&self, connection: &mut DbPoolConnection) -> Result<User, PanelHttpResponse>{

        let req = self as &Self::Request; // casting the self into a request object

        if let Some(authen_header) = req.headers().get("Authorization"){
            if let Ok(authen_str) = authen_header.to_str(){
                if authen_str.starts_with("bearer") || authen_str.starts_with("Bearer"){
                    
                    let token = authen_str[6..authen_str.len()].trim();

                    let token_result = User::decode_token(token);
                
                    match token_result{
                        Ok(token) => {

                            /* cookie time is not expired yet */
                            let token_data = token.claims;
                            let _id = token_data._id;
                            let role = token_data.user_role.clone();
                            let _token_time = token_data.token_time; /* if a user do a login this will be reset and the last JWT will be invalid */
                            let exp_time = token_data.exp;
                            
                            if !token_data.is_refresh{
                                let resp = Response{
                                    data: Some(_id.to_owned()),
                                    message: INVALID_REF_JWT,
                                    status: 406,
                                    is_error: true,
                                };
                                return Err(
                                    Ok(HttpResponse::NotAcceptable().json(resp))
                                );
                            }

                            if Utc::now().timestamp_nanos_opt().unwrap() > exp_time{
                                let resp = Response{
                                    data: Some(_id.to_owned()),
                                    message: EXPIRED_REF_JWT,
                                    status: 406,
                                    is_error: true,
                                };
                                return Err(
                                    Ok(HttpResponse::NotAcceptable().json(resp))
                                );
                            } 


                            let get_user = User::find_by_id_none_async(_id, connection);
                            let Ok(user) = get_user else{
                                let err_resp = get_user.unwrap_err();
                                return Err(err_resp);
                            };
                            
                            Ok(user)

                        },
                        Err(e) => {
                            let resp = Response::<&[u8]>{
                                data: Some(&[]),
                                message: &e.to_string(),
                                status: 500,
                                is_error: true,
                            };
                            return Err(
                                Ok(HttpResponse::InternalServerError().json(resp))
                            );
                        }
                    }

                } else{
                    Ok(
                        User::default()
                    )
                }
            } else{
                Ok(
                    User::default()
                )
            }
        } else{
            Ok(
                User::default()
            )
        } 

    }

    async fn get_user(&self, role: Option<UserRole>, 
        connection: &mut DbPoolConnection) 
        -> Result<JWTClaims, PanelHttpResponse>{

        /*
            HttpRequest doesn't implement Copy thus dereferencing it
            is now allowed since dereferencing, moves out of the type
            itself and return the owned type which is not possible if 
            there is a pointer of the type exists and that type is not
            Copy, we must either use the borrow form of data or pass its 
            clone to other scopes
        */
        let req = self as &Self::Request; // casting the self into a request object

        /* 
            Copy trait is not implemented for req thus we can't dereference it or 
            move out of it since we can't deref a type if it's behind a pointer, 
            in our case req is behind a pointer and we must clone it to pass 
            it to other scopes
        */
        match User::passport(req.clone(), role, connection).await{
            Ok(token_data) => Ok(token_data),
            Err(resp) => Err(resp)
        }

    }

    async fn stream() -> impl Iterator<Item = String> {
        [String::from("")].into_iter()
    }

    // UserDataExt is only visible in this crate and can 
    // only be impl for other types only in here
    async fn get_passport<T: UserDataExt>(&self, login_info: T,
        redis_client:redis::Client, 
        redis_actor:Addr<RedisActor>, 
        connection: &mut DbPoolConnection) 
        -> Result<PanelHttpResponse, PanelHttpResponse>{
        
        let req = self as &Self::Request; // casting the self into a request object 
        
        match User::find_by_identifier(&login_info.get_identifier().to_owned(), connection).await{
            Ok(user) => {

                let pswd_verification = user.verify_pswd(&login_info.get_password()); 
                let Ok(pswd_flag) = pswd_verification else{
                    let err_msg = pswd_verification.unwrap_err();

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: &err_msg.to_string(),
                        status: 500,
                        is_error: true,
                    };
                    return 
                        Ok(Ok(HttpResponse::InternalServerError().json(resp)));
                };

                if !pswd_flag{

                    let resp = Response::<String>{
                        data: Some(login_info.get_identifier()),
                        message: WRONG_PASSWORD,
                        status: 403,
                        is_error: true,
                    };
                    return 
                        Ok(Ok(HttpResponse::Forbidden().json(resp)));
                }
    
                Ok(
                    user.
                        get_user_data_response_with_cookie(
                            &login_info.get_device_id(), 
                            redis_client.clone(), 
                            redis_actor, 
                            connection).await.unwrap()
                        )
            },
            Err(resp) => {

                let resp = Response::<String>{
                    data: Some(login_info.get_password()),
                    message: WRONG_IDENTIFIER,
                    status: 403,
                    is_error: true,
                };
                return 
                    Ok(Ok(HttpResponse::Forbidden().json(resp)));

            }
        }

    }

    // UserDataExt is only visible in this crate and can 
    // only be impl for other types only in here
    async fn create_passport<T: UserDataExt>(&self, new_user_info: T,
        connection: &mut DbPoolConnection) 
        -> Result<PanelHttpResponse, PanelHttpResponse>{
        
        let req = self as &Self::Request; // casting the self into a request object
        
        
        match User::find_by_identifier(&new_user_info.get_identifier().to_owned(), connection).await{
            Ok(user) => {

                let resp = Response::<String>{
                    data: Some(new_user_info.get_identifier()),
                    message: IDENTIFIER_ALREADY_EXISTS,
                    status: 406,
                    is_error: true,
                };
                return 
                    Ok(Ok(HttpResponse::NotAcceptable().json(resp)));

            },
            Err(resp) => {

                /* USER NOT FOUND response */
                // resp
                
                /* gently, we'll insert this user into table */
                match User::insert_by_identifier_password(new_user_info.get_identifier(), new_user_info.get_password(), connection).await{
                    Ok((user_login_data, cookie)) => {

                        let resp = Response::<UserData>{
                            data: Some(user_login_data),
                            message: REGISTERED,
                            status: 200,
                            is_error: true,
                        };
                        return 
                            Ok(Ok(HttpResponse::Created().json(resp)));

                    },
                    Err(resp) => {
                        
                        /* 
                            🥝 response can be one of the following:
                            
                            - DIESEL INSERT ERROR RESPONSE
                            - CANT_GENERATE_COOKIE
                        */
                        Err(resp)
                    }
                }

            }
        }

    }

}

// if we want to call methods of UserDataExt trait 
// on other struct instances outside of here we must
// make it public like using either pub or pub(crate)
// right now it's visible only in here and can be impl
// for structures only in this crate
pub(self) trait UserDataExt{
    type UserInfo;

    fn get_identifier(&self) -> String;
    fn get_password(&self) -> String;
    fn get_device_id(&self) -> String;
}

impl UserDataExt for UserLoginInfoRequest{
    type UserInfo = Self;
    
    // can't move out of a shared reference we have to clone it since 
    // self is behind a reference which is valid as long as the object
    // is valid
    fn get_identifier(&self) -> String {
        self.identifier.clone()
    }

    fn get_password(&self) -> String {
        self.password.clone()
    }

    fn get_device_id(&self) -> String{
        self.device_id.clone()
    }
}