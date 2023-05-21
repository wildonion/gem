


# 🤏 Conse

Conse is an AI based Crypto Game Event Manager Platform on top of [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) and Solana blockchain. 

<p align="center">
    <img width=350 height=350 src="https://github.com/wildonion/gem/blob/master/assets/conse.png"
</p>

## 🛠️ Development Setup

> Note that to update a user access level to `dev` first do a signup for the user using `/auth/signup` API then run the binary server like so: `./cosne wildonion 0` or `cargo run --bin conse wildonion 0` finally login with that user to register a new god for the game.

### 🧪 Test Conse Hyper Server

```cargo test --bin conse```

### 🏃 Run Conse Hyper Server

```cargo run --bin conse```

### 🏃🏽‍♀️ Run Conse Actix Panel Server

```cargo run --bin panel```

### 🏃🏻‍♀️ Run Conse Discord Bot Server

```cargo run --bin dis-bot```

### 🏃🏿 Run Conse Test Codes

```cargo run --bin tests```

## 🚀 Production Setup

> Make sure that you have a domain up and running that is pointing to the machine where the `gem` is hosted on.

> To access the `mongodb` container shell, login to the `portrainer` then fireup the `mongodb` container CMD and run ```mongosh --port 7441``` or you can go inside using ```sudo docker exec -it mongodb mongosh --port 7441``` command.

> After updating application's `docker-compose.yml` file, we should rebuild our container images by running ```./deploy.sh``` script again.

First run ```sudo chmod +x deploy.sh && ./deploy.sh``` to up and run docker containers then to update a user access level to dev, first signup the user using `/auth/signup` API then update the `access_level` field of the user to `0` manually inside the db in `mongodb` container using `portrainer` finally login with dev user to register a new god for the game.

## 🗃️ Directory Explained

* `core`: all in-game APIs which are related to the player app.
    * `contexts`: 
        * `bot`: Discord and Twitter bots .
        * `panel`: Dev and Admin panel app written in Yew and Tauri.
        * `blockchain`: Solana Anchor smart contracts.
    * `controllers`: in-game API async controllers.
    * `routers`: in-game API routers.
    * `schemas`: in-game mongodb schemas.
* `infra`: all infrastructure configs and setup files.

## 🍟 Notes

* all docker container the mounted volumes are inside `infra/data` folder. 

* use ```diesel migration generate <MIGRAION_NAME>``` to create the migration file for your postgres table, ```diesel migration redo``` to drop the table and ```diesel migration run``` to apply all migration tables to your database.

* first run ```sudo chmod +x setup.sh && ./setup.sh``` to setup the VPS for both development and production.

* note that if you want to use an authorized db (mongodb or postgres) connection just update the `DB_USERNAME` and `DB_PASSWORD` inside the `.env` and change the `ENVIRONMENT` variable to `prod`.

* since we're using docker compose to build the docker images the network that continas those images will be `gem_net` because ther directory name that the `docker-compose.yml` file is inside of is `gem` thus docker will create a network bridge with the prefix of the directory name or `gem` in this case and put every network created inside the `docker-compose.yml` file into this category.    

* `gem_net` is the network that contains `gem-redis`, `gem-mongodb`, `gem-postgres`, `gem-adminer`, `gem-conse-panel`, `gem-conse`, `gem-haproxy` and `gem-bot` containers.

* connect to `mongodb` container either in portrainer or terminal using ```docker exec -it mongodb mongosh --port 7441```.

* connect to `postgres` container either in portrainer or terminal using ```docker exec -it postgres psql -u postgres conse```.

* in order to use docker containers inside another one by its DNS name, all of them must be inside the same network bridge like if we want to use the mongodb container inside the gem container they must be in the same network called `gem`. 

* clean docker cache using ```sudo docker buildx prune --all``` or ```docker system prune --all``` command.

* conse client can subscribes to the fired or emitted role reveal event and topics by sending websocket connections to the redis server docker on the VPS in the meanwhile we're sure that the `/reveal/roles` API has been called by the dev or the god inside the panel thus players can see their roles without refreshing the page.

## 🚧 WIP

* setup **TLS/SSL** for `hyper`, `ws` and `actix` servers using `tokio-rustls` and `openssl` over certbot certificate files, note that for this we must have a domain poiting to the VPS that the gem is inside.  

* `ed25519` keypair for server checksum, verification using its commit (like ssh keys) and **SSL/TLS** certificate, updating app and time hash based (**`hash(user_id + time + ip + user agent)`**) locking api with rate limit feature to avoid api call spamming (like sleeping in thread) using `argon2`, `rust-crypto`, `noise`, `ring` and `ed25519-dalek` tools, also see the one inside the [payma](https://github.com/wildonion/payma) repo.

* all TODOs inside the app, `panel` and twitter bot services also create a proc macro attribute like `#[passport]` to put on top of the auth controllers.

* check the containers status using using [portainer](https://www.portainer.io/), balance the loads between conse docker services and images inside the `docker-compose` file using `k8s` on `DigitalOcean` PaaS over `gem` repo.

* backend design pattern sketch using freeform and moongodb ERD schemas inside wiki.

* communication between Conse and the [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) must be done through the TCP stream since [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) supports TCP stream.

* use an AI model like [STEM](https://github.com/wildonion/stem) which suggests player the tips and tricks for a new game based on behavioural graph of the player collected by the history of each event's `phases` field

* `MMQ` and order matching engine to suggests players events and other games based on their past experiences, scores (MMR) and earned tokens also and order matching engine for players to sell their minted roles based on highest or lowest order in order book.  

* use [uniXerr](https://github.com/wildonion/uniXerr) coin generation AI model which players get rewarded based on their scores and positions which are collected by each event manager inside the game, then update the balance field of the user based on that
