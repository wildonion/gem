


use actix_web::HttpResponse;
use chrono::{Utc, NaiveDateTime};
use lettre::{
    message::{header::ContentType as LettreContentType, Mailbox},
    transport::smtp::authentication::Credentials, 
    AsyncSmtpTransport, AsyncTransport, Message as LettreMessage,
    Tokio1Executor, 
};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Response<'m, T>{
    pub data: Option<T>,
    pub message: &'m str, // &str are a slice of String thus they're behind a pointer and every pointer needs a valid lifetime which is 'm in here 
    pub status: u16,
    pub is_error: bool
}

pub type PanelHttpResponse = Result<actix_web::HttpResponse, actix_web::Error>;



pub async fn send_mail(
    APP_NAME: &str,
    mail_owner_id: i32,
    user_mail: &str,
    random_code: &str,
) -> Result<NaiveDateTime, PanelHttpResponse>{

    let smtp_username = std::env::var("SMTP_USERNAME").unwrap();
    let smtp_password = std::env::var("SMTP_PASSWORD").unwrap();
    let smtp_server = std::env::var("SMTP_SERVER").unwrap();
    let smtp_creds = Credentials::new(smtp_username.clone(), smtp_password);
    
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_server.as_str())
        .unwrap()
        .credentials(smtp_creds)
        .build();

    let from = format!("{}: <{}>", APP_NAME, smtp_username).parse::<Mailbox>();
    let to = format!("{}: <{}>", mail_owner_id, user_mail).parse::<Mailbox>();

    if from.is_err() || to.is_err(){

        // let from_send_mail_error = from.unwrap_err();
        // let to_send_mail_error = to.unwrap_err(); /* or this cause they have same error message */
        let final_err = format!("Invalid Sender Or Receiver Mail Address");

        let resp = Response::<'_, &[u8]>{
            data: Some(&[]),
            message: &final_err,
            status: 417,
            is_error: true
        };
        return Err(
            Ok(HttpResponse::ExpectationFailed().json(resp))
        );
            
    }

    /* ------------------------- */
    /* matching over from and to */
    /* ------------------------- */
    // let (from, to) = match (from, to){
    //     (Ok(from), Ok(to)) => {
    //         (from, to)
    //     }, 
    //     (Err(from_err)) | (Err(to_err)) => {

    //         /* handle error */
    //         // ...

    //     }
    // };

    let now = Utc::now();
    let five_mins_later = (now + chrono::Duration::minutes(5)).naive_local();

    let subject = "Mail Verification";
    let body = format!("
        <p>Use this code to get verified in {}: <b>{}</b></p>
        <br>
        <p>This code will expire at: <b>{} UTC</b></p>", 
        APP_NAME, random_code, five_mins_later.and_utc().to_rfc2822().to_string());

    let email = LettreMessage::builder()
        .from(from.unwrap())
        .to(to.unwrap())
        .subject(subject)
        .header(LettreContentType::TEXT_HTML)
        .body(body)
        .unwrap();

    let get_mail_res = mailer.send(email).await;
    let Ok(_) = get_mail_res else {

        let send_mail_error = get_mail_res.unwrap_err();

        let resp = Response::<'_, &[u8]>{
            data: Some(&[]),
            message: &send_mail_error.to_string(),
            status: 417,
            is_error: true
        };
        return Err(
            Ok(HttpResponse::ExpectationFailed().json(resp))
        );
    };

    Ok(
        five_mins_later
    )

}