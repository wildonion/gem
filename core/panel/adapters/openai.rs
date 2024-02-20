


use crate::*;

// https://github.com/64bit/async-openai/tree/main/examples

pub mod generate{

    pub async fn create_image_from(title: &str) -> String{

        // generate an image based on the passed in title 
        // ...

        tokio::spawn(async move{

        });
        
        let img_url = String::from("");
        img_url
    }
} 

pub mod summarize{
    
    pub async fn create_titles_from(chats: &[String]) -> String {

        // summarize all the texts inside the chats into a single title
        // this will be done per each user's chats
        // ...

        tokio::spawn(async move{
            
        });

        todo!()
    }

}