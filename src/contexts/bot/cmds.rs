

/*
    
    ┏———————————————————————————————┓
        REGISTERING SLASH COMMANDS
    ┗———————————————————————————————┛

*/


use crate::*;

//// --------------------------------------
//// ----------- SLASH COMMANDS -----------
//// --------------------------------------
pub mod slash{
    
    use super::*; //// loading all the crates that has loaded outside of this module
    
    //// command param will be over written later thus it must be defined mutable
    pub fn wrapup_register(command: &mut builder::CreateApplicationCommand) -> &mut builder::CreateApplicationCommand {
        command
            .name("wrapup")
            .description("conse wrap up summarizer")
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

    //// command will be over written later thus it must be defined mutable
    pub fn expand_register(command: &mut builder::CreateApplicationCommand) -> &mut builder::CreateApplicationCommand {
        command
            .name("expand")
            .description("conse wrap up expand")
            .create_option(|opt| {
                opt
                    .name("bullet")
                    .description("bullet list number for expantion")
                    .kind(CommandOptionType::Integer)
                    .min_int_value(1)
                    .max_int_value(1000)
                    .required(true)
            })
    }

    //// command will be over written later thus it must be defined mutable
    pub fn help_register(command: &mut builder::CreateApplicationCommand) -> &mut builder::CreateApplicationCommand {
        command
            .name("help")
            .description("conse wrap up help")

    }

    //// command will be over written later thus it must be defined mutable
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