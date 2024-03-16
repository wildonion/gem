

pub use super::*;



#[post("/ticket/send")]
pub(self) async fn send(
    req: HttpRequest,
    ticket_data: web::Json<NewUserTicketRequest>,
    app_state: web::Data<AppState>
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            match UserTicket::insert(ticket_data.to_owned(), connection).await{
                Ok(ticket_data) => {
                    
                    resp!{
                        UserTicket, // the data type
                        ticket_data, // response data
                        CREATED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
                    }

                },
                Err(resp) => {
                    return Err(resp.into());
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
    pub use super::send;
}