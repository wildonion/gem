


use crate::*;
use crate::misc::{Response, gen_chars, gen_random_idx, gen_random_number};
use crate::schema::users;
use crate::schema::users::dsl::*;
use crate::constants::*;



/* 

    diesel migration generate users ---> create users migration sql files
    diesel migration run            ---> apply sql files to db 
    diesel migration redo           ---> drop tables 

*/
#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct User{
    pub id: i32,
    pub username: String,
    pub activity_code: String,
    pub twitter_username: Option<String>,
    pub facebook_username: Option<String>,
    pub discord_username: Option<String>,
    pub wallet_address: Option<String>,
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
    pub wallet_address: Option<String>,
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
    pub wallet_address: Option<String>,
    pub user_role: String,
    pub token_time: Option<i64>,
    pub last_login: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct LoginInfoRequest{
    pub username: String,
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
    pub wallet_address: &'l str,
    pub user_role: UserRole,
    pub pswd: &'l str,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name=users)]
#[derive(Clone, Debug)]
pub struct EditUserByAdmin<'p>{
    pub user_role: UserRole,
    pub username: &'p str,
    pub wallet_address: &'p str,
    pub pswd: &'p str
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTClaims{
    pub _id: i32, //// mongodb object id
    pub username: Option<String>,
    pub wallet: Option<String>,
    pub user_role: UserRole,
    pub exp: i64, //// expiration timestamp
    pub iat: i64, //// issued timestamp
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct NewAdminInfoRequest{
    pub username: String,
    pub wallet: String,
    pub password: String
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct EditUserByAdminRequest{
    pub user_id: i32,
    pub role: String,
    pub username: String,
    pub wallet: String,
    pub password: Option<String>
}

impl User{

    pub fn decode_token(token: &str) -> Result<TokenData<JWTClaims>, jsonwebtoken::errors::Error>{
        let encoding_key = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
        let decoded_token = decode::<JWTClaims>(token, &DecodingKey::from_secret(encoding_key.as_bytes()), &Validation::new(Algorithm::HS512));
        decoded_token
    }

    pub fn passport(req: HttpRequest, pass_role: Option<UserRole>, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<JWTClaims, Result<HttpResponse, actix_web::Error>>{

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
 
        } else{ /* perhaps we have cookie since the jwt flag is not true */

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

    fn generate_token(&self) -> Result<String, jsonwebtoken::errors::Error>{
        
        let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nano to sec
        let exp_time = now + env::var("JWT_EXPIRATION").expect("⚠️ found no jwt expiration time").parse::<i64>().unwrap();
        
        let payload = JWTClaims{
            _id: self.id,
            username: Some(self.username.clone()), /* here username and user_role are behind a reference which can't be moved thus we must clone them */
            wallet: self.wallet_address.clone(),
            user_role: self.user_role.clone(),
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
    pub fn generate_cookie(self) -> Option<(Cookie<'static>, i64)>{

        /* if we're here means that the password was correct */
        let token = self.generate_token().unwrap();

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
        
        let cookie_value = format!("{token:}::{time_hash_hex_string:}");
        let mut cookie = Cookie::build("jwt", cookie_value)
                                    .same_site(cookie::SameSite::Strict)
                                    .finish();
        let cookie_exp_days = env::var("COOKIE_EXPIRATION_DAYS").expect("⚠️ no cookie exporation days variable set").parse::<i64>().unwrap();
        let mut now = OffsetDateTime::now_utc();
        now += Duration::days(cookie_exp_days);
        cookie.set_expires(now);

        Some((cookie, time_hash_now))

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

    pub async fn find_by_username(user_name: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Self, Result<HttpResponse, actix_web::Error>>{

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

    pub async fn find_by_wallet(wallet: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Self, Result<HttpResponse, actix_web::Error>>{

        let single_user = users
            .filter(wallet_address.eq(wallet.to_string()))
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(wallet),
                message: USER_NOT_FOUND,
                status: 404
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub async fn find_by_id(user_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Self, Result<HttpResponse, actix_web::Error>>{

        let single_user = users
            .filter(users::id.eq(user_id))
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(user_id),
                message: USER_NOT_FOUND,
                status: 404
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub async fn insert(wallet: String, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<(UserData, Cookie), Result<HttpResponse, actix_web::Error>>{

        let random_chars = gen_chars(gen_random_number(5, 11));
        let random_code: String = (0..5).map(|_|{
            let idx = gen_random_idx(random::<u8>() as usize); //// idx is one byte cause it's of type u8
            CHARSET[idx] as char //// CHARSET is of type utf8 bytes thus we can index it which it's length is 10 bytes (0-9)
        }).collect();

        let new_user = NewUser{
            username: "",
            activity_code: &random_code,
            wallet_address: &wallet,
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
                        wallet_address: fetched_user.wallet_address.clone(),
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
                    };

                    /* generate cookie 🍪 from token time and jwt */
                    /* since generate_cookie() takes the ownership of the user instance we must clone it then call this */
                    /* generate_cookie() returns a Cookie instance with a 'static lifetime which allows us to return it from here*/
                    let Some(cookie_info) = fetched_user.clone().generate_cookie() else{
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
    

    pub async fn insert_new_admin(user: NewAdminInfoRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<usize, Result<HttpResponse, actix_web::Error>>{

        let hash_pswd = User::hash_pswd(user.password.as_str()).unwrap();
        let user = NewUser{
            username: user.username.as_str(),
            activity_code: "",
            wallet_address: user.wallet.as_str(),
            user_role: UserRole::Admin,
            pswd: hash_pswd.as_str()
        };

        match diesel::insert_into(users::table)
            .values(&user)
            .execute(connection)
            {
                Ok(affected_row) => Ok(affected_row),
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

    pub async fn edit_by_admin(new_user: EditUserByAdminRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserData, Result<HttpResponse, actix_web::Error>>{

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
        let password = if let Some(password) = &new_user.password{ //// borrowing the user to prevent from moving

            /* we can pass &str to the method by borrowing the String since String will be coerced into &str at compile time */
            User::hash_pswd(password).unwrap()

        } else{
            
            /* if the passed in password was none then we must use the old one */
            user.pswd

        };
        
        match diesel::update(users.find(new_user.user_id.to_owned()))
            .set(EditUserByAdmin{
                user_role: {
                    let role = new_user.role.as_str(); 
                    match role{
                        "user" => UserRole::User,
                        "admin" => UserRole::Admin,
                        _ => UserRole::Dev
                    }
                },
                /* 
                    pswd, username and wallet is of type &str thus by borrowing these 
                    feilds from new_user instance we can convert them into &str 
                */
                pswd: &password,
                username: &new_user.username,
                wallet_address: &new_user.wallet
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
                            wallet_address: updated_user.wallet_address, 
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
                        }
                    )
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

    pub async fn delete_by_admin(user_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<usize, Result<HttpResponse, actix_web::Error>>{

        match diesel::delete(users.filter(users::id.eq(user_id.to_owned())))
            .execute(connection)
            {
                Ok(num_deleted) => Ok(num_deleted),
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

    pub async fn get_all(connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<UserData>, Result<HttpResponse, actix_web::Error>>{

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
                            wallet_address: u.wallet_address, 
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
                        })
                        .collect::<Vec<UserData>>()
                )
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

    pub async fn logout(who: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<(), Result<HttpResponse, actix_web::Error>>{

        match diesel::update(users.find(who))
            .set(token_time.eq(0))
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => Ok(()),
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

    pub async fn update_social_account(
        wallet: &str, 
        account_name: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserData, Result<HttpResponse, actix_web::Error>>{


            let Ok(user) = User::find_by_wallet(wallet, connection).await else{
                let resp = Response{
                    data: Some(wallet),
                    message: USER_NOT_FOUND,
                    status: 404
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                );
            };

            match diesel::update(users.find(user.id))
                .set(twitter_username.eq(account_name))
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
                                wallet_address: updated_user.wallet_address, 
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
                            }
                        )
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