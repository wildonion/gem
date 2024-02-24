


pub use super::*;


#[derive(Serialize, Deserialize, Clone)]
pub struct Health{
    pub status: String,
}



#[get("/check-server")]
#[passport(admin, user, dev)]
pub(self) async fn index(
        req: HttpRequest,  
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

        let iam_healthy = Health{
            status: "🥞 Alive".to_string()
        };
    
        resp!{
            Health, // the data type
            iam_healthy, // response data
            IAM_HEALTHY, // response message
            StatusCode::OK, // status code
            None::<Cookie<'_>>,
        }

}

pub mod exports{
    pub use super::index;
}