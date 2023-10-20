


discord bot to pull out new admin tasks using redispubsub into a discord channel also it has the following features:

- auto assigning roles for different points thresholds/task completion rate

- exclusive raffles/giveaways above x number of points

- broadcast new task topic to a discord channel once the admin created a new one

## How 2?

> note that you have to setup an application in your discord development account to get the key

step 0. fill the `XCORD_*` vars inside the `.env` with your token and channel id

step 1. deploy the redis docker container

step 2. deploy the docker container of `xcord` bot

## Flow

> basically this bot subscribes to new task topic constantly using redis then it'll broadcast the task into the discord channel.

```
in panel server 
    |
    |------ once a new task is created
    | 
     ------ publish new task topic to redis pubsub channel

in discord ws/http client 
    |
    |------ subscribe to the published new task topic inside the event listener (loop {})
    |
     ------ send them to all discord channel(s) of all guilds that this bot is inside
```