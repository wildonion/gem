


use crate::*;
use openai_api_rs::v1::image::ImageGenerationRequest;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4;



pub mod generate{


    // https://platform.openai.com/docs/api-reference/images/create
    pub async fn create_image_from(title: &str) -> String{

        // generate an image based on the passed in title 
        // ...
        
        let img_url = String::from("");
        img_url
    }
} 

pub mod summarize{
    
    // https://platform.openai.com/docs/api-reference/chat/create
    pub async fn create_titles_from(chats: &[String]) -> String {

        // summarize all the texts inside the chats into a single title
        // ...

        todo!()
    }

}