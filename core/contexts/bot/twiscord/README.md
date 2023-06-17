
# 🤖 Conse Twiscord Bot

this bot subscribes to redis pubsub channels related to twitter's mentions, tweets and replies topics and send them to the specified channel in discord. 

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

> Make sure that you've ran the `scripts/setup.sh` already, so the tokens can be accessible from `.env`.

> Make sure that you've setup the [twidis](https://github.com/wildonion/twidis) bot already in order to get this bot works.  

```bash
cd /$USER/gem/infra
sudo docker build -t twiscord -f $(pwd)/infra/docker/twiscord/Dockerfile . --no-cache
sudo docker run -d --link redis --network gem --name twiscord -v $(pwd)/infra/data/twiscord-logs/:/usr/src/app/logs/ twiscord
```