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

* remember to fill the `OPENAI_KEY` and `DISCORD_TOKEN` variables with your credentials inside the `.env` file.

* with [this link](https://discord.com/api/oauth2/authorize?client_id=1092048595605270589&permissions=277025475584&scope=bot%20applications.commands) we can add the conse bot to discord servers.  

> get token from [here](https://discord.com/developers/applications/1092048595605270589/bot)

## 📱 Features

* show the sysinfo and servers status of the conse server

* channel messages summerization using chat GPT

* configured with 10 shards by default also we're using redis to manage the shared state data between clusters.

## 🚀 Deploy

> Make sure that you've ran the `scripts/setup.sh` already, so the tokens can be accessible from `.env`.

```bash
cd /$USER/gem/infra
sudo docker build -t conse-catchup-bot -f $(pwd)/infra/docker/catchup-bot/Dockerfile . --no-cache
sudo docker run -d --link redis --network gem --name conse-catchup-bot -v $(pwd)/infra/data/catchup-bot-logs/:/usr/src/app/logs/ conse-catchup-bot
```