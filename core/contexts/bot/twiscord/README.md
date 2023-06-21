
# 🤖 Conse Twiscord Bot

> [bot link](https://discord.com/api/oauth2/authorize?client_id=1121128286433595504&permissions=274877908992&scope=bot)

this bot subscribes to redis pubsub channels related to twitter's mentions, tweets and replies topics and send them to the specified channel in discord. 

> with [bot link](https://discord.com/api/oauth2/authorize?client_id=1121128286433595504&permissions=274877908992&scope=bot) we can add the conse bot to discord servers. 

> get token from [here](https://discord.com/developers/applications/1121128286433595504/bot)

**Code Flow:**

```
in twitter bot server 
    |
    |------ once the fetch user mentions api is called
    | 
     ------ publish response to redis pubsub channel

in discord ws/http client 
    |
    |------ subscribe to the published tweets inside the event listener (loop {})
    |
     ------ send them to all discord channel(s) of all guilds that this bot is inside
```

## 🚀 Deploy

Make sure that

- you've ran the `scripts/setup.sh` already, so the token and the discord channel id can be accessible from `.env`, also before running the  script please build a new application for this bot inside the discord developer panel to get the token and invitation link.

- this bot and redis must be in a same docker network.

- you've setup the [twidis](https://github.com/wildonion/twidis) bot already in order to get this bot works.  

```bash
cd /$USER/gem/infra
sudo docker build -t twiscord -f $(pwd)/infra/docker/twiscord/Dockerfile . --no-cache
sudo docker run -d --link redis --network gem --name twiscord -v $(pwd)/infra/data/twiscord-logs/:/usr/src/app/logs/ twiscord
```

## 🖼️ Twiscord Architecture Diagram

<p align="center">
    <img src="https://github.com/wildonion/gem/blob/master/infra/rediscord.png">
</p>
