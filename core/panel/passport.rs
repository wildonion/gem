


use crate::{*, 
    models::users::{UserData, User, UserRole, JWTClaims}, 
    constants::{FETCHED, EXPIRED_JWT, EXPIRED_REF_JWT, INVALID_REF_JWT}, misc::Response
};



/* 

    read more: https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html

    #[trait_variant::make(PassportSend: Send)] creates two versions of the trait: 
    Passport for single-threaded executors and PassportSend for multithreaded 
    work-stealing executors like so:
    pub trait PassportSend: Send {
        type Request;
        async fn get_user(&self, role: Option<UserRole>, 
            connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<JWTClaims, PanelHttpResponse>;

        fn check_refresh_token(&self, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<User, PanelHttpResponse>;
    }

    in the future this would be like:
    trait PassportSend = Passport<get_user(): Send> + Send;

*/
#[trait_variant::make(PassportSend: Send)] // Passport trait must be Send so we can call its async method in tokio runtime threads
pub trait Passport{

    type Request;
    async fn get_user(&self, role: Option<UserRole>, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<JWTClaims, PanelHttpResponse>;

    fn check_refresh_token(&self, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
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

    fn check_refresh_token(&self, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<User, PanelHttpResponse>{

        let req = self as &Self::Request;

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


                            let get_user = User::find_by_id_none_sync(_id, connection);
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
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<JWTClaims, PanelHttpResponse>{

        /*
            HttpRequest doesn't implement Copy thus dereferencing it
            is now allowed since dereferencing, moves out of the type
            itself and return the owned type which is not possible if 
            there is a pointer of the type exists and that type is not
            Copy, we must either use the borrow form of data or pass its 
            clone to other scopes
        */
        let req = self as &Self::Request;

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
        vec![String::from("")].into_iter()
    }

}