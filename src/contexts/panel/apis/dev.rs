


use crate::*;

// god and dev panel using yew and tauri 


#[derive(Serialize, Deserialize)]
pub struct Dev{
    pub id: u8,
}


#[get("/panel/dev/api/{id}")]
async fn index(req: HttpRequest, id: web::Path<u8>) -> Result<HttpResponse, actix_web::Error> {
    let id = id.to_owned();
    Ok(
        HttpResponse::Ok().json(
            Dev{
                id,
            }
        )
    )
}






pub fn dev_service_init(config: &mut web::ServiceConfig){
    config.service(index);
}