




use crate::*;


//// ----------------------------------------------
//// -------------- GPT STRUCTURE -----------------
//// ----------------------------------------------

#[derive(Clone, Debug)]
pub struct Gpt{
    pub messages: Vec<ChatCompletionMessage>,
    pub last_content: String, //// utf8 bytes is easier to handle tokenization process later
    pub current_response: String,
}

impl Gpt{
    
    pub async fn new() -> Gpt{
        let content = "Hello,"; //// starting conversation to feed later tokens to the GPT model for prediction
        Self{
            messages: vec![
                ChatCompletionMessage{
                    role: ChatCompletionMessageRole::System,
                    content: content.to_string(),
                    name: None,
                }
            ],
            last_content: content.to_string(),
            current_response: "".to_string()
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
    //// if we want to mutate the pointer it must be defined as mutable also its underlying data must be mutable 
    //// we borrow since copy trait doesn't implement for the type also we can clone it too to pass to other scopes 
    //// dereferencing and clone method will return the Self and can't deref if the underlying data doesn't implement Copy trait 
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
        
        let mut messages = self.messages.clone(); //// clone messages vector since we can't move a type if it's behind a pointer 
        messages.push(ChatCompletionMessage{ //// pushing the current token to the vector so the GPT can be able to predict the next tokens based on the previous ones 
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
        messages.push(ChatCompletionMessage{ //// we must also push the response of the chat GPT to the messages in order he's able to predict the next tokens based on what he just saied :)  
            role: ChatCompletionMessageRole::Assistant,
            content: self.current_response.clone(),
            name: None
        });
        self.messages = messages.clone(); //// finally update the message field inside the Gpt instance 
        Self{
            messages,
            last_content: content.to_string(),
            current_response : self.current_response.clone(), //// cloning sicne rust doesn't allow to move the current_response into new scopes (where it has been called) since self is behind a pointer
        }
    }

}    