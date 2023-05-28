


use crate::*;
use crate::misc::Response;
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

#[derive(Serialize, Deserialize, Clone, Debug, Queryable, Selectable)]
#[diesel(table_name=users)]
pub struct FetchUser{
    pub id: i32,
    pub username: String,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserLoginData{
    pub id: i32,
    pub username: String,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    pub user_role: UserRole,
    pub pswd: &'l str,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name=users)]
#[derive(Clone, Debug)]
pub struct EditUserByAdmin<'p>{
    pub user_role: UserRole,
    pub pswd: &'p str
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTClaims{
    pub _id: i32, //// mongodb object id
    pub username: String,
    pub user_role: UserRole,
    pub exp: i64, //// expiration timestamp
    pub iat: i64, //// issued timestamp
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewAdminInfoRequest{
    pub username: String,
    pub password: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditUserByAdminRequest{
    pub user_id: i32,
    pub role: String,
    pub password: Option<String>
}

impl User{

    pub fn decode_token(token: &str) -> Result<TokenData<JWTClaims>, jsonwebtoken::errors::Error>{
        let encoding_key = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
        let decoded_token = decode::<JWTClaims>(token, &DecodingKey::from_secret(encoding_key.as_bytes()), &Validation::new(Algorithm::HS512));
        decoded_token
    }

    pub fn passport(req: HttpRequest, pass_role: UserRole, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<JWTClaims, Result<HttpResponse, actix_web::Error>>{
        
        /* checking that the request cookie has the jwt key */
        let Some(cookie) = req.cookie("jwt") else{
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: NOT_FOUND_JWT_VALUE,
                status: 404
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

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
                            if user.user_role != pass_role{
                                let resp = Response{
                                    data: Some(_id.to_owned()),
                                    message: ACCESS_DENIED,
                                    status: 403
                                };
                                return Err(
                                    Ok(HttpResponse::Forbidden().json(resp))
                                );
                            } 

                            /* returning token data, if we're here means that nothing went wrong */
                            Ok(token_data)
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

    fn generate_token(&self) -> Result<String, jsonwebtoken::errors::Error>{
        
        let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nano to sec
        let exp_time = now + env::var("JWT_EXPIRATION").expect("⚠️ found no jwt expiration time").parse::<i64>().unwrap();
        
        let payload = JWTClaims{
            _id: self.id,
            username: self.username.clone(), /* here username and user_role are behind a reference which can't be moved thus we must clone them */
            user_role: self.user_role.clone(),
            exp: exp_time,
            iat: now
        };
    
        let encoding_key = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
        let token = encode(&Header::new(Algorithm::HS512), &payload, &EncodingKey::from_secret(encoding_key.as_bytes()));
        token
    
    }

    pub fn generate_cookie(&self) -> Option<(Cookie, i64)>{

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

        let time_hash_hex_string = time_hash
                                        .into_iter()
                                        .map(|byte| format!("{:02x}", byte))
                                        .collect::<String>();
        
        let cookie_value = format!("{}::{}", token, time_hash_hex_string);
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

    pub async fn insert_new_admin(user: NewAdminInfoRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<usize, Result<HttpResponse, actix_web::Error>>{

        let hash_pswd = User::hash_pswd(user.password.as_str()).unwrap();
        let user = NewUser{
            username: user.username.as_str(),
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

    pub async fn edit_by_admin(new_user: EditUserByAdminRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<FetchUser, Result<HttpResponse, actix_web::Error>>{

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
                /* pswd is of type &str thus by borrowing password we can convert it into &str */
                pswd: &password
            })
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => Ok(updated_user),
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

    pub async fn get_all(connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<User>, Result<HttpResponse, actix_web::Error>>{

        match users.load::<User>(connection)
        {
            Ok(all_users) => Ok(all_users),
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

    pub async fn edit(new_user: EditUserByAdminRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<FetchUser, Result<HttpResponse, actix_web::Error>>{

        todo!()
        
    }


}