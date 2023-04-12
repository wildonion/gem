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

--- bot link example --- 
https://discord.com/api/oauth2/authorize?client_id=1092048595605270589&permissions=277025475584&scope=bot%20applications.commands

get token from : https://discord.com/developers/applications/1092048595605270589/bot


command examples:

    → show the help message
        !help conse

    → feed the chat GPT all the messages before the passed in hours ago (4 hours ago in this case) for summarization
        !conse wrapup 4
    
    → feed the chat GPT the selected bullet list to exapnd it
        !conse expand 2  
```
