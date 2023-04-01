




// TODO - discrod channel summerization using chatGPT
// TODO - discord bot for conse PaaS like checking services status by typing commands on the discord
// ...
// https://developers.facebook.com/blog/post/2020/09/30/build-discord-bot-with-rust-and-serenity/
// https://betterprogramming.pub/writing-a-discord-bot-in-rust-2d0e50869f64
// https://github.com/valentinegb/openai/blob/main/examples/chat_cli/src/main.rs

/*

    BOT:
      -> get messages of a channel using discord api OR get messages of a user inside a channel 
      -> for every fetched message feed that to the ChatGPT api to get some bullet list based summerization 
      -> store the summerizations inside the mongodb alongside with their dates
      -> write /summerize discord command inside the bot to fetch the summerization based on a specific date or for today


*/
 

pub mod wwu_bot{

    use openai::{ //// openai crate is using the reqwest lib under the hood
        chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole}
    }; 

    pub async fn get_message_of(){

    }

    pub async fn feed_gpt(content: String, mut messages: Vec<ChatCompletionMessage>) -> (Option<String>, Vec<ChatCompletionMessage>){

        messages.push(ChatCompletionMessage{
            role: ChatCompletionMessageRole::User,
            content,
            name: None,
        });
        let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
                                                                    .create()
                                                                    .await
                                                                    .unwrap()
                                                                    .unwrap();
        let returned_message = chat_completion.choices.first().unwrap().message.clone();
        (
            Some(returned_message.content.to_string()),
            messages.clone()
        )

    }
    

}



pub mod messanger{
    
    
    use uuid::Uuid;


    
    pub struct Server<'a>{ //// 'a is the lifetime of &[u8] which is the borrowed type of [u8] due to its unknown size at compile time  
        pub cluster_id: Uuid, //// the id of the cluster which this server is inside
        pub api_token: &'a [u8], //// is an array of a borrowed type of utf8 bytes with a valid lifetime 
        pub name: String,
        pub channels: Vec<Channel>,
        pub members: Vec<ServerMember>,
    }
    
    pub struct Thread{
        pub id: Uuid,
        pub name: String,
    }
    
    pub struct Channel{
        pub name: String,
        pub members: Vec<ChannelMember>,
        pub threads: Vec<Thread>,
        pub permissions: Vec<Permission>,
        pub cmds: Commands, //// pre builtin commands for this channel 
    }
    
    pub struct Permission;
    pub struct ServerMember;
    pub struct ChannelMember;
    pub struct Level;
    pub struct Role;
    pub enum Commands{
        Ban,
        Kick,
        Mute,
    }
        


}


pub trait void{
    type Input;
    type Output;

}