


# 🏗 Conse PaaS

Conse is an AI based Crypto Game Event Manager Platform on top of [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) blockchain. 

> To update a user access level to dev first signup the user using `/auth/signup` API inside the `PaaS` then run the binary like so: `./app wildonion 0`

## 🧪 Test Conse Server

```cargo test --bin conse```

## 🛠️ Setup

```sudo chmod +x app.sh && ./app.sh```

### 🚧 WIP

* HAProxy, k8s-ing docker containers in `docker-compose.yml` and CI/CD in `app.sh`

* all TODOs inside the app

* communication between Conse and the [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) must be done through the TCP stream since [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) supports TCP stream.

* an AI model which suggests player the tips and tricks for a new game based on behavioural graph of the player collected by the history of each event's `phases` field

* coin generation based on player scores which are collected by each event manager inside the game, then update the balance field of the user based on that

