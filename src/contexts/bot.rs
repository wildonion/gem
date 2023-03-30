




// TODO - discrod channel summerization using chatGPT
// TODO - discord bot for conse conse PaaS like checking services status by typing commands on the discord
// ...
// https://developers.facebook.com/blog/post/2020/09/30/build-discord-bot-with-rust-and-serenity/
// https://betterprogramming.pub/writing-a-discord-bot-in-rust-2d0e50869f64


/*

    BOT:
      -> get messages of a channel using discord api OR get messages of a user inside a channel 
      -> for every fetched message feed that to the ChatGPT api to get some bullet list based summerization 
      -> store the summerizations inside the mongodb alongside with their dates
      -> write /summerize discord command inside the bot to fetch the summerization based on a specific date or for today

*/
 

pub mod wwu_bot{


    pub async fn get_channel_messages(){

    }

    pub async fn summerize_channel_messages(){

        // send fetched messages to chatgpt api
        // ...

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