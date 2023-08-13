



use crate::*;
use crate::misc::{Response, gen_chars, gen_random_idx, gen_random_number};
use crate::schema::{users, users_tasks};
use crate::schema::users::dsl::*;
use crate::schema::users_tasks::dsl::*;
use crate::models::bot::Twitter;
use crate::constants::*;
use super::users_tasks::UserTask;



/* 

    diesel migration generate users ---> create users migration sql files
    diesel migration run            ---> apply sql files to db 
    diesel migration redo           ---> drop tables 

*/
#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct User{
    pub id: i32,
    pub username: String, /* unique */
    pub activity_code: String,
    pub twitter_username: Option<String>, /* unique */
    pub facebook_username: Option<String>, /* unique */
    pub discord_username: Option<String>, /* unique */
    pub identifier: Option<String>, /* unique */
    pub gmail: Option<String>, /* unique */
    pub phone_number: Option<String>, /* unique */
    pub paypal_id: Option<String>, /* unique */
    pub account_number: Option<String>, /* unique */
    pub device_id: Option<String>, /* unique */
    pub social_id: Option<String>, /* unique */
    pub cid: Option<String>, /* unique */
    pub snowflake_id: Option<i64>, /* unique */
    pub stars: Option<i64>,
    pub user_role: UserRole,
    pub pswd: String,
    pub token_time: Option<i64>,
    pub last_login: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=users)]
pub struct FetchUser{
    pub id: i32,
    pub username: String,
    pub activity_code: String,
    pub twitter_username: Option<String>,
    pub facebook_username: Option<String>,
    pub discord_username: Option<String>,
    pub identifier: Option<String>,
    pub gmail: Option<String>, /* unique */
    pub phone_number: Option<String>, /* unique */
    pub paypal_id: Option<String>, /* unique */
    pub account_number: Option<String>, /* unique */
    pub device_id: Option<String>, /* unique */
    pub social_id: Option<String>, /* unique */
    pub cid: Option<String>, /* unique */
    pub snowflake_id: Option<i64>, /* unique */
    pub stars: Option<i64>,
    pub user_role: UserRole,
    pub token_time: Option<i64>,
    pub last_login: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct UserData{
    pub id: i32,
    pub username: String,
    pub activity_code: String,
    pub twitter_username: Option<String>,
    pub facebook_username: Option<String>,
    pub discord_username: Option<String>,
    pub identifier: Option<String>,
    pub gmail: Option<String>, /* unique */
    pub phone_number: Option<String>, /* unique */
    pub paypal_id: Option<String>, /* unique */
    pub account_number: Option<String>, /* unique */
    pub device_id: Option<String>, /* unique */
    pub social_id: Option<String>, /* unique */
    pub cid: Option<String>, /* unique */
    pub snowflake_id: Option<i64>, /* unique */
    pub stars: Option<i64>,
    pub user_role: String,
    pub token_time: Option<i64>,
    pub last_login: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct DepositRequest{
    pub from_cid: String,
    pub recipient_cid: String,
    pub amount: u64,
    pub signature: String, /* this must be generated inside the client by signing the operation using the client private key */
    pub iat: i64, // deposited at
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct WithdrawRequest{
    pub deposited_id: i32,
    pub cid: String,
    pub signature: String, /* this must be generated inside the client by signing the operation using the client private key */
    pub cat: i64, // claimed at
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct UserIdResponse{
    pub id: i32,
    pub username: String,
    pub activity_code: String,
    pub twitter_username: Option<String>,
    pub facebook_username: Option<String>,
    pub discord_username: Option<String>,
    pub identifier: Option<String>,
    pub gmail: Option<String>, /* unique */
    pub phone_number: Option<String>, /* unique */
    pub paypal_id: Option<String>, /* unique */
    pub account_number: Option<String>, /* unique */
    pub device_id: Option<String>, /* unique */
    pub social_id: Option<String>, /* unique */
    pub cid: Option<String>, /* unique */
    pub snowflake_id: Option<i64>, /* unique */
    pub stars: Option<i64>,
    pub signer: Option<String>,
    pub user_role: String,
    pub token_time: Option<i64>,
    pub last_login: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct NewIdRequest{
    pub gmail: String,
    pub username: String,
    pub phone_number: String,
    pub paypal_id: String,
    pub account_number: String,
    pub device_id: String,
    pub social_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, ToSchema)]
pub struct Id{
    pub user_gmail: String,
    pub user_phone_number: String,
    pub user_id: i32,
    pub paypal_id: String,
    pub account_number: String,
    pub device_id: String,
    pub social_id: String,
    pub username: String,
    pub new_snowflake_id: Option<i64>,
    pub new_cid: Option<String>, /* pubkey */
    pub signer: Option<String>, /* prvkey */
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, Default)]
pub struct LoginInfoRequest{
    pub username: String,
    pub password: String
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, Default)]
pub struct UserLoginInfoRequest{
    pub identifier: String,
    pub password: String
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[derive(diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::Userrole"]
pub enum UserRole{
    Admin,
    User,
    Dev
}

#[derive(Insertable)]
#[diesel(table_name=users)]
pub struct NewUser<'l> {
    pub username: &'l str,
    pub activity_code: &'l str,
    pub identifier: &'l str,
    pub user_role: UserRole,
    pub pswd: &'l str,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name=users)]
#[derive(Clone, Debug)]
pub struct EditUserByAdmin<'p>{
    pub user_role: UserRole,
    pub username: &'p str,
    pub identifier: &'p str,
    pub pswd: &'p str
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTClaims{
    pub _id: i32, // mongodb object id
    pub username: Option<String>,
    pub identifier: Option<String>,
    pub user_role: UserRole,
    pub token_time: i64,
    pub exp: i64, // expiration timestamp
    pub iat: i64, // issued timestamp
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct NewUserInfoRequest{
    pub username: String,
    pub identifier: String,
    pub role: String,
    pub password: String
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct EditUserByAdminRequest{
    pub user_id: i32,
    pub role: String,
    pub username: String,
    pub identifier: String,
    pub password: Option<String>
}

impl User{

    pub fn decode_token(token: &str) -> Result<TokenData<JWTClaims>, jsonwebtoken::errors::Error>{
        let encoding_key = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
        let decoded_token = decode::<JWTClaims>(token, &DecodingKey::from_secret(encoding_key.as_bytes()), &Validation::new(Algorithm::HS512));
        decoded_token
    }

    pub const SCHEMA_NAME: &str = "User";

    pub async fn passport(req: HttpRequest, pass_role: Option<UserRole>, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<JWTClaims, PanelHttpResponse>{

        let mut jwt_flag = false;
        let mut cookie_flag = false;
        let mut jwt_token = ""; 

        if let Some(authen_header) = req.headers().get("Authorization"){
            if let Ok(authen_str) = authen_header.to_str(){
                if authen_str.starts_with("bearer") || authen_str.starts_with("Bearer"){
                    let token = authen_str[6..authen_str.len()].trim();

                    jwt_flag = true;
                    jwt_token = token;

                }
            }
        } else{
            /* checking that the request cookie has the jwt key */
            if let Some(_) = req.cookie("jwt"){

                cookie_flag = true;

            }
        }


        if !jwt_flag && !cookie_flag {
            
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: NOT_FOUND_COOKIE_VALUE_OR_JWT,
                status: 404
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );

        }

        if jwt_flag{

             /* decoding the jwt */
            let token_result = User::decode_token(jwt_token);
            
            match token_result{
                Ok(token) => {

                    /* cookie time is not expired yet */
                    let token_data = token.claims;
                    let _id = token_data._id;
                    let role = token_data.user_role.clone();
                    let _token_time = token_data.token_time;

                    /* fetch user info based on the data inside jwt */ 
                    let single_user = users
                        .filter(id.eq(_id))
                        .first::<User>(connection);

                    if single_user.is_err(){
                        let resp = Response{
                            data: Some(_id.to_owned()),
                            message: USER_NOT_FOUND,
                            status: 404
                        };
                        return Err(
                            Ok(HttpResponse::NotFound().json(resp))
                        );
                    }

                    let user = single_user.unwrap();

                    /* 
                        check that the user is authorized with the 
                        passed in role and the one inside the jwt
                        since some of the routes require role guard 
                    */
                    if pass_role.is_some(){
                        if user.user_role != pass_role.unwrap() &&
                            user.user_role != role{
                            let resp = Response{
                                data: Some(_id.to_owned()),
                                message: ACCESS_DENIED,
                                status: 403
                            };
                            return Err(
                                Ok(HttpResponse::Forbidden().json(resp))
                            );
                        } 
                    }

                    /*
                        if the current token time of the fetched user 
                        wasn't equal to the one inside the passed in JWT
                        into the request header means that the user did
                        a logout or did a login again since by logging out 
                        the token time will be set to zero and by logging 
                        in again a new token time will be initialized.
                    */
                    if user.token_time.is_none() && /* means that the user has passed an invalid token that haven't a token time which means it doesn't belong to the user him/her-self */
                        user.token_time.unwrap() != _token_time{
                        
                        let resp = Response{
                            data: Some(_id.to_owned()),
                            message: DO_LOGIN, /* comple the user to login again to set a new token time in his/her jwt */
                            status: 403
                        };
                        return Err(
                            Ok(HttpResponse::Forbidden().json(resp))
                        );
                        
                    }

                    /* returning token data, if we're here means that nothing went wrong */
                    return Ok(token_data);

                },
                Err(e) => {
                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: &e.to_string(),
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );
                }
            };
 
        } else{ /* surely we have cookie since the jwt flag is not true */

            let cookie = req.cookie("jwt").unwrap();

            let expire_datetime = cookie.expires_datetime();
            let cookie_time_hash;
            let token;

            /* parsing the jwt value inside the cookie */
            let jwt = cookie.value();
            if jwt.contains("::"){
                let mut splitted_token = jwt.split("::");
                let Some(cookie_token) = splitted_token.next() else{
                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: NOT_FOUND_TOKEN,
                        status: 403
                    };
                    return Err(
                        Ok(HttpResponse::Forbidden().json(resp))
                    );
                };
                token = cookie_token;

                let Some(cth) = splitted_token.next() else{
                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: NOT_FOUND_COOKIE_TIME_HASH,
                        status: 406
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    );
                };
                cookie_time_hash = cth;
            } else{
                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: INVALID_COOKIE_FORMAT,
                    status: 406
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                );
            }

            /* decoding the jwt */
            let token_result = User::decode_token(token);
            
            match token_result{
                Ok(token) => {

                    /* checking the expiration time inside the cookie */
                    match expire_datetime{
                        Some(exp) => {

                            let now = OffsetDateTime::now_utc();
                            
                            /* we have an expired cookie */
                            if exp >= now{
                                let resp = Response::<&[u8]>{
                                    data: Some(&[]),
                                    message: EXPIRED_COOKIE,
                                    status: 403
                                };
                                return Err(
                                    Ok(HttpResponse::Forbidden().json(resp))
                                );
                            } else{

                                /* cookie time is not expired yet */
                                let token_data = token.claims;
                                let _id = token_data._id;
                                let role = token_data.user_role.clone();

                                /* fetch user info based on the data inside jwt */ 
                                let single_user = users
                                    .filter(id.eq(_id))
                                    .first::<User>(connection);

                                if single_user.is_err(){
                                    let resp = Response{
                                        data: Some(_id.to_owned()),
                                        message: USER_NOT_FOUND,
                                        status: 404
                                    };
                                    return Err(
                                        Ok(HttpResponse::NotFound().json(resp))
                                    );
                                }

                                let user = single_user.unwrap();

                                /* check that the time hash of the cookie is valid */
                                if !user.check_cookie_time_hash(cookie_time_hash){
                                    let resp = Response::<&[u8]>{
                                        data: Some(&[]),
                                        message: INVALID_COOKIE_TIME_HASH,
                                        status: 406
                                    };
                                    return Err(
                                        Ok(HttpResponse::NotAcceptable().json(resp))
                                    );
                                }

                                /* check that the user is authorized with the passed in role */
                                if pass_role.is_some(){
                                    if user.user_role != pass_role.unwrap(){
                                        let resp = Response{
                                            data: Some(_id.to_owned()),
                                            message: ACCESS_DENIED,
                                            status: 403
                                        };
                                        return Err(
                                            Ok(HttpResponse::Forbidden().json(resp))
                                        );
                                    } 
                                }

                                /* returning token data, if we're here means that nothing went wrong */
                                return Ok(token_data);
                            }
            
                        },
                        None => {
                            let resp = Response::<&[u8]>{
                                data: Some(&[]),
                                message: NOT_FOUND_COOKIE_EXP,
                                status: 406
                            };
                            return Err(
                                Ok(HttpResponse::NotAcceptable().json(resp))
                            );
                        }
                    }

                },
                Err(e) => {
                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: &e.to_string(),
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );
                }
            }

        }
    }

    fn generate_token(&self, _token_time: i64) -> Result<String, jsonwebtoken::errors::Error>{
        
        let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nano to sec
        let exp_time = now + env::var("JWT_EXPIRATION").expect("⚠️ found no jwt expiration time").parse::<i64>().unwrap();
        
        let payload = JWTClaims{
            _id: self.id,
            username: Some(self.username.clone()), /* here username and user_role are behind a reference which can't be moved thus we must clone them */
            identifier: self.identifier.clone(),
            user_role: self.user_role.clone(),
            token_time: _token_time,
            exp: exp_time,
            iat: now
        };
    
        let encoding_key = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
        let token = encode(&Header::new(Algorithm::HS512), &payload, &EncodingKey::from_secret(encoding_key.as_bytes()));
        token
    
    }

    /* 
        since self is not behind & thus the Cookie can't use the lifetime of the self reference hence we 
        must specify the 'static lifetime for the Cookie also the reason that the self is not behind a pointer
        is because this function returns a Cookie instance which takes a valid lifetime in which we can't return
        it from the the caller space of this method since rust says can't returns a value referencing data owned by 
        the current function means that the returned cookie instance from here to the caller space has a reference 
        to the instance of User struct in which we can't return the cookie instance from the caller scope to other scopes
        in other words we can't return reference to a data which is owned by the current function. 
    */
    pub fn generate_cookie_and_jwt(self) -> Option<(Cookie<'static>, i64, String)>{

        /*
            since cookie can be stored inside the request object thus for peers on the same network 
            which have an equal ip address they share a same cookie thus we'll face the bug of which 
            every user can be every user in which they can see other peer's jwt info inside their browser!
        */
        let time_hash_now = chrono::Local::now().timestamp_nanos();
        let time_hash_now_now_str = format!("{}", time_hash_now);
        let mut hasher = Sha256::new();
        hasher.update(time_hash_now_now_str.as_str());
        let time_hash = hasher.finalize();

        /* every 2 chars is 1 byte thus in sha256 we have 32 bytes elements which is 64 chars in hex */
        let time_hash_hex_string = time_hash
                                        .into_iter()
                                        .map(|byte| format!("{:02x}", byte))
                                        .collect::<String>();

        /* if we're here means that the password was correct */
        let token = self.generate_token(time_hash_now).unwrap();
        
        let cookie_value = format!("{token:}::{time_hash_hex_string:}");
        let mut cookie = Cookie::build("jwt", cookie_value)
                                    .same_site(cookie::SameSite::Strict)
                                    .secure(true)
                                    .finish();
        let cookie_exp_days = env::var("COOKIE_EXPIRATION_DAYS").expect("⚠️ no cookie exporation days variable set").parse::<i64>().unwrap();
        let mut now = OffsetDateTime::now_utc();
        now += Duration::days(cookie_exp_days);
        cookie.set_expires(now);

        Some((cookie, time_hash_now, token))

    }

    fn check_cookie_time_hash(&self, cookie_time_hash: &str) -> bool{
        
        let Some(utt) = self.token_time else{
            return false;
        };

        let utt_string = format!("{}", utt);
        let mut hasher = Sha256::new();
        hasher.update(utt_string.as_str());
        let time_hash = hasher.finalize();

        /* every 2 chars is 1 byte thus in sha256 we have 32 bytes elements which is 64 chars in hex */
        let time_hash_hex_string = time_hash
                                        .into_iter()
                                        .map(|byte| format!("{:02x}", byte))
                                        .collect::<String>();

        if time_hash_hex_string == cookie_time_hash{
            true
        } else{
            false
        }

    }

    pub fn hash_pswd(password: &str) -> Result<String, argon2::Error>{
        let salt = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
        let salt_bytes = salt.as_bytes();
        let password_bytes = password.as_bytes();
        argon2::hash_encoded(password_bytes, salt_bytes, &argon2::Config::default())
    }

    pub fn verify_pswd(&self, raw_pswd: &str) -> Result<bool, argon2::Error>{
        let password_bytes = raw_pswd.as_bytes();
        Ok(argon2::verify_encoded(&self.pswd, password_bytes).unwrap())
    }

    pub async fn find_by_username(user_name: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Self, PanelHttpResponse>{

        let single_user = users
            .filter(username.eq(user_name.to_string()))
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(user_name),
                message: USER_NOT_FOUND,
                status: 404
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub async fn find_by_identifier(identifier_login: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Self, PanelHttpResponse>{

        let single_user = users
            .filter(identifier.eq(identifier_login.to_string()))
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(identifier_login),
                message: USER_NOT_FOUND,
                status: 404
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub async fn find_by_id(doer_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Self, PanelHttpResponse>{

        let single_user = users
            .filter(users::id.eq(doer_id))
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(doer_id),
                message: USER_NOT_FOUND,
                status: 404
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub async fn insert(identifier_login: String, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<(UserData, Cookie), PanelHttpResponse>{

        let random_chars = gen_chars(gen_random_number(5, 11));
        let random_code: String = (0..5).map(|_|{
            let idx = gen_random_idx(random::<u8>() as usize); // idx is one byte cause it's of type u8
            CHARSET[idx] as char // CHARSET is of type utf8 bytes thus we can index it which it's length is 10 bytes (0-9)
        }).collect();

        let new_user = NewUser{
            username: &identifier_login, /* first insert the username is the identifier address */
            activity_code: &random_code,
            identifier: &identifier_login,
            user_role: UserRole::User,
            pswd: "",
        };
        
        match diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result::<User>(connection)
            {
                Ok(fetched_user) => {

                    let user_login_data = UserData{
                        id: fetched_user.id,
                        username: fetched_user.username.clone(),
                        activity_code: fetched_user.activity_code.clone(),
                        twitter_username: fetched_user.twitter_username.clone(),
                        facebook_username: fetched_user.facebook_username.clone(),
                        discord_username: fetched_user.discord_username.clone(),
                        identifier: fetched_user.identifier.clone(),
                        user_role: {
                            match fetched_user.user_role.clone(){
                                UserRole::Admin => "Admin".to_string(),
                                UserRole::User => "User".to_string(),
                                _ => "Dev".to_string(),
                            }
                        },
                        token_time: fetched_user.token_time,
                        last_login: { 
                            if fetched_user.last_login.is_some(){
                                Some(fetched_user.last_login.unwrap().to_string())
                            } else{
                                Some("".to_string())
                            }
                        },
                        created_at: fetched_user.created_at.to_string(),
                        updated_at: fetched_user.updated_at.to_string(),
                        gmail: fetched_user.clone().gmail,
                        phone_number: fetched_user.clone().phone_number,
                        paypal_id: fetched_user.clone().paypal_id,
                        account_number: fetched_user.clone().account_number,
                        device_id: fetched_user.clone().device_id,
                        social_id: fetched_user.clone().social_id,
                        cid: fetched_user.clone().cid,
                        snowflake_id: fetched_user.snowflake_id,
                        stars: fetched_user.stars
                    };

                    /* generate cookie 🍪 from token time and jwt */
                    /* since generate_cookie_and_jwt() takes the ownership of the user instance we must clone it then call this */
                    /* generate_cookie_and_jwt() returns a Cookie instance with a 'static lifetime which allows us to return it from here*/
                    let Some(cookie_info) = fetched_user.clone().generate_cookie_and_jwt() else{
                        let resp = Response::<&[u8]>{
                            data: Some(&[]),
                            message: CANT_GENERATE_COOKIE,
                            status: 500
                        };
                        return Err(
                            Ok(HttpResponse::InternalServerError().json(resp))
                        );
                    };
                    
                    let cookie = cookie_info.0;
                    let cookie_token_time = cookie_info.1;

                    /* update the login token time */
                    let now = chrono::Local::now().naive_local();
                    let updated_user = diesel::update(users.find(fetched_user.id))
                        .set((last_login.eq(now), token_time.eq(cookie_token_time)))
                        .execute(connection)
                        .unwrap();
                    
                    Ok((user_login_data, cookie))

                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec(); /* extend the empty msg_content from the error utf8 slice */
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)));
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

    pub async fn insert_by_identifier_password(identifier_login: String, password: String, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<(UserData, Cookie), PanelHttpResponse>{

        let random_chars = gen_chars(gen_random_number(5, 11));
        let random_code: String = (0..5).map(|_|{
            let idx = gen_random_idx(random::<u8>() as usize); // idx is one byte cause it's of type u8
            CHARSET[idx] as char // CHARSET is of type utf8 bytes thus we can index it which it's length is 10 bytes (0-9)
        }).collect();

        let pass = User::hash_pswd(password.as_str()).unwrap();
        let new_user = NewUser{
            username: &identifier_login, /* first insert the username is the identifier address */
            activity_code: &random_code,
            identifier: &identifier_login,
            user_role: UserRole::User,
            pswd: &pass
        };
        
        match diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result::<User>(connection)
            {
                Ok(fetched_user) => {

                    let user_login_data = UserData{
                        id: fetched_user.id,
                        username: fetched_user.username.clone(),
                        activity_code: fetched_user.activity_code.clone(),
                        twitter_username: fetched_user.twitter_username.clone(),
                        facebook_username: fetched_user.facebook_username.clone(),
                        discord_username: fetched_user.discord_username.clone(),
                        identifier: fetched_user.identifier.clone(),
                        user_role: {
                            match fetched_user.user_role.clone(){
                                UserRole::Admin => "Admin".to_string(),
                                UserRole::User => "User".to_string(),
                                _ => "Dev".to_string(),
                            }
                        },
                        token_time: fetched_user.token_time,
                        last_login: { 
                            if fetched_user.last_login.is_some(){
                                Some(fetched_user.last_login.unwrap().to_string())
                            } else{
                                Some("".to_string())
                            }
                        },
                        created_at: fetched_user.created_at.to_string(),
                        updated_at: fetched_user.updated_at.to_string(),
                        gmail: fetched_user.clone().gmail,
                        phone_number: fetched_user.clone().phone_number,
                        paypal_id: fetched_user.clone().paypal_id,
                        account_number: fetched_user.clone().account_number,
                        device_id: fetched_user.clone().device_id,
                        social_id: fetched_user.clone().social_id,
                        cid: fetched_user.clone().cid,
                        snowflake_id: fetched_user.snowflake_id,
                        stars: fetched_user.stars
                    };

                    /* generate cookie 🍪 from token time and jwt */
                    /* since generate_cookie_and_jwt() takes the ownership of the user instance we must clone it then call this */
                    /* generate_cookie_and_jwt() returns a Cookie instance with a 'static lifetime which allows us to return it from here*/
                    let Some(cookie_info) = fetched_user.clone().generate_cookie_and_jwt() else{
                        let resp = Response::<&[u8]>{
                            data: Some(&[]),
                            message: CANT_GENERATE_COOKIE,
                            status: 500
                        };
                        return Err(
                            Ok(HttpResponse::InternalServerError().json(resp))
                        );
                    };
                    
                    let cookie = cookie_info.0;
                    let cookie_token_time = cookie_info.1;

                    /* update the login token time */
                    let now = chrono::Local::now().naive_local();
                    let updated_user = diesel::update(users.find(fetched_user.id))
                        .set((last_login.eq(now), token_time.eq(cookie_token_time)))
                        .execute(connection)
                        .unwrap();
                    
                    Ok((user_login_data, cookie))

                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec(); /* extend the empty msg_content from the error utf8 slice */
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)));
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

    pub async fn insert_new_user(user: NewUserInfoRequest, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: &RedisClient
        ) -> Result<usize, PanelHttpResponse>{

        let hash_pswd = User::hash_pswd(user.password.as_str()).unwrap();
        let u_name = user.username.as_str();
        let identifier_login = user.identifier.as_str();
        let uname = if u_name == ""{
            chrono::Local::now().timestamp_nanos().to_string()
        } else{
            u_name.to_string()
        };

        let user = NewUser{
            username: &uname,
            activity_code: "",
            identifier: {
                if identifier_login == ""{
                    &uname
                } else{
                    identifier_login
                }
            },
            user_role: match user.role.as_str(){
                "Admin" => UserRole::Admin,
                "Dev" => UserRole::Dev,
                _ => UserRole::User
            },
            pswd: hash_pswd.as_str()
        };

        match diesel::insert_into(users::table)
            .values(&user)
            .execute(connection)
            {
                Ok(affected_row) => Ok(affected_row),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec(); /* extend the empty msg_content from the error utf8 slice */
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)));
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

    pub async fn edit_by_admin(new_user: EditUserByAdminRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserData, PanelHttpResponse>{

        /* fetch user info based on the data inside jwt */ 
        let single_user = users
            .filter(users::id.eq(new_user.user_id.to_owned()))
            .first::<User>(connection);

        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(new_user.user_id.to_owned()),
                message: USER_NOT_FOUND,
                status: 404
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };
        
        /* if the passed in password was some then we must updated the password */
        let password = if let Some(password) = &new_user.password{ // borrowing the user to prevent from moving

            /* we can pass &str to the method by borrowing the String since String will be coerced into &str at compile time */
            User::hash_pswd(password).unwrap()

        } else{
            
            /* if the passed in password was none then we must use the old one */
            user.pswd

        };


        let _username = if new_user.username != "".to_string(){
            &new_user.username
        } else{
            let resp = Response{
                data: Some(new_user.user_id.to_owned()),
                message: USERNAME_CANT_BE_EMPTY,
                status: 406
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        let _identifier = if new_user.identifier != "".to_string(){
            &new_user.identifier
        } else{
            let resp = Response{
                data: Some(new_user.user_id.to_owned()),
                message: WALLET_CANT_BE_EMPTY,
                status: 406
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };


        match diesel::update(users.find(new_user.user_id.to_owned()))
            .set(EditUserByAdmin{
                user_role: {
                    let role = new_user.role.as_str(); 
                    match role{
                        "User" => UserRole::User,
                        "Admin" => UserRole::Admin,
                        _ => UserRole::Dev
                    }
                },
                /* 
                    pswd, username and identifier is of type &str thus by borrowing these 
                    feilds from new_user instance we can convert them into &str 
                */
                pswd: &password,
                username: &_username,
                identifier: &_identifier
            })
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {
                    Ok(
                        UserData { 
                            id: updated_user.id, 
                            username: updated_user.username, 
                            activity_code: updated_user.activity_code, 
                            twitter_username: updated_user.twitter_username, 
                            facebook_username: updated_user.facebook_username, 
                            discord_username: updated_user.discord_username, 
                            identifier: updated_user.identifier, 
                            user_role: {
                                match updated_user.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: updated_user.token_time,
                            last_login: { 
                                if updated_user.last_login.is_some(){
                                    Some(updated_user.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: updated_user.created_at.to_string(),
                            updated_at: updated_user.updated_at.to_string(),
                            gmail: updated_user.gmail,
                            phone_number: updated_user.phone_number,
                            paypal_id: updated_user.paypal_id,
                            account_number: updated_user.account_number,
                            device_id: updated_user.device_id,
                            social_id: updated_user.social_id,
                            cid: updated_user.cid,
                            snowflake_id: updated_user.snowflake_id,
                            stars: updated_user.stars
                        }
                    )
                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec(); /* extend the empty msg_content from the error utf8 slice */
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)));
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

    pub async fn delete_by_admin(doer_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<usize, PanelHttpResponse>{

        /* we must first delete from users_tasks */
        
        match UserTask::delete_by_doer(doer_id, connection).await {
            Ok(users_tasks_rows_deleted) => {

                match diesel::delete(users.filter(users::id.eq(doer_id.to_owned())))
                    .execute(connection)
                    {
                        Ok(mut num_deleted) => {
                            
                            /* also delete any users record if there was any */

                            num_deleted += users_tasks_rows_deleted;

                            Ok(num_deleted)
                        
                        },
                        Err(e) => {

                            let resp_err = &e.to_string();


                            /* custom error handler */
                            use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                             
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec(); /* extend the empty msg_content from the error utf8 slice */
                            let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)));
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

            },
            Err(e) => {
                
                return Err(e);
            }
        }

    
    }

    pub async fn get_all(connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<UserData>, PanelHttpResponse>{

        match users.load::<User>(connection)
        {
            Ok(all_users) => {
                Ok(
                    all_users
                        .into_iter()
                        .map(|u| UserData { 
                            id: u.id, 
                            username: u.username, 
                            activity_code: u.activity_code, 
                            twitter_username: u.twitter_username, 
                            facebook_username: u.facebook_username, 
                            discord_username: u.discord_username, 
                            identifier: u.identifier, 
                            user_role: {
                                match u.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: u.token_time,
                            last_login: { 
                                if u.last_login.is_some(){
                                    Some(u.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: u.created_at.to_string(),
                            updated_at: u.updated_at.to_string(),
                            gmail: u.gmail,
                            phone_number: u.phone_number,
                            paypal_id: u.paypal_id,
                            account_number: u.account_number,
                            device_id: u.device_id,
                            social_id: u.social_id,
                            cid: u.cid,
                            snowflake_id: u.snowflake_id,
                            stars: u.stars
                        })
                        .collect::<Vec<UserData>>()
                )
            },
            Err(e) => {

                let resp_err = &e.to_string();


                /* custom error handler */
                use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                 
                let error_content = &e.to_string();
                let error_content = error_content.as_bytes().to_vec(); /* extend the empty msg_content from the error utf8 slice */
                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)));
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

    pub async fn logout(who: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<(), PanelHttpResponse>{

        match diesel::update(users.find(who))
            .set(token_time.eq(0))
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => Ok(()),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec(); /* extend the empty msg_content from the error utf8 slice */
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)));
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

    pub async fn update_social_account(
        identifier_login: &str, 
        account_name: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserData, PanelHttpResponse>{


            let Ok(user) = User::find_by_identifier(identifier_login, connection).await else{
                let resp = Response{
                    data: Some(identifier_login),
                    message: USER_NOT_FOUND,
                    status: 404
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                );
            };

            // let bot_endpoint = env::var("THIRD_PARY_TWITTER_BOT_ENDPOINT").expect("⚠️ no twitter bot endpoint key variable set");
            // let new_twitter = Twitter::new(Some(bot_endpoint)).await;
            let new_twitter = Twitter::new(None).await;
            let Ok(bot) =  new_twitter else{
                return Err(new_twitter.unwrap_err());
            };

            let is_user_verified = bot.is_twitter_user_verified(account_name, connection).await;
            let Ok(is_verified) = is_user_verified else{
                return Err(is_user_verified.unwrap_err());
            };

            if is_verified{
                
                match diesel::update(users.find(user.id))
                    .set(twitter_username.eq(account_name.to_lowercase()))
                    .returning(FetchUser::as_returning())
                    .get_result(connection)
                    {
                        Ok(updated_user) => {
                            Ok(
                                UserData { 
                                    id: updated_user.id, 
                                    username: updated_user.username, 
                                    activity_code: updated_user.activity_code, 
                                    twitter_username: updated_user.twitter_username, 
                                    facebook_username: updated_user.facebook_username, 
                                    discord_username: updated_user.discord_username, 
                                    identifier: updated_user.identifier, 
                                    user_role: {
                                        match updated_user.user_role.clone(){
                                            UserRole::Admin => "Admin".to_string(),
                                            UserRole::User => "User".to_string(),
                                            _ => "Dev".to_string(),
                                        }
                                    },
                                    token_time: updated_user.token_time,
                                    last_login: { 
                                        if updated_user.last_login.is_some(){
                                            Some(updated_user.last_login.unwrap().to_string())
                                        } else{
                                            Some("".to_string())
                                        }
                                    },
                                    created_at: updated_user.created_at.to_string(),
                                    updated_at: updated_user.updated_at.to_string(),
                                    gmail: updated_user.gmail,
                                    phone_number: updated_user.phone_number,
                                    paypal_id: updated_user.paypal_id,
                                    account_number: updated_user.account_number,
                                    device_id: updated_user.device_id,
                                    social_id: updated_user.social_id,
                                    cid: updated_user.cid,
                                    snowflake_id: updated_user.snowflake_id,
                                    stars: updated_user.stars
                                }
                            )
                        },
                        Err(e) => {
                            
                            let resp_err = &e.to_string();


                            /* custom error handler */
                            use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                             
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec(); /* extend the empty msg_content from the error utf8 slice */
                            let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)));
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

                let resp = Response{
                    data: Some(identifier_login),
                    message: TWITTER_USER_IS_NOT_VALID,
                    status: 406
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                );

            }


    }

}


impl Id{

    pub async fn new_or_update(id_: NewIdRequest, id_owner: i32, id_username: String,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Id, PanelHttpResponse>{

        let Ok(user) = User::find_by_id(id_owner, connection).await else{
            let resp = Response{
                data: Some(id_owner),
                message: USER_NOT_FOUND,
                status: 404
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            ); 
        };

        match user.cid{
            /* we'll be here only if the old_cid is not an empty string */
            Some(old_cid) if !old_cid.is_empty() => { 
                
                /* updating other fields except cid and snowflake id */
                match diesel::update(users.find(id_owner))
                    .set(
                (
                            gmail.eq(id_.gmail.clone()),
                            phone_number.eq(id_.phone_number.clone()),
                            username.eq(id_.username.clone()),
                            paypal_id.eq(id_.paypal_id.clone()),
                            account_number.eq(id_.account_number.clone()),
                            device_id.eq(id_.device_id.clone()),
                            social_id.eq(id_.social_id.clone()),
                        )
                    )
                    .returning(FetchUser::as_returning())
                    .get_result(connection)
                    {
                        Ok(updated_user) => {

                            let user_data = UserData { 
                                id: updated_user.id, 
                                username: updated_user.username, 
                                activity_code: updated_user.activity_code, 
                                twitter_username: updated_user.twitter_username, 
                                facebook_username: updated_user.facebook_username, 
                                discord_username: updated_user.discord_username, 
                                identifier: updated_user.identifier, 
                                user_role: {
                                    match updated_user.user_role.clone(){
                                        UserRole::Admin => "Admin".to_string(),
                                        UserRole::User => "User".to_string(),
                                        _ => "Dev".to_string(),
                                    }
                                },
                                token_time: updated_user.token_time,
                                last_login: { 
                                    if updated_user.last_login.is_some(){
                                        Some(updated_user.last_login.unwrap().to_string())
                                    } else{
                                        Some("".to_string())
                                    }
                                },
                                created_at: updated_user.created_at.to_string(),
                                updated_at: updated_user.updated_at.to_string(),
                                gmail: updated_user.gmail,
                                phone_number: updated_user.phone_number,
                                paypal_id: updated_user.paypal_id,
                                account_number: updated_user.account_number,
                                device_id: updated_user.device_id,
                                social_id: updated_user.social_id,
                                cid: updated_user.cid,
                                snowflake_id: updated_user.snowflake_id,
                                stars: updated_user.stars
                            };

                            let resp = Response{
                                data: Some(user_data),
                                message: CID_RECORD_UPDATED,
                                status: 200
                            };
                            return Err(
                                Ok(HttpResponse::Found().json(resp))
                            ); 
                        },
                        Err(e) => {
                            
                            let resp_err = &e.to_string();

                            /* custom error handler */
                            use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                                
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec(); /* extend the empty msg_content from the error utf8 slice */
                            let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)));
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

            },
            _ => {

                /* ECDSA keypair */
                let ec_key_pair = gen_ec_key_pair(); // generates a pair of Elliptic Curve (ECDSA) keys
                let (private, public) = ec_key_pair.clone().split();
                let new_cid = Some(hex::encode(public.as_ref()));
                let signer = Some(hex::encode(private.as_ref()));

                /* generating snowflake id */
                let machine_id = std::env::var("MACHINE_ID").unwrap_or("1".to_string()).parse::<i32>().unwrap();
                let node_id = std::env::var("NODE_ID").unwrap_or("1".to_string()).parse::<i32>().unwrap();
                let mut id_generator_generator = SnowflakeIdGenerator::new(machine_id, node_id);
                let new_snowflake_id = id_generator_generator.real_time_generate();
                let new_snowflake_id = Some(new_snowflake_id);

                Ok(
                    Id { 
                        user_gmail: id_.gmail,
                        user_phone_number: id_.phone_number,
                        user_id: id_owner,
                        username: id_username, 
                        paypal_id: id_.paypal_id, 
                        account_number: id_.account_number, 
                        device_id: id_.device_id, 
                        social_id: id_.social_id, 
                        new_snowflake_id,
                        new_cid: Some(format!("0x{}", new_cid.unwrap())),
                        signer: Some(format!("0x{}", signer.unwrap()))
                    }
                )

            }
        } 

    }

    pub fn retrieve_keypair(hex_pubkey: &str, hex_prvkey: &str) -> themis::keys::KeyPair{

        /* building ECDSA keypair from pubkey and prvkey slices */
        let pubkey_bytes = hex::decode(hex_pubkey).unwrap();
        let prvkey_bytes = hex::decode(hex_prvkey).unwrap();
        let ec_pubkey = EcdsaPublicKey::try_from_slice(&pubkey_bytes).unwrap();
        let ec_prvkey = EcdsaPrivateKey::try_from_slice(&prvkey_bytes).unwrap();
        let generated_ec_keypair = ThemisKeyPair::try_join(ec_prvkey, ec_pubkey).unwrap();
        generated_ec_keypair

    }

    pub fn test_sign(&mut self) -> Option<String>{

        /* building the signer from the private key */
        let prvkey_bytes = hex::decode(self.signer.as_ref().unwrap()).unwrap();
        let ec_prvkey = EcdsaPrivateKey::try_from_slice(&prvkey_bytes).unwrap();
        let ec_signer = SecureSign::new(ec_prvkey.clone());

        /* stringifying the object_id instance to generate the signature */
        let json_input = serde_json::json!({
            "gmail": self.user_gmail,
            "phone_number": self.user_phone_number,
            "user_id": self.user_id,
            "paypal_id": self.paypal_id,
            "account_number": self.account_number,
            "social_id": self.social_id,
            "username": self.username,
            "new_snowflake_id": self.new_snowflake_id,
            "device_id": self.device_id.clone(),
            "new_cid": self.new_cid.as_ref().unwrap(), /* unwrap() takes the ownership of self thus we've used as_ref() to prevent from moving */
        });
        
        /* json stringifying the json_input value */
        let inputs_to_sign = serde_json::to_string(&json_input).unwrap(); 
    
        /* generating signature from the input data */
        let ec_sig = ec_signer.sign(inputs_to_sign.as_bytes()).unwrap();
        
        /* converting the signature byte into hex string */
        Some(hex::encode(&ec_sig))

    }

    pub fn verify(signature: &[u8], pubkey: &[u8]) -> Result<Vec<u8>, themis::Error>{

        /* building the public key from public key bytes */
        let Ok(ec_pubkey) = EcdsaPublicKey::try_from_slice(pubkey) else{
            let err = EcdsaPublicKey::try_from_slice(pubkey).unwrap_err();
            return Err(err);
        };

        /* building the verifier from the public key */
        let ec_verifier = SecureVerify::new(ec_pubkey.clone());

        /* verifying the signature byte which returns the data itself in form of utf8 bytes */
        let encoded_data = ec_verifier.verify(signature);

        encoded_data

    }

    pub async fn save(&mut self, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserIdResponse, PanelHttpResponse>{
        
        let Ok(user) = User::find_by_id(self.user_id, connection).await else{
            let resp = Response{
                data: Some(self.user_id),
                message: USER_NOT_FOUND,
                status: 404
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );  
        };

        let hex_cid = Some(format!("0x{}", self.new_cid.clone().unwrap()));
        match diesel::update(users.find(self.user_id))
            .set(
        (   
                /* 
                    can't return heap data like self.user_gmail which is of type String 
                    we must clone them or use their borrowed form or return the static 
                    version of their slice like &'static str
                */
                    gmail.eq(self.user_gmail.clone()),
                    phone_number.eq(self.user_phone_number.clone()),
                    username.eq(self.username.clone()),
                    paypal_id.eq(self.paypal_id.clone()),
                    account_number.eq(self.account_number.clone()),
                    device_id.eq(self.device_id.clone()),
                    social_id.eq(self.social_id.clone()),
                    cid.eq(hex_cid),
                    snowflake_id.eq(self.new_snowflake_id),
                )
            )
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {
                    Ok(
                        UserIdResponse { 
                            id: updated_user.id, 
                            username: updated_user.username, 
                            activity_code: updated_user.activity_code, 
                            twitter_username: updated_user.twitter_username, 
                            facebook_username: updated_user.facebook_username, 
                            discord_username: updated_user.discord_username, 
                            identifier: updated_user.identifier, 
                            user_role: {
                                match updated_user.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: updated_user.token_time,
                            last_login: if updated_user.last_login.is_some(){
                                    Some(updated_user.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            ,
                            created_at: updated_user.created_at.to_string(),
                            updated_at: updated_user.updated_at.to_string(),
                            gmail: updated_user.gmail,
                            phone_number: updated_user.phone_number,
                            paypal_id: updated_user.paypal_id,
                            account_number: updated_user.account_number,
                            device_id: updated_user.device_id,
                            social_id: updated_user.social_id,
                            cid: updated_user.cid,
                            signer: self.signer.clone(),
                            snowflake_id: updated_user.snowflake_id,
                            stars: updated_user.stars
                        }
                    )
                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec(); /* extend the empty msg_content from the error utf8 slice */
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)));
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