


pub use super::*;



#[post("/ticket/get/all/")]
#[passport(admin)]
pub(self) async fn get_all(
    req: HttpRequest,
    limit: web::Path<Limit>,
    app_state: web::Data<AppState>
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            match UserTicket::get_all(limit, connection).await{
                Ok(tickets_data) => {
                    
                    resp!{
                        Vec<UserTicket>, // the data type
                        tickets_data, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
                    }

                },
                Err(resp) => resp,
                
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
    pub use super::get_all;
}