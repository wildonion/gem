

<p align="center">
    <img width=350 height=350 src="https://github.com/wildonion/gem/blob/master/assets/conse.png"
</p>

## 🖥 Conse

Conse is an AI based Crypto Game Event Manager Platform on top of [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) and Solana blockchain. 

## 🧪 Test Conse Hyper Server

```cargo test --bin conse```

## 🏃 Run Conse Hyper Server

```cargo run --bin conse```

## 🏃🏽‍♀️ Run Conse Actix Panel Server

```cargo run --bin panel```

## 🏃🏻‍♀️ Run Conse Discord Bot Server

```cargo run --bin dis-bot```

## 🏃🏿 Run Conse Test Codes

```cargo run --bin tests```

## 🛠️ Development Setup

> To update a user access level to dev first signup the user using `/auth/signup` API then run the binary server like so: `./cosne wildonion 0` or `cargo run --bin conse wildonion 0` finally login with dev user to register a new god for the game.

## 🛠️ Production Setup

> First run ```sudo chmod +x deploy.sh && ./deploy.sh``` to up and run docker containers then to update a user access level to dev, first signup the user using `/auth/signup` API then update the `access_level` field of the user to `0` manually inside the db in `mongodb` container using `portrainer` finally login with dev user to register a new god for the game.

> To access the `mongodb` container shell, login to the `portrainer` then fireup the `mongodb` container CMD and run ```mongosh --port 7441``` or you can go inside using ```sudo docker exec -it mongodb mongosh --port 7441``` command.

> After updating application's `docker-compose.yml` file, we should rebuild our container images by running ```./deploy.sh``` script again.

## 🗃️ Directory Explained

* `src`: all in-game APIs which are related to the player app.
    * `contexts`: 
        * `bot`: Discord and Twitter bots .
        * `panel`: Dev and Admin panel app written in Yew and Tauri.
        * `blockchain`: Solana Anchor smart contract.
    * `controllers`: in game API async controllers.
    * `routers`: in game API routers.
    * `schemas`: in game Db schemas.
* `devops`: all devops configs and setup files.

## 🍟 Notes

* First run ```sudo chmod +x setup.sh && ./setup.sh``` to setup the VPS for both development and production.

* Remember to change the `DB_HOST`, `REDIS_HOST` and `SURREAL_DB_HOST` in `.env` file to their container name.

* since we're using docker compose to build the docker images the network that continas those images will be `gem_net` because ther directory name that the `docker-compose.yml` file is inside of is `gem` thus docker will create a network bridge with the prefix of the directory name or `gem` in this case and put every network created inside the `docker-compose.yml` file into this category.    

* `gem_net` is the network that contains `gem-redis`, `gem-mongodb`, `gem-conse`, `gem-haproxy` and `gem-catchup-bot` containers.

* connect to `mongodb` container either in portrainer or terminal using ```docker exec -it mongodb mongosh --port 7441```.

* in order to use docker containers inside another one by its DNS name, all of them must be inside the same network bridge like if we want to use the mongodb container inside the gem container they must be in the same network called `gem`. 

* clean docker cache using ```sudo docker buildx prune --all``` or ```docker system prune --all``` command.

* register push notification strategy: client `<--hyper REST-->` register a push notif route using redis client `<--REDIS SERVER-->` register pubsub topic (emit events) on redis server.

* subscribing to push notification strategy: client `<--gql subscription-->` redis published topics (emitted events) inside the server.

* subscribing to realtiming chat strategy: client `<--gql subscription ws-->` hyper gql ws server contains redis and mongodb clients setup `<--REDIS & MONGODB SERVER-->` store data on redis for caching and persistence in mongodb.

## 🚧 WIP

* setup **TLS/SSL** for `hyper`, `ws` and `actix` servers using `tokio-rustls` and `openssl` based on certbot certificate files.

* `ed25519` keypair for server checksum, verification using its commit (like ssh keys) and **SSL/TLS** certificate, updating app and time hash based (`hash(**user_id + time + ip + user agent**)`) locking api with rate limit feature to avoid api call spamming (like sleeping in thread) using `argon2`, `rust-crypto`, `noise`, `ring` and `ed25519-dalek` tools, also see the one inside the [payma](https://github.com/wildonion/payma) repo.

* create a proc macro attribute like `#[passport]` to put on top of the auth controllers.

* complete graphql, redis and websocket routes and controllers setup for realtime strategies like game monitoring, chatapp and push notification.

* balance the loads between conse docker services and images inside the `docker-compose` file using `k8s` on `DigitalOcean` PaaS on `gem` repo, also CI/CD configuration files based on the latest commits and managing containers using [portainer](https://www.portainer.io/).

* all TODOs inside the app, `panel` and twitter bot services.

* backend design pattern sketch using freeform inside wiki.

* communication between Conse and the [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) must be done through the TCP stream since [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) supports TCP stream.

* use an AI model like [STEM](https://github.com/wildonion/stem) which suggests player the tips and tricks for a new game based on behavioural graph of the player collected by the history of each event's `phases` field

* `MMQ` and order matching engine to suggests players events and other games based on their past experiences, scores (MMR) and earned tokens also and order matching engine for players to sell their minted roles based on highest or lowest order in order book.  

* use [uniXerr](https://github.com/wildonion/uniXerr) coin generation AI model which players get rewarded based on their scores and positions which are collected by each event manager inside the game, then update the balance field of the user based on that
