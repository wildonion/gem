


use crate::*;
use crate::resp;
use crate::constants::*;


// god and dev panel using yew and tauri 


#[derive(Serialize, Deserialize, Clone)]
pub struct Dev{
    pub id: u8,
}


#[get("/panel/dev/api/{id}")]
async fn index(req: HttpRequest, id: web::Path<u8>) -> Result<HttpResponse, actix_web::Error> {
    
    let id = id.to_owned();
    let data = Dev{id};
    
    resp!{
        data.clone(), //// response data
        FETCHED, //// response message
        StatusCode::OK, //// status code
    }

}








pub fn dev_service_init(config: &mut web::ServiceConfig){
    config.service(index);
}