




// https://discord.com/api/oauth2/authorize?client_id=1092048595605270589&permissions=274877974528&scope=bot
// TODO - discord bot for conse PaaS like checking services status by typing commands on the discord
 

pub mod wwu_bot{


    use openai::{ //// openai crate is using the reqwest lib under the hood
        chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole}
    }; 



    // https://betterprogramming.pub/writing-a-discord-bot-in-rust-2d0e50869f64 
    // https://developers.facebook.com/blog/post/2020/09/30/build-discord-bot-with-rust-and-serenity/
    // https://github.com/serenity-rs/serenity/blob/current/examples/e14_slash_commands/src/main.rs

    pub struct DiscordBot{
        
    }


    pub struct Gpt<'c>{
        pub messages: Vec<ChatCompletionMessage>,
        pub last_content: &'c [u8], //// utf8 bytes is easier to handle tokenizations
        pub current_response: String,
    }

    impl<'c> Gpt<'c>{
        
        pub async fn new() -> Gpt<'c>{
            //// this is not owned by the current function 
            //// so we can return it since:
            ////    - it's behind a pointer with a valid lifetime ('c)
            ////    - it's str and is inside either the stack or binary   
            let content = "Hello You're Amazing"; //// starting conversation to feed later tokens to the GPT model for prediction
            Self{
                messages: vec![
                    ChatCompletionMessage{
                        role: ChatCompletionMessageRole::System,
                        content: content.to_string(),
                        name: None,
                    }
                ],
                last_content: content.as_bytes(), //// since content is a string slice which is behind a pointer there is no need to clone it
                current_response: "".to_string()
            }
        }
        
        //→ if the content was String we couldn't return its &str since this is 
        //   owned by the function and its lifetime will be dropped once the function 
        //   gets executed thus we can't return a &str or a pointer to its utf8 bytes 
        //   because its pointer might be a dangling one in the caller space since 
        //   we don't have that String anymore inside the function! this is different
        //   about the &str in the first place cause we're cool with returning them
        //   since they are behind a pointer and kinda stack data types.
        //→ also if the self wasn't behind a reference by calling the first method on 
        //  the Gpt instance the instance will be moved and we can't call other methods.
        pub async fn feed(&mut self, content: &'c str) -> Gpt<'c>{
            
            //→ based on borrowing and ownership rules in rust we can't move a type into new scope when there
            //     is a borrow or a pointer of that type exists, rust moves heap data types by default since it 
            //     has no gc rules means if the type doesn't implement Copy trait by moving it its lifetime will 
            //     be dropped from the memory and if the type is behind a pointer rust doesn't allow the type to 
            //     be moved, the reason is, by moving the type into new scopes its lifetime will be dropped 
            //     accordingly its pointer will be a dangling one in the past scope, to solve this we must either 
            //     pass its clone or its borrow to other scopes. in this case self is behind a mutable reference 
            //     thus by moving every field of self which doesn't implement Copy trait we'll lose the ownership 
            //     of that field and since it's behin a pointer rust won't let us do that in the first place which 
            //     forces us to pass either its borrow or clone to other scopes.   
            
            let mut messages = self.messages.clone(); //// clone messages vector since we can't move a type if it's behind a pointer 
            messages.push(ChatCompletionMessage{
                role: ChatCompletionMessageRole::User,
                content: content.to_string(),
                name: None,
            });
            let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
                                                                        .create()
                                                                        .await
                                                                        .unwrap()
                                                                        .unwrap();
            let returned_message = chat_completion.choices.first().unwrap().message.clone();
            self.current_response = returned_message.content.to_string();
            Self{
                messages: self.messages.clone(),
                last_content: content.as_bytes(),
                current_response : self.current_response.clone(), //// cloning sicne rust doesn't allow to move the current_response into new scopes (where it has been called) since self is behind a pointer
            }
        }

    }

}