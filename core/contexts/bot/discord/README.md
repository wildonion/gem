<p align="center">
    <img width=350 height=250 src="https://github.com/wildonion/gem/blob/master/assets/disbot.png"
</p>

# 🤖 Conse Discord Bot

> [bot link](https://discord.com/api/oauth2/authorize?client_id=1092048595605270589&permissions=277025475584&scope=bot%20applications.commands)

## 📋 Requirements

> Make sure that the redis server is up and running before starting the bot, otherwise it'll halt on `The application did not respond`.

```console
sudo apt install libssl-dev libudev-dev pkg-config
```

## 🚀 Deploy

```console
sudo docker build -t catchup-bot .
sudo docker run --name catchup-bot -v ./infra/data/logs:/usr/src/app/logs/ -d catchup-bot
```

## 📱 Features

* show the sysinfo and servers status of the conse server

* channel messages summerization using chat GPT

* configured with 10 shards by default also we're using redis to manage the shared state data between clusters.

* remember to fill the `OPENAI_KEY` and `DISCORD_TOKEN` variables with your credentials inside the `.env` file.

* with [this link](https://discord.com/api/oauth2/authorize?client_id=1092048595605270589&permissions=277025475584&scope=bot%20applications.commands) we can add the conse bot to discord servers.  

> get token from [here](https://discord.com/developers/applications/1092048595605270589/bot)
