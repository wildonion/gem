
# 🤖 Conse Twiscord Bot

Code Flow:

````
in twitter bot server 
    - once the fetch user mentions api is called
    - publish response to redis pubsub channel

in discord ws/http client 
    - subscribe to the published tweets inside the event listener (loop {})
    - send them to all discord channel(s) of all guilds that this bot is inside
```