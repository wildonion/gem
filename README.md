


# 🤏 Conse Backend Rust Services

Conse is an AI based Crypto Game Event Manager Platform on top of [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) and Solana blockchain which uses: 
- a based [STEM](https://github.com/wildonion/stem) like AI model which will suggests players the tips and tricks for a new game based on behavioural graph of the each player collected by the history of each event's `phases` field inside the game.
- [uniXerr](https://github.com/wildonion/uniXerr) coin generation AI model which players get rewarded based on their scores and positions, collected by each event manager inside the game, then update the balance field of the user based on those attributes.
- match making rating (**MMR**) graph engine which is a weighted tree that suggests players events and other games based on their past experiences, scores and earned tokens during the game also players can sell their minted roles based on highest or lowest order setup in conse order book.
- event collaboration queue (**ECQ**) system in which admins can share their registered events and collaborate with other admins.

<p align="center">
    <img width=350 height=350 src="https://github.com/wildonion/gem/blob/master/assets/conse.png"
</p>

```bash
# panel dev username/password              : devdevy/d3v@%$^$3hjsD
# panel admin username/password            : adminy/4dmin@%$^$3hjsD
# postgres adminer username/password/server: postgres/geDteDd0Ltg2135FJYQ6rjNYHYkGQa70/postgres
🥛 PUSH NOTIFICATION ROUTE ==> ws://ws.panel.conse.app/notifs/
🌍 MAIN SITE ==> https://conse.app/
👨🏻‍⚖️ ADMIN PANEL ==> https://panel.conse.app/
🛤️ ADMIN/DEV API ROUTE ==> https://api.panel.conse.app/
🗺️ MAIN API ROUTE ==> https://api.conse.app/
📡 SWAGGER DOC ==> https://api.panel.conse.app/swagger/
🛢️ ADMINER PANEL ==> https://adminer.conse.app
🎙️ HOSTED ON ==> Digitalocean
🚉 TLPs ==> https://github.com/wildonion/gem/wiki/TLPs
🥪 GEM ERD SCHEMAS ==> https://github.com/wildonion/gem/wiki/Gem-ERD-Schemas
```

## 🍩 V0.1.3 Features

* **🦀 Rust flows in you 🌊**: write codes that are the matter-of-futer flows!

* **☕ sit-back-and-drink-your-coffee** scripts do deploy the project on VPS!

* 🦾 functional, macro, MVC and ACL based design pattern server APIs

* ☢️ better error handling using **match** and **Option** syntax

* ❌ custom error type (`PanelError`) to handle all possible server (actixweb and websocket) and storage (redis and diesel) IO errors in conse panel.
 
* 🧑🏻‍💼 game managers can define score based twitter tasks for users, reveal role, collaborate with other admins and share their registered events using conse **ECQ** (Event Collaboration Queue) system and advertise their events via SMS inside the panel  

* 🍪 **cookie** and **JWT** based authentication strategy

* 🔐 **Argon2** as the **KDF** used for password hasing

* 🥝 server health-check APIs (check-token, health and logout)

* 🍅 catchup discord bot for channel messages summarization 

* 📡 **swagger** docs using **utoipa openapi** for all admin, dev and user panel APIs supports all possible server's responses 

* 🎒 supports **postgres**, **mongodb** and **redis** as the app storage  

* 🛎️ **actix web** and **hyper** based HTTP servers

* 📣 **redis** based pubsub streaming channel to publish and subscribe to the reveal role, **ECQ** (Event Collaboration Queue), **MMR** (Match Making Rating), new task defined by admins, task verification logs and twitter bot responses topics

* 💾 **redis** http response caching to avoid high latencies cause I believe reading from RAM is much faster than HardDisk.   

* 🎯 **actix ws** server for streaming over redis subscribed topics  

### 🗃️ Directory and Structure Explained

> Note that to use dev panel APIs Remember to run conse hyper server first.

* `core`: hyper server which are related to the player app.
    * `contexts`: 
        * `bot`: serenity discord and twiscord bots.
        * `panel`: dev and admin panel actix server and yew app.
    * `controllers`: in-game async controllers related to hyper server.
    * `routers`: in-game API routers related to hyper server.
    * `schemas`: in-game mongodb schemas related to hyper server.
* `infra`: all infrastructure and devops configs.
* `errors`: gem possible errors handler
* `jobs`: gem crontab jobs
* `logs`: gem log files generated by discord bot service, conse panel and other parts of the app.
* `migrations`: diesel postgres sql files
* `scripts`: deployment scripts
* `test`: gem test codes

**NOTE**: All `conse`, `panel` and `bot` services are just different binaries which are sharing a same `Cargo.toml` setup.

## 🛠️ Development Setup

> Before developing, read the following notes: 

- **NOTE**: makre sure that you've installed the following packages on **MacOS M1**:
```bash
brew install openjdk
brew install pkg-config
brew install openssl
brew install diesel
brew link --force openssl
brew install libpq && brew link --force libpq
brew install graphviz
cargo clean
```

- **NOTE**: all docker container the mounted volumes are inside `infra/data` folder. 

- **NOTE**: if you want to extend the last table fields first update its `up.sql` file then run ```diesel migration redo``` and finally ```diesel migration run```, to regenerate all tables run ```diesel migration redo -n 3``` which **3** refers to the number of tables we've created so far.

- **NOTE**: before migrating any table, make sure that you've an already setup database using ```diesel setup && diesel migration run``` command.

- **NOTE**: use ```diesel migration generate <MIGRAION_NAME>``` to create the migration file containing the postgres table setup, ```diesel migration redo``` to drop the table and ```diesel migration run``` to apply all migration tables to the database after submitting changes to the sql fiels.

- **NOTE**: to update a user access level to `dev` first do a signup for the user using `/auth/signup` API then run the conse binary server like so: `./cosne wildonion 0` or `cargo run --bin conse wildonion 0` finally login with that user to register a new god for the game.

- **NOTE**: in development environment remember to fill the `OPENAI_KEY`, `DISCORD_TOKEN`, `TWITTER_BEARER_TOKEN`, `TWITTER_ACCESS_TOKEN`, `TWITTER_ACCESS_TOKEN_SECRET`, `TWITTER_CONSUMER_KEY`, `TWITTER_CONSUMER_SECRET`, `TWITTER_CLIENT_ID` and `TWITTER_CLIENT_SECRET` vars inside the `.env` with appropriate values, but for production deployment remove these fields from `.env`

```bash
# 🧪 Test Conse Hyper Server
cargo test --bin conse
# 🏃 Run Conse Hyper Server
cargo run --bin conse
# 🏃🏽‍♀️ Run Conse Actix Panel Server
cargo run --bin panel
# 🏃🏻‍♀️ Run Conse Discord Bot Server
cargo run --bin catchup-bot
# 🏃🏻‍♀️ Run Conse Twiscord Bot Server
cargo run --bin twiscord
# 🏃🏿 Run Conse Argon2 Test Codes
cargo run --bin argon2test
```
    
## 🚀 Production Setup

> Before going for production, read the following notes: 

- **NOTE**: there is a env var called `THIRD_PARY_TWITTER_BOT_ENDPOINT` which can be set to an external twitter bot server endpoint to send requests for user task verification, if you want to use a third party bot remember to pass the endpoint to the instance of the `Twitter` struct like `let bot = Twitter::new(Some(bot_endpoint));`.

- **NOTE**: currently the `/bot/check-users-tasks` API will be called every day at **7 AM** via a setup crontab inside the `jobs` folder to avoid twitter rate limit issue, if you want to change the cron just run `crontab -e` command inside the `jobs` folder and edit the related cron file.

- **NOTE**: in order to use twitter APIs you must have a paid developer account and you must use keys and tokens from a twitter developer App that is attached to a Project also you can add new keys in `twitter-accounts.json` by calling the `/admin/add-twitter-accounts` API.

- **NOTE**: to generate a new password for admin and dev users just edit the `argon2test.rs` code inside the `tests` folder then run ```cargo run --bin argon2test``` to generate those passwords finally update the `up.sql` inside the `migrations/2023-05-22-184005_users` folder to insert a new admin and dev user info into the table when you run ```diesel migration run```. 

- **NOTE**: to access the `mongodb` container shell, login to the `portrainer` then fireup the `mongodb` container CMD and run ```mongosh``` or you can go inside using ```sudo docker exec -it mongodb mongosh``` command.

- **NOTE**: after updating application's `Dockerfile` files, we should rebuild our container images by running ```./deploy.sh``` script again.

- **NOTE**: to update a user access level of the conse hyper server to dev, first signup the user using `/auth/signup` API then update the `access_level` field of the user to `0` manually inside the db in `mongodb` container using `portrainer` finally login with dev user to register a new god for the game.

- **NOTE**: in order to use docker containers inside another one by its DNS name, all of them must be inside the same network bridge like if we want to use the mongodb container inside the panel container they must be in the same network called `gem`. 

- **NOTE**: Make sure that you have the `conse.app` domain enabled and is pointing to the machine where the `gem` codes is hosted on.

- **NOTE**: Rerun the `renew.sh` on every changes to the nginx config file like hosting new codes, services or adding a new domain to the VPS.

- **NOTE**: For every new (sub)domain inside the VPS there must be a new config file and a new ssl certificate inside the `infra/docker/nginx` folder related to that (sub)domain name.

- **NOTE**: There must be three registered (sub)domains in DNS panel of `conse.app`: `api.conse.app`, `api.panel.conse.app`, `panel.conse.app` which points to the conse hyper APIs, Actix APIs and the panel UI respectively.

- **NOTE**: To serve static files using nginx just make sure you copied the `build-{PROJECT-NAME}` folder of JS projects into `infra/docker/nginx/build` folder.   

- **NOTE**: Multiple domains can point to a same VPS which their ssl-s and routes can be setup by nginx also multiple (sub)domains of different domains can point to multiple VPS-es which can be setup inside the DNS panel of those domains like the following:

**DNS records of conse.app domain**

```
Type	    Hostname	               Value	          TTL (seconds)	
A	    conse.app              directs to 64.226.71.201	     3600
A	    api.conse.app   	   directs to 68.183.137.151     3600 
A	    panel.conse.app    	   directs to 68.183.201.134     3600 
```
**DNS records of wildonion.io domain**

```
Type	    Hostname	               Value	          TTL (seconds)	
A	    wildonion.io           directs to 64.226.71.201	     3600
A	    api.wildonion.app      directs to 68.183.137.154     3600 
A	    admin.wildonion.app    directs to 68.183.201.129     3600 
```
in the above records `wildonion.io` and `conse.app` are pointing to a same VPS but their (sub)domains are pointing to different VPS-es. 

```bash
# -----------------------
# ---- read/write access
sudo chown -R root:root . && sudo chmod -R 777 .
sudo chmod +x /root && sudo chown -R root:root /root && sudo chmod -R 777 /root
cd scripts
# ---------------
# ---- setup VPS
./setup.sh
# ---------------
# ---- deploy containers
./redeploy.sh
# ---------------
# ---- renew nginx 
./renew.sh
```
    
## 🪟 Schemas and ERDs

> Note that to regenerate the ERD from the postgres database just run ```sqlant postgresql://postgres:<PASSWORD>@localhost/conse > infra/panel.uml && java -jar infra/plantuml.jar infra/panel.uml```.

### 🥪 Conse Panel Postgres ERD Schema

<p align="center">
    <img src="https://github.com/wildonion/gem/blob/master/infra/panel.png">
</p>

### 🍢 Conse Mongodb ERD Schema

<p align="center">
    <img src="https://github.com/wildonion/gem/blob/master/infra/conse.schema.PNG">
</p>

### 🖼️ [Conse Panel](https://github.com/wildonion/gem/tree/master/core/contexts/panel) Architecture Diagram

<p align="center">
    <img src="https://github.com/wildonion/gem/blob/master/infra/arch.jpg">
</p>

### 🥧 [Twiscord](https://github.com/wildonion/gem/tree/master/core/contexts/bot/twiscord) Architecture Diagram

<p align="center">
    <img src="https://github.com/wildonion/gem/blob/master/infra/rediscord.png">
</p>

## 🧐 WrapUps 

* to all gorgeous admins, **if you don't want to set a new password for a user then don't pass that field to the request body** of the `/admin/edit-user` API, also the `role` field must be **uppercase** and it's default value when it's not passed is **Dev**.

* the generated cookie inside the response of the conse panel admin and user login APIs is in form `<JWT>::<SHA256_OF_LOGIN_TIME>`.

* admin and user login APIs of conse panel returns a response which contains the generated cookie for the user in which we can use the first part of `::` sign, as the **JWT** to send authorized requests in postman and swagger UI. 

* all conse panel APIs except admin and user login, health check and logout APIs need a **JWT** inside the Authorization header or the cookie variable in the their request objects also the **JWT** can be set in their swagger UI page using the `Authorize 🔒` button. 

* in order to reveal the roles of an event, admin **JWT** token generated by the conse hyper server inside the response of the login API, must be passed to the `/admin/notif/register/reveal-role/{event_id}` API of the panel server.   

* **admins** are game managers and **users** are players. 

* conse client can subscribes to the fired or emitted events and topics like role reveal, ecq, new tasks and task verification logs and see notifications coming from redis docker server by sending websocket packets to the actix websocket server.

* pubsub new task, twitter task verification response, twitter bot response, **ECQ**, **MMR** and reveal role topics are `tasks`, `task-verification-responses`, `twitter-bot-response`, `ecq-{event_id}`, `mmr-{event_id}`, `reveal-role-{event_id}` respectively.   

* push notification routes for new task, twitter task verification response, twitter bot response, **ECQ**, **MMR** and reveal role topics are `ws://ws.panel.conse.app/notifs/tasks`, `ws://ws.panel.conse.app/notifs/task-verification-responses`, `ws://ws.panel.conse.app/notifs/twitter-bot-response`, `ws://ws.panel.conse.app/notifs/ecq-{event_id}`, `ws://ws.panel.conse.app/notifs/mmr-{event_id}`, `ws://ws.panel.conse.app/notifs/{user_objectid}/reveal-role-{event_id}` respectively.   

* twitter task names defined by admins, must be prefixed with `twitter-*` and are twitter activities such as `username` which is a task that must be done to verify his/her twitter username, `tweet` which can be a specific content or the generated code by the backend, `like`, `hashtag` and `retweet` that must be done to reward users based on the score of each task.

* admins can define multiple twitter tasks in the same activity, all tasks will be separated by a random chars like `*-<RANDOM_CHARS>` so the final task name will be `twitter-username-iYTC^`.

* every day at **7 AM** all the users tasks will be checked automatically using a cronjob to see that the user is still verified or not, this will be done by checking all the records of the `users_tasks` table inside the `/check-users-tasks` API. 

* once the user is loggedin, first the `/user/verify-twitter-account/{account_name}` API must be called with **user** token to update the twitter username of the user inside the db then then we must compel the user to tweet the activity code which is inside the user data response, after that the `/bot/verify-twitter-task/{job_id}/{twitter_username}` API must be called to verify the users' tasks, this can be behind a **check** button in frontend or inside an intreval http call.

## 🚧 WIPs

* admin SMS panel to advertise the event

* solana [ticket reservation contract](https://github.com/wildonion/solmarties/tree/main/programs/ticket)

* redis pubsub streaming to publish reveal role, ecq (for registered events) and mmr (for event suggestion to players) topics inside `core/contexts/panel/events/redis` folder.

* setup websocket nginx config file (ws://ws.panel.conse.app/notifs/)

* macros inside the `core/contextss/panel/misc.rs` and a proc macro attribute like `#[passport(access=2)]` to put on top of the admin, user and dev APIs, struct and their fields

* god and dev panel app using `yew`

* publish docker containers to docker hub also add CI/CD setup like digitalocean platform