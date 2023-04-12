<p align="center">
    <img width=350 height=250 src="https://github.com/wildonion/gem/blob/master/assets/disbot.png"
</p>

# 🤖 Conse Discord Bot


## 📋 Requirements

```console
sudo apt install libssl-dev libudev-dev pkg-config
```

## 📱 Features

* show the sysinfo status of the conse server

* channel messages summerization using chat GPT

## Commands

```
--- sources ---
https://blog.logrocket.com/building-rust-discord-bot-shuttle-serenity/
https://github.com/serenity-rs/serenity/tree/current/examples/
https://github.com/serenity-rs/serenity/blob/current/examples/e05_command_framework/src/main.rs

--- bot link example --- 
https://discord.com/api/oauth2/authorize?client_id=1092048595605270589&permissions=277025483776&scope=bot
https://discord.com/oauth2/authorize?client_id=1092048595605270589&scope=applications.commands

get token from : https://discord.com/developers/applications/1092048595605270589/bot


command examples:

    → show the help message
        !help conse

    → feed the chat GPT all the messages before the passed in hours ago (4 hours ago in this case) for summarization
        !conse wrapup 4
    
    → feed the chat GPT the selected bullet list to exapnd it
        !conse expand 2  
```
