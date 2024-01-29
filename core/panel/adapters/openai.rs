


use crate::*;
use openai_api_rs::v1::image::ImageGenerationRequest;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4;



pub mod generate{


    // https://platform.openai.com/docs/api-reference/images/create
    // generate nft like image based on the user's summarization
    // ...
} 

pub mod summarize{
    
    // https://platform.openai.com/docs/api-reference/chat/create
    // summarize user's chats into a title like statement 
    // ...
}