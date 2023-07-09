

/*
    
    ┏———————————————————————————————┓
        REGISTERING SLASH COMMANDS
    ┗———————————————————————————————┛

*/


use crate::*;

// --------------------------------------
// ----------- SLASH COMMANDS -----------
// --------------------------------------
pub mod slash{
    
    use super::*; // loading all the crates that has loaded outside of this module
    
    // command param will be over written later thus it must be defined mutable
    pub fn catchup_register(command: &mut builder::CreateApplicationCommand) -> &mut builder::CreateApplicationCommand {
        command
            .name("catchup")
            .description("conse catch up")
            .create_option(|opt| {
                opt
                    .name("hours")
                    .description("hours ago from 1 to 24")
                    .kind(CommandOptionType::Integer)
                    .min_int_value(1)
                    .max_int_value(24)
                    .required(true)
            })
    }

    // command will be over written later thus it must be defined mutable
    pub fn help_register(command: &mut builder::CreateApplicationCommand) -> &mut builder::CreateApplicationCommand {
        command
            .name("help")
            .description("conse catch up help")

    }

    // command will be over written later thus it must be defined mutable
    pub fn stats_register(command: &mut builder::CreateApplicationCommand) -> &mut builder::CreateApplicationCommand {
        command
            .name("stats")
            .description("conse server stats")

    }



}


#[hook]
pub async fn delay_action(ctx: &Context, msg: &Message) {
    let _ = msg.react(ctx, '⏱').await;
}