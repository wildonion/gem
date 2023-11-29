


use std::collections::HashMap;
use actix_web::HttpResponse;
use chrono::{Utc, NaiveDateTime};
use serde::{Serialize, Deserialize};

pub type PanelHttpResponse = Result<actix_web::HttpResponse, actix_web::Error>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Response<'m, T>{
    pub data: Option<T>,
    pub message: &'m str, // &str are a slice of String thus they're behind a pointer and every pointer needs a valid lifetime which is 'm in here 
    pub status: u16,
    pub is_error: bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SMSResponse{
    pub r#return: SMSResponseReturn, // use r# to escape reserved keywords to use them as identifiers in rust
    pub entries: Vec<SMSResponseEntries>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SMSResponseReturn{
    pub status: u16,
    pub message: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SMSResponseEntries{
    pub messageid: f64,
    pub message: String,
    pub status: u8,
    pub statustext: String,
    pub sender: String,
    pub receptor: String,
    pub date: i64,
    pub cost: u16, 
}

pub async fn send_code(
    APP_NAME: &str,
    random_code: &str,
    OTP_PROVIDER_DIDNT_SEND_CODE: &str,
    phone_owner_id: i32,
    u_country: &str,
    user_phone: &str
) -> Result<NaiveDateTime, PanelHttpResponse>{

    let otp_token = std::env::var("OTP_API_TOKEN").unwrap();
    let otp_template = std::env::var("OTP_API_TEMPLATE").unwrap();
    let thesmsworks_jwt = format!("JWT {}", std::env::var("THESMSWORKS_JWT").unwrap());

    /* 
        the phone verification process is before building crypto id 
        thus we don't have region in here just make an api call to 
        get the region.
    */
    // if single_user.region.is_none(){
    //     let resp = Response{
    //         data: Some(phone_owner_id),
    //         message: REGION_IS_NONE,
    //         status: 406,
    //     };
    //     return Err(
    //         Ok(HttpResponse::NotAcceptable().json(resp))
    //     );
    // }

    // let u_region = single_user.region.unwrap();

    let now = Utc::now();
    let two_mins_later = (now + chrono::Duration::minutes(2)).naive_local();
    let _ = match u_country{
        "ir" => {
            
            let otp_endpoint = format!("http://api.kavenegar.com/v1/{}/verify/lookup.json?receptor={}&token={}&template={}", otp_token, user_phone, random_code, otp_template);
            let otp_response = reqwest::Client::new()
                .get(otp_endpoint.as_str())
                .send()
                .await;

            let res_stat = otp_response
                .as_ref()
                .unwrap()
                .status()
                .as_u16();

            let otp_response_data = otp_response
                .unwrap()
                /* mapping the streaming of future io bytes into the SMSResponse struct */
                .json::<SMSResponse>()
                .await;

            if res_stat != 200{

                let resp = Response{
                    data: Some(phone_owner_id),
                    message: OTP_PROVIDER_DIDNT_SEND_CODE,
                    status: 417,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::ExpectationFailed().json(resp))
                );

            }  

        },
        _ => {

            let body_content = format!("Use this code to get verified in {}: {}", APP_NAME, random_code);
            let mut data = HashMap::new();
            data.insert("sender", APP_NAME.to_string());
            data.insert("destination", user_phone.to_string());
            data.insert("content", body_content);

            let otp_endpoint = format!("https://api.thesmsworks.co.uk/v1/message/send");
            
            let otp_response = reqwest::Client::new()
                .post(otp_endpoint.as_str())
                .header("Authorization", thesmsworks_jwt.as_str())
                .json(&data)
                .send()
                .await;

            let res_stat = otp_response
                .as_ref()
                .unwrap()
                .status()
                .as_u16();

                /* accessing json data dynamically without mapping the response bytes into a struct */
            let otp_response_data = otp_response.unwrap().json::<serde_json::Value>().await.unwrap();
            // let otp_response_data = otp_response.unwrap().text().await.unwrap();

            if res_stat != 201{

                let resp = Response{
                    data: Some(phone_owner_id),
                    message: OTP_PROVIDER_DIDNT_SEND_CODE,
                    status: 417,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::ExpectationFailed().json(resp))
                );

            }    

        }
    };

    Ok(
        two_mins_later
    )

}