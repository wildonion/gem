



use crate::*;
use crate::models::users::*;
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::schema::users::dsl::*;




/*
     ------------------------
    |          APIS
    | ------------------------
    |
    |

*/

#[post("/login")]
pub(super) async fn login(
        req: HttpRequest, 
        user_id: web::Path<i32>,
        password: web::Path<String>,
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {
   
    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            let single_user = users
                                .filter(id.eq(user_id.to_owned()))
                                .first::<User>(connection);

            let Ok(user) = single_user else{
                resp!{
                    i32, //// the data type
                    user_id.to_owned(), //// response data
                    USER_NOT_FOUND, //// response message
                    StatusCode::NOT_FOUND, //// status code
                } 
            };

            match user.user_role{
                UserRole::Admin => {

                    let hash_pswd = user.hash_pswd(password.as_str()).unwrap();
                    let Ok(_) = user.verify_pswd(hash_pswd.as_str()) else{
                        resp!{
                            i32, //// the data type
                            user_id.to_owned(), //// response data
                            WRONG_PASSWORD, //// response message
                            StatusCode::FORBIDDEN, //// status code
                        }
                    };

                    /* if we're here means that the password was correct */
                    let token = user.generate_token();

                    /*
                        since cookie can be stored inside the request object thus for peers on the same network 
                        which have an equal ip address they share a same cookie thus we'll face the bug of which 
                        every user can be every user in which they can see other peer's jwt info inside their browser!
                    */
                    let now = chrono::Local::now().timestamp_nanos();
                    let now_str = format!("{}", now).as_str();
                    let time_hash = Sha3_512::new()
                                .chain_update(now_str.as_bytes())
                                .finalize();
                    let cookie_value = format!("{}+{}", time_hash, token);
                    /* stroing the generated jwt inside the cookie and send the cookie */
                    let cookie = Cookie::build("jwt", cookie_value).same_site(cookie::SameSite::Strict).finish();
            
                    resp!{
                        Cookie, //// the data type
                        cookie, //// response data
                        FETCHED, //// response message
                        StatusCode::OK, //// status code
                    } 

                },
                _ => {

                    resp!{
                        i32, //// the data type
                        user_id.to_owned(), //// response data
                        ACCESS_DENIED, //// response message
                        StatusCode::FORBIDDEN, //// status code
                    } 
                }
            }

        },
        None => {
            resp!{
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
            }
        }
    }

}

#[post("/register-new-admin")]
async fn register_new_admin(
        req: HttpRequest, 
        wallet: web::Path<String>, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

        let jwt_cookie = req.cookie("jwt").unwrap();
        let token = jwt_cookie.value();

        // decode token to check the user info like access level
        // ...

        resp!{
            &[u8], //// the data type
            &[], //// response data
            FETCHED, //// response message
            StatusCode::OK, //// status code
        } 


}

#[post("/register-new-task")]
async fn register_new_task(
        req: HttpRequest, 
        wallet: web::Path<String>, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {


        // need token 
        // ...


        resp!{
            &[u8], //// the data type
            &[], //// response data
            FETCHED, //// response message
            StatusCode::OK, //// status code
        } 


}





pub mod exports{
    pub use super::login;
    pub use super::register_new_admin;
    pub use super::register_new_task;    
}