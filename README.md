


# 🤏 Conse Rust Backend and Engines

Conse is a crypto based friendly gathering **Game Event Manager** and advertising platform on top of Polygon uses the following engines: 
- **pubsub** pattern to reveal player in-game roles using the redis publisher and subscriber and websocket server to notify players of new roles once the server subscribed to the revealed roles topic.
- event collaboration queue (**ECQ**) system in which admins can share their registered events and collaborate with other admins.
- behavioural graph virtual machine (**GVM**) built on top of each event's `phases` field inside the game for each player to suggests them the tips and tricks for a new game and reward them based on their game scores using an AI based coin generation model in essence, each player gets rewarded and ranked based on their scores and in-game positions then the `balance` field will be updated based on those attributes, the match making rating (**MMR**) engine, on the other hand is is a weighted tree based suggestion engine that suggests players, events and other games based on their past experiences, scores, tokens and rewards earned using **GVM** during the game.

## 🚟 Infra Routes and APIs

> Remember to setup jenkins and portainer panel, for jenkins, we should use the administrator password which can be seen inside `jenkins-blueocean` container logs, after that we can create a pipeline job in jenkins and setup a webhook in **gem** repo to start building automatically on every push through the jenkins pipeline schema, for more info refer to [this](https://www.jenkins.io/doc/tutorials/build-a-node-js-and-react-app-with-npm/) setup.

```bash
# conse panel dev username/password              : devdevy/d3v@%$^$3hjsD
# conse panel admin username/password            : adminy/4dmin@%$^$3hjsD
# postgres adminer username/password/server: postgres/geDteDd0Ltg2135FJYQ6rjNYHYkGQa70/postgres
🥛 WEBSOCKET PUSH NOTIFICATION ROUTE ==> wss://notif.panel.conse.app/subscribe/
🌍 MAIN SITE ==> https://conse.app/
👨🏻‍⚖️ ADMIN PANEL ==> https://panel.conse.app/
🛤️ ADMIN/DEV API ROUTE WITH POSTGRES DB STORAGE ==> https://api.panel.conse.app/
🛣 ADMIN/DEV API ROUTE WITH MONGO DB STORAGE ==> https://api.panel.conse.app/mongo
🗺️ MAFIA API ROUTE ==> https://api.mafia.conse.app/
📡 SWAGGER DOC ==> https://api.panel.conse.app/swagger/
🛢️ ADMINER PANEL ==> https://adminer.conse.app
🛎️ JENKINS PANEL ==> https://jenkins.conse.app
⛵ PORTAINER PANEL ==> https://portainer.conse.app
🎙️ HOSTED ON ==> Digitalocean
```

## 🍩 Panel APIs V0.1.3 Infra Features

* **🦀 Rust flows in you 🌊**: write codes that are the matter-of-future flows!

* **☕ sit-back-and-drink-your-coffee** scripts do deploy the project on VPS!

* 🦾 functional, macro, MVC and ACL based design pattern server APIs

* 🎨 **Jenkins** based CI/CD

* ☢️ better error handling using **match** and **Option** syntax, also all errors are in form `Result<actix::HttpResponse, actix::Error>` which allows the client to know the all in-app error reasons and prevent server from crashing.

* ❌ custom error handler (`PanelError`) to logs all possible server (actixweb and websocket) and storage (redis, redis async and diesel) IO errors into file in conse panel service.
 
* 🧑🏻‍💼 game managers (admins) can define score based twitter tasks for users (players), reveal role, collaborate with other admins and share their registered events using conse **ECQ** (Event Collaboration Queue) system and advertise their events inside the panel  

* 🍪 **cookie** and **JWT** based authentication strategy

* 🔐 **Argon2** as the **KDF** used for password hasing

* 📬 mail verification process for users

* 🥝 server health-check APIs (check-token, health and logout)

* 📡 **swagger** docs using **utoipa openapi** for all admin, dev and user panel APIs supports all possible server's responses 

* 🎒 supports **postgres**, **mongodb** and **redis** as the app storage  

* 🛎️ **actix web** (for handling push notif subscriptions and panel APIs) and **hyper** (conse mafia APIs) based HTTP servers

* 📣 **redis** based pubsub streaming channel to publish and subscribe to the revealed roles, **ECQ** (Event Collaboration Queue), **MMR** (Match Making Rating) topics

* 💾 **redis** http response caching to avoid high latencies cause we all know reading from RAM is much faster than HardDisk.   

* 🎯 **actix ws** notif servers for streaming over redis subscribed topics to send notifs to ws sessions.

* 🪙 **ECDSA-secp256k1** based cryptoghrapy algorithm in [wallexerr](https://crates.io/crates/wallexerr) is being used to generate a unique crypto Id for each user to allow them to sign in-game operations and API calls with their own private keys using **web3**, the signature verification process will be done in panel using [web3](https://crates.io/crates/web3) crate, to verify the signer.

### 🗃️ Directory and Structure Explained

> Note that to use dev and admin panel APIs Remember to run conse mafia hyper server first.

* `core`: hyper, actix web HTTP and actix WS servers.
    * `panel`: user, dev and admin dashboard panel APIs with actix web and actix WS server.
    * `nftport`: nftport fastapi server to upload file to ipfs.
    * `mafia`: mafia game APIs
        * `controllers`: in-game async controllers related to hyper server.
        * `routers`: in-game API routers related to hyper server.
        * `schemas`: in-game mongodb schemas related to hyper server.
* `infra`: all infrastructure and devops configs.
* `errors`: gem possible errors handler
* `jobs`: gem crontab jobs
* `logs`: gem log files generated by conse panel and other parts of the app.
* `migrations`: diesel postgres sql files
* `scripts`: deployment scripts
* `test`: gem test codes like admin and dev password generator script

**NOTE**: both `mafia` and `panel` services inside `core` are just different binaries which are sharing a same `Cargo.toml` setup.

## 📘 Git Books and Docs

* [Engines](https://conse.gitbook.io/engines)

* [Crypto ID](https://conse.gitbook.io/cid)

* [Reveal Role](https://conse.gitbook.io/reveal-role)

* [TLPS](https://github.com/wildonion/gem/wiki/TLPs)

## 🛠️ Development Setup

> Before developing, read the following notes: 

- **NOTE**: makre sure that you've installed the following packages on **MacOS M1**:
```bash
brew tap cossacklabs/tap
brew install libthemis
brew install openjdk
brew install pkg-config
brew install openssl
brew install diesel
brew link --force openssl
brew install libpq && brew link --force libpq
brew install graphviz
cargo clean
```

- **NOTE**: also make sure that you have a compatible version of `bigdecimal` crate with `diesel` by running `cargo tree -p diesel --depth=1` command to find the version used by `diesel` also add the `numeric` feature to diesel create, an example of running this command on MacOS M1 is (refer to [this](https://stackoverflow.com/questions/55783064/the-trait-dieselexpression-is-not-implemented-for-bigdecimalbigdecimal) issue on stackoverflow): 
```bash
diesel v2.1.0
├── bigdecimal v0.2.2
├── bitflags v2.3.1
├── byteorder v1.4.3
├── chrono v0.4.26
├── diesel_derives v2.1.0 (proc-macro)
├── itoa v1.0.6
├── num-bigint v0.4.3
│   [build-dependencies]
├── num-integer v0.1.45
│   [build-dependencies]
├── num-traits v0.2.15
│   [build-dependencies]
├── pq-sys v0.4.8
├── r2d2 v0.8.10
└── uuid v1.3.3
```

- **NOTE**: **Regards to conse panel actix APIs**, two docker instances of panel service will be built, one contains the postgres and the other mongodb as their database storage framework which are accessible on port **7443** and port **7444** respectively.

- **NOTE**: **Regards to conse panel actix APIs**, if you want to extend the last table fields first update its `up.sql` file then run ```diesel migration redo``` and finally ```diesel migration run```, to regenerate all tables run ```diesel migration redo -n 3``` which **3** refers to the number of tables we've created so far.

- **NOTE**: **Regards to conse panel actix APIs**, before migrating any table, make sure that you've an already setup database using ```diesel setup && diesel migration run``` command.

- **NOTE**: **Regards to conse panel actix APIs**, use ```diesel migration generate <MIGRAION_NAME>``` to create the migration file containing the postgres table setup, ```diesel migration redo``` to drop the table and ```diesel migration run``` to apply all migration tables to the database after submitting changes to the sql fiels.

- **NOTE**: **Regards to conse mafia hyper APIs**, to update a user access level to `dev` first do a signup for the user using `/auth/signup` API then run the mafia binary server like so: `./mafia dev 0` or `cargo run --bin mafia dev 0` finally login with that user to register a new god for the game.

- **NOTE**: **Regards to conse mafia hyper APIs**, remember to fill the `OTP_API_TOKEN` var inside the `.env` file.

```bash
# 🧪 Test Conse Hyper Server
cargo test --bin mafia
# 🏃 Run Conse Hyper Server
cargo run --bin mafia #---> cargo build --bin mafia --release
# 🏃🏽‍♀️ Run Conse Actix Panel Server
cargo run --bin panel #---> cargo build --bin panel --release
# 🏃🏿 Run Conse Argon2 Test Codes
cargo run --bin argon2test
```
    
## 🚀 Production Setup

> Before going for production, read the following notes: 

- **NOTE**: **Regards to conse panel actix APIs**, two docker instances of panel service will be built, one contains the postgres and the other mongodb as their database storage framework which are accessible on **https://api.panel.conse.app** and **https://api.panel.conse.app/mongo** respectively.

- **NOTE**: **Regards to conse panel actix APIs**, there is a env var called `THIRD_PARY_TWITTER_BOT_ENDPOINT` which can be set to an external twitter bot server endpoint to send requests for user task verification, if you want to use a third party bot remember to pass the endpoint to the instance of the `Twitter` struct like `let bot = Twitter::new(Some(bot_endpoint));`.

- **NOTE**: **Regards to conse panel actix APIs**, currently the `/bot/check-users-tasks` API will be called every day at **7 AM** via a setup crontab inside the `jobs` folder to avoid twitter rate limit issue, if you want to change the cron just run `crontab -e` command inside the `jobs` folder and edit the related cron file.

- **NOTE**: **Regards to conse panel actix APIs**, in order to use twitter APIs you must have a paid developer account and you must use keys and tokens from a twitter developer App that is attached to a project also you can add new keys in `twitter-accounts.json` by calling the `/admin/add-twitter-accounts` API.

- **NOTE**: **Regards to conse panel actix APIs**, to generate a new password for admin and dev users just edit the `argon2test.rs` code inside the `tests` folder then run ```cargo run --bin argon2test``` to generate those passwords finally update the `up.sql` inside the `migrations/2023-05-22-184005_users` folder to insert a new admin and dev user info into the table when you run ```diesel migration run```. 

- **NOTE**: **Regards to conse mafia hyper APIs**, to update a user access level of the conse mafia hyper server to dev, first signup the user using `/auth/signup` API then update the `access_level` field of the user to `0` manually inside the db in `mongodb` container using `portainer` finally login with dev user to register a new god for the game.

- **NOTE**: **Regards to conse mafia hyper APIs**, the default `dev` and `conse` user passwords are `dev@1234%` and `conse@1234` respectively which will be imported to the mongodb automatically by running the `./redeploy.sh` script, also we can run the `./mafia dev 0` binary inside the VPS to update the access level of a user to dev, finally we can register a new god or admin for the mafia game APIs using the dev user token.

- **NOTE**: to access the `mongodb` container shell, login to the `portainer` then fireup the `mongodb` container CMD and run ```mongosh``` or you can go inside using ```sudo docker exec -it mongodb mongosh``` command.

- **NOTE**: after updating application's `Dockerfile` files, we should rebuild our container images by running ```./redeploy.sh``` script again.

- **NOTE**: all docker container the mounted volumes are inside `infra/data` folder.

- **NOTE**: in order to use docker containers inside another one by its DNS name, all of them must be inside the same network bridge like if we want to use the mongodb container inside the panel container they must be in the same network called `gem`. 

- **NOTE**: make sure that you have the `conse.app` domain enabled and is pointing to the machine where the `gem` codes is hosted on.

- **NOTE**: rerun the `renew.sh` on every changes to the nginx config file like hosting new codes, services or adding a new domain to the VPS.

- **NOTE**: for every new (sub)domain inside the VPS there must be a new config file and a new ssl certificate inside the `infra/docker/nginx` folder related to that (sub)domain name.

- **NOTE**: registered (sub)domain records in DNS panel must be:
```bash
conse.app #---> this main domain is related to the home UI of the app
api.mafia.conse.app #---> points to the conse mafia hyper APIs
api.panel.conse.app #----> points to the conse actix APIs
panel.conse.app #---> points to the panel UI
notif.panel.conse.app #---> points to the websocket push notification server APIs
adminer.conse.app #---> points to the adminer UI
jenkins.conse.app #---> points to the jenkins UI
portainer.conse.app #---> points to the portainer UI
```
- **NOTE**: to serve static files using nginx just make sure you copied the `build-{PROJECT-NAME}` folder of JS projects into `infra/docker/nginx/build` folder.   

- **NOTE**: multiple domains can point to a same VPS which their ssl-s and routes can be setup by nginx also multiple (sub)domains of different domains can point to multiple VPS-es which can be setup inside the DNS panel of those domains like the following:

**DNS records of conse.app domain**

```
Type	    Hostname	               Value	          TTL (seconds)	
A	    conse.app              directs to 64.226.71.201	     3600
A	    api.mafia.conse.app   	   directs to 68.183.137.151     3600 
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
sudo chmod +x /root && sudo chown -R www-data:www-data /root && sudo chmod -R 777 /root
sudo gpasswd -a www-data root && sudo chmod g+x /root && sudo -u www-data stat /root
sudo chown -R root:root /root && sudo chmod -R 777 /root
sudo chown -R www-data:www-data . && sudo chmod +x /root
sudo chown -R root:root . && sudo chmod -R 777 . 
sudo chmod +x /root && sudo chmod +x /root/gem && sudo chmod +x /root/gem/infra && sudo chmod +x /root/gem/infra/assets && cd scripts
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

### 🍢 Conse Mafia Mongodb ERD Schema

<p align="center">
    <img src="https://github.com/wildonion/gem/blob/master/infra/conse.schema.PNG">
</p>

### 🖼️ [Conse Panel](https://github.com/wildonion/gem/tree/master/core/panel) Architecture Diagram

<p align="center">
    <img src="https://github.com/wildonion/gem/blob/master/infra/arch.jpg">
</p>

## 🧐 WrapUps 

* there is an API named `/public/get-token-price/{amount}` which returns the value of the passed in tokens in USD, remember that in order to show the exact amount, the value must gets divided by `1000000` to extract the floating point format.

* to see a full list of conse mafia hyper server, import the `gem.http.api.json` into the postman which is inisde the `infra` folder, also for the conse panel actix APIs there is swagger UI which can be loaded through the `https://api.panel.conse.app/swagger/` address to see all available APIs.

> **Regards to conse panel actix APIs**:

* front-end can access the image of an event through the address `https://api.panel.conse.app/assets/images/events/{image_path}` like so: `https://api.panel.conse.app/assets/images/events/event64c93cc7d19645f57fd9f98d-img1692289627686439.jpg`

* we must mount the `assets` directory from the `conse-panel-pg` and `conse-panel-mongo` containers into the host then into the `nginx` container and finally load the assets inside the `api.panel.conse.app.conf` file from `/etc/nginx/assets/` path, by doing the following, any changes made in the `/usr/src/app/assets` directory of the conse panel containers will be reflected in the `$(pwd)/infra/assets/` directory on the host and because this same host directory is also mounted to the nginx container, changes will also be reflected in the `/etc/nginx/assets` directory of the nginx container:
```bash
# mount from conse panel containers into host: 
... -v $(pwd)/infra/assets/:/usr/src/app/assets ...
# mount from host into the nginx container 
... -v $(pwd)/infra/assets/:/etc/nginx/assets ...
```

* based on the last wrapup the `assets` directory can be accessible through the `https://api.panel.conse.app/assets/` address.

* the logic flow of an HTTP API body should be as the following:

> make sure that you're passing the JWT into the request header, defined the request body strucutre and storage (diesel, mongodb or redis) schemas. 

```rust

#[post("/a/sexy/route/{sexy-param}/{another-sexy-param-id}")]
async fn api(
        req: HttpRequest, app_storage: 
        storage: web::Data<Option<Arc<Storage>>>,
        req_body: web::Json<ReqBody>,
        a_sexy_param: web::Path<(String, i32)>
    ) -> PanelHttpResponse{


    /* extracting storage objects (none async redis, redis async, redis pubsub conn, postgres and mongodb) */
    // ...

    /* extracting required roles */
    // ...

    /* matching over extracted storage object */
    // ...
        
        /* passport checking over extracted roles */
        // ...
            
            /* redis rate limit checker */ 
            
                /* api body and code flow responses */ 
                // ...

}
```

* to all gorgeous admins, the `role` field must be **uppercase** and it's default value when it's not passed is **Dev**.

* the generated cookie inside the response of the conse panel admin and user login APIs is in form `<JWT>::<SHA256_OF_LOGIN_TIME>`, we can use the first part of `::` sign, as the **JWT** to send authorized requests in postman and swagger UI. 

* all conse panel APIs except admin and user login, health check and logout APIs need a **JWT** inside the Authorization header or the cookie variable in the their request objects also the **JWT** can be set in their swagger UI page using the `Authorize 🔒` button. 

* in order to reveal the roles of an event, admin **JWT** token generated by the conse mafia hyper server must be passed to the request header of the `/admin/notif/register/reveal-role/{event_objectid}` API inside the panel server, also same as for the dev APIs including `/get/admin/{admin_id}/data` and `/get/user/{user_id}/data`.

* push notification routes for **ECQ**, **MMR** and reveal role topics are `wss://notif.panel.conse.app/subscribe/ecq-{event_objectid}`, `wss://notif.panel.conse.app/subscribe/mmr-{event_objectid}`, `wss://notif.panel.conse.app/subscribe/{user_objectid}/reveal-role-{event_objectid}` respectively and in order to receive realtime role, ecq and mmr notifs data users must use `/join-roles`, `/join-ecq` and `/join-mmr` commands respectively which are used to join the ws channel to create a session once they gets connected, for listening on incoming events (ecq, mmr and reveal role), note that these routes **are guarded with conse mafia JWT** so remember to pass the conse mafia JWT to the header of these routes like `Bearer ...JWT...`, also there is an slash command called `/events` which shows the total online events to the player.

* twitter task names defined by admins, must be prefixed with `twitter-*` and are twitter activities such as `tweet` which can be a specific content or the generated code by the backend, `like`, `hashtag` and `retweet` that must be done to reward users based on the score of each task.

* admins can define multiple twitter tasks in the same activity, all tasks will be separated by a random chars like `*-<RANDOM_CHARS>` so the final task name will be `twitter-username-iYTC^`.

* every day at **7 AM** all the users tasks will be checked automatically using a cronjob to see that the user is still verified or not, this will be done by checking all the records of the `users_tasks` table inside the `/check-users-tasks` API. 

* once the user gets loggedin, first the `/user/verify-twitter-account/{account_name}` API must be called to verify and update the twitter username inside the db then we must compel the user to tweet the activity code which is inside the user data response.
