


use crate::*;
use crate::constants::OPENAI_IMAGES;
use async_openai::{
    types::{CreateImageRequestArgs, CreateImageRequest, ImageSize, ResponseFormat, ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, CreateChatCompletionRequest},
    Client as OpenAiClient,
};
use self::constants::CHATGPT_SUMMARIZATION_PROMPT;
use crate::constants::{APP_NAME, THIRDPARTYAPI_ERROR_CODE};


// https://github.com/64bit/async-openai/tree/main/examples

pub mod generate{

    pub use super::*;

    pub async fn create_image_from(title: &'static str, event_id: i32) -> String{ // make title static so it can be valid across tokio::spawn

        // the receiver must be mutable since we want to mutate the 
        // state of the receiver once we received the data from the channel
        let (generated_img_sender, mut generated_img_receiver) = 
            tokio::sync::oneshot::channel::<String>();

        tokio::spawn(async move{

            let client = OpenAiClient::new();
            let img_request = CreateImageRequestArgs::default()
                .prompt(title)
                .n(1)
                .response_format(ResponseFormat::Url)
                .size(ImageSize::S1024x1024)
                .user(APP_NAME)
                .build();

            if img_request.is_err(){

                let why = img_request.as_ref().unwrap_err();
                let err_resp_str = why.to_string();
                let err_resp_vec = err_resp_str.as_bytes().to_vec();

                /* custom error handler */
                use helpers::error::{ErrorKind, ThirdPartyApiError, PanelError};
                let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str)), "openai::create_image_from::create_image");
                let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
            }

            let images_request = img_request.unwrap_or(CreateImageRequest::default());
            let response = client.images().create(images_request).await.unwrap();
            let image_full_path = format!("{}/{}/", OPENAI_IMAGES, event_id);
            let path_buffs = response.save(image_full_path).await.unwrap();

            // getting a single element requires to be behind a pointer
            // since the operation takes the ownership of the vector
            let generated_nft_path_buf = &path_buffs[0]; // there is only one images is generated
            let nft_path = generated_nft_path_buf.to_str().unwrap();

            generated_img_sender.send(nft_path.to_string());

        });

        if let Ok(img_url) = generated_img_receiver.try_recv(){

            return img_url;
        }

        return String::from("");

    }
} 

pub mod summarize{


    pub use super::*;
    
    pub async fn create_titles_from(chats: &[String]) -> String {

        let (summarization_sender, mut summarization_receiver) = 
            tokio::sync::oneshot::channel::<Option<String>>();

        
        // gathering all the user's chats into a single string
        let mut user_chats = String::from("");
        for chat in chats{

            user_chats += format!("{:?}. ", chat.to_owned()).as_str();
            
        }

        let gpt_prompt = format!("{} '{:?}'", CHATGPT_SUMMARIZATION_PROMPT, user_chats);
        tokio::spawn(async move{
            let client = OpenAiClient::new();
            let chat_request = CreateChatCompletionRequestArgs::default()
                .max_tokens(1024u16)
                .model("gpt-4-turbo")
                .messages([
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(gpt_prompt.as_str())
                        .build()
                        .unwrap()
                        .into(),
                ])
                .build();

            if chat_request.is_err(){

                let why = chat_request.as_ref().unwrap_err();
                let err_resp_str = why.to_string();
                let err_resp_vec = err_resp_str.as_bytes().to_vec();

                /* custom error handler */
                use helpers::error::{ErrorKind, ThirdPartyApiError, PanelError};
                let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str)), "openai::summarize::create_titles_from");
                let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
            }
            
            let chat_request = chat_request.unwrap_or(CreateChatCompletionRequest::default());
            let response = client.chat().create(chat_request).await.unwrap();
            let choices = response.choices;
            let assitant_answer = &choices[0];
            let assitant_content = &assitant_answer.message.content;
            
            summarization_sender.send(assitant_content.to_owned())
        
        });


        if let Ok(summarization) = summarization_receiver.try_recv(){

            if summarization.is_some(){
                return summarization.unwrap();
            } else{
                return String::from("");        
            }
        }

        return String::from("");


    }

}