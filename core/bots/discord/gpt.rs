



/*

    at the time of this writing:
    currently only chat gpt 3.5 or gpt-3.5-turbo is available for api calling with a rate limit of 3 messages per minute
    gpt 4  is available newly which is in its beta stage but openai didn't add it for api calling also its rate limit is 
    25 messages every 3 hours that's why after sending 10 messages per second the bot gets halted and faced discord rate 
    limit since we'll reach the gpt rate limit and the rest logic will be dropped thus the bot gets halted which makes 
    discord angry. 
    ```
        [2023-04-29 12:13:25.058610239 +02:00] - Rate limit reached for default-gpt-3.5-turbo in 
        organization org-fxQTuZabNydHKpMpVHcA3it2 on requests per min. Limit: 3 / min. 
        Please try again in 20s. Contact support@openai.com if you continue to have issues. 
        Please add a payment method to your account to increase your rate limit. 
        Visit https://platform.openai.com/account/billing to add a payment method.
    ```

    we can update the RPM inside the admin panel to avoid openai rate limit

*/



pub mod chat{


    use tokio::io::AsyncWriteExt; // this is required to call the write_all() method of the Write trait on the created file 

    use crate::*;


    pub const GPT_PROMPT: &str = "Summarise the discussion from the discord server but remove any greetings. Include only the items that created the most engagement. Use a sarcastically whimsical tone";


    // ----------------------------------------------
    // -------------- GPT STRUCTURE -----------------
    // ----------------------------------------------
    // at the time of this writing
    // rate limits for the ChatGPT API?
    // Free trial users: 20 RPM 40000 TPM
    // Pay-as-you-go users (first 48 hours): 60 RPM 60,000 TPM
    // Pay-as-you-go users (after 48 hours): 3500 RPM 90,000 TPM
    // RPM = requests per minute
    // TPM = tokens per minute



    #[derive(Clone, Debug)]
    pub struct Gpt{
        pub messages: Vec<ChatCompletionMessage>,
        pub last_content: String, // utf8 bytes is easier to handle tokenization process later
        pub current_response: String,
        pub is_rate_limit: bool,
    }

    impl Gpt{
        
        pub async fn new(messages: Option<Vec<ChatCompletionMessage>>) -> Gpt{
            if let Some(gpt_messages) = messages{
                Self{
                    messages: gpt_messages.clone(),
                    last_content: gpt_messages[gpt_messages.len() - 1].content.clone().unwrap().to_string(),
                    current_response: "".to_string(),
                    is_rate_limit: false,
                }
            } else{
                let content = "Hello,"; // starting conversation to feed later tokens to the GPT model for prediction
                Self{
                    messages: vec![
                        ChatCompletionMessage{
                            role: ChatCompletionMessageRole::System,
                            content: Some(content.to_string()),
                            name: None,
                            function_call: None
                        }
                    ],
                    last_content: content.to_string(),
                    current_response: "".to_string(),
                    is_rate_limit: false,
                }
            }
        }
        
        //→ if the content was String we couldn't return its &str since this is 
        //  owned by the function and its lifetime will be dropped once the function 
        //  gets executed thus we can't return a &str or a pointer to its utf8 bytes 
        //  because its pointer might be a dangling one in the caller space since 
        //  we don't have that String anymore inside the function! this is different
        //  about the &str in the first place cause we're cool with returning them
        //  since they are behind a pointer and kinda stack data types which by
        //  passing them to other scopes their lifetime won't be dropped since they
        //  will be copied bit by bit instead moving the entire underlying data.
        //→ also if the self wasn't behind a reference by calling the first method on 
        //  the Gpt instance the instance will be moved and we can't call other methods.
        //
        // we can't return a pointer to the String from the function since Strings or Vecs
        // are heap data types and once the function gets executed their lifetime will be dropped
        // from the ram to free the allocations and because of this returning a pointer to them 
        // might be a dangling pointer which rust doesn't allow us to do this in the first place.
        //
        // since heap data doesn't implement Copy thus by moving them we'll lose the 
        // ownership of them also can't move if the heap type is behind a shared reference
        // since by moving it'll be dropped from the ram and its pointer will be a dangled.
        //
        // → if we want to mutate the pointer it must be defined as mutable also its underlying data must be mutable
        // → we borrow since copy trait doesn't implement for the type also we can clone it too to pass to other scopes 
        // → dereferencing and clone method will return the Self and can't deref if the underlying data doesn't implement Copy trait
        //
        // it's ok to return slices types like str and [] behind a pointer by using the lifetime of the self or a passed 
        // in one since they are on the stack but we can't return dynamic types behind a pointer since copy trait is not 
        // implemented for them and once the scope gets executed their lifetime will be dropped from the ram since they're 
        // expensive and rust doesn't store them forever thus it moves them to the newly scope but since there is a pointer of them 
        // exists in function we can't return the pointer and in other scopes we can't move them to the new scope
        // we could clone them or borrow them using using Rc, & or as_ref() 
        pub async fn feed(&mut self, content: &str) -> Gpt{
            
            //→ based on borrowing and ownership rules in rust we can't move a type into new scope when there
            //  is a borrow or a pointer of that type exists, rust moves heap data types by default since it 
            //  has no gc rules means if the type doesn't implement Copy trait by moving it its lifetime will 
            //  be dropped from the memory and if the type is behind a pointer rust doesn't allow the type to 
            //  be moved, the reason is, by moving the type into new scopes its lifetime will be dropped 
            //  accordingly its pointer will be a dangling one in the past scope, to solve this we must either 
            //  pass its clone or its borrow to other scopes. in this case self is behind a mutable reference 
            //  thus by moving every field of self which doesn't implement Copy trait we'll lose the ownership 
            //  of that field and since it's behin a pointer rust won't let us do that in the first place which 
            //  forces us to pass either its borrow or clone to other scopes.   
            
            let mut messages = self.messages.clone(); // clone messages vector since we can't move a type if it's behind a pointer 
            messages.push(ChatCompletionMessage{ // pushing the current token to the vector so the GPT can be able to predict the next tokens based on the previous ones 
                role: ChatCompletionMessageRole::User,
                content: Some(content.to_string()),
                name: None,
                function_call: None
            });
            let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
                                                                        .create()
                                                                        .await;
            // rate limi reached probably :(
            // save openai error into a log file 
            if let Err(e) = chat_completion.clone(){
                let log_content = format!("[{}] - {}", chrono::Local::now(), e);
                let log_name = format!("[{}]", chrono::Local::now());
                let filepath = format!("logs/openai-logs/{}.log", log_name);
                let mut openai_log = tokio::fs::File::create(filepath.as_str()).await.unwrap();
                openai_log.write_all(log_content.as_bytes()).await.unwrap();
                return Self{
                    messages: self.messages.clone(),
                    last_content: self.last_content.clone(),
                    current_response : self.current_response.clone(), // cloning sicne rust doesn't allow to move the current_response into new scopes (where it has been called) since self is behind a pointer
                    is_rate_limit: true,
                };
            }
            let returned_message = chat_completion.unwrap().choices.first().unwrap().message.clone();
            self.current_response = returned_message.content.unwrap().to_string();
            messages.push(ChatCompletionMessage{ // we must also push the response of the chat GPT to the messages in order he's able to predict the next tokens based on what he just saied :)  
                role: ChatCompletionMessageRole::Assistant,
                content: Some(self.current_response.clone()),
                name: None,
                function_call: None
            });
            // since self.messages is behind a mutable reference we can't 
            // move it around since rust doesn't allow use to move the 
            // heap type if it's behind a muable pointer the soultion 
            // can be either cloning which will return the type itself 
            // or Self, borrowing (use their slice form) or dereferencing.
            self.messages = messages.clone(); // finally update the message field inside the Gpt instance 
            Self{
                messages,
                last_content: content.to_string(),
                current_response : self.current_response.clone(), // cloning sicne rust doesn't allow to move the current_response into new scopes (where it has been called) since self is behind a pointer
                is_rate_limit: self.is_rate_limit,
            }
        }

    }    
}