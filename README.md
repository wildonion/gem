

<img src="https://github.com/wildonion/gem/blob/master/assets/conse.png" width="150"/>

# 🛞 Rust Both Core Monolithic and Micro-service Backend APIs and Engines

Conse is a crypto based friendly gathering **Game Event Manager**, advertising platform (**advieh**), gallery based NFT Marketplace on top of **Polygon** which uses **Sense**: A Near-Duplicate NFT Detection Protocol on the Pastel Network, with the following engines as its core backend: 
- chatroom launchpad feature to mint generated AI based NFT images to users based on their chats in each launchpad events.
- **pubsub** pattern to reveal player in-game roles inside the rendezvous service using redis publisher and subscriber and websocket server to notify players of new roles once the server subscribed to the revealed roles topic.
- event collaboration queue (**ECQ**) system in which admins or game managers can share their registered events and collaborate with other admins.
- behavioural graph virtual machine (**[GVM](https://github.com/wildonion/gvm/)**) built on top of each event's `phases` field inside the game for each player to match them for new game and rank them based on their in-game statuses, the match making rating or ranking (**MMR**) engine, on the other hand is is a weighted tree based suggestion engine that suggests players, events and other games and players based on their ranks earned using **GVM** during the game.

## 🚟 Infra Routes and APIs

> Ensure that any self-signed SSL certificates used by gRPC server are valid and issued by a recognized certificate authority, if you are using a self-signed certificate, it may not be trusted by default by clients.

```bash
# ---------------------------------------------------------------------------------------------------
# conse panel dev username/password              : devdevy/d3v@%$^$3hjsD
# conse panel admin username/password            : adminy/4dmin@%$^$3hjsD
# postgres adminer username/password/server      : postgres/geDteDd0Ltg2135FJYQ6rjNYHYkGQa70/postgres
# ---------------------------------------------------------------------------------------------------
🎙️ HOSTED ON                                    ==> Digitalocean
🥛 WEBSOCKET PUSH NOTIFICATION ROUTE            ==> wss://event.panel.conse.app/subscribe/
🌍 MAIN SITE                                    ==> https://conse.app/
👨🏻‍⚖️ ADMIN PANEL                                  ==> https://panel.conse.app/
🛤️ ADMIN/DEV API ROUTE WITH POSTGRES DB STORAGE ==> https://api.panel.conse.app/
🛣 ADMIN/DEV API ROUTE WITH MONGO DB STORAGE    ==> https://api.panel.conse.app/mongo
🗺️ RENDEZVOUS API ROUTE                         ==> https://api.rendezvous.conse.app/
📡 SWAGGER DOC                                  ==> https://api.panel.conse.app/swagger/
🛢️ ADMINER PANEL                                ==> https://adminer.conse.app
🛎️ JENKINS PANEL                                ==> https://jenkins.conse.app
⛵ PORTAINER PANEL                              ==> https://portainer.conse.app
🏦 STRIPE WEBHOOK ENDPOINT                      ==> https://api.panel.stripewh.conse.app
🤖 X BOT                                        ==> https://api.xbot.conse.app
🗞️ PANEL AND XCORD ERROR LOGS                   ==> https://api.panel.conse.app/logs
🗂️ PANEL ASSETS FOLDER                          ==> https://api.panel.conse.app/assets
🧙‍♂️ KYC gRPC SERVER                              ==> grpcs://rpc.conse.app/kyc.KycService/Verify

# Push Notification WS Routes
`wss://event.panel.conse.app/subscribe/64b827fad916781c6d68948a/reveal-role-64b82757d916781c6d689488`
`wss://event.panel.conse.app/subscribe/64b827fad916781c6d68948a/mmr-64b82757d916781c6d689488`
`wss://event.panel.conse.app/subscribe/64b827fad916781c6d68948a/ecq-64b82757d916781c6d689488`

# Chatroom Launchpad WS Route
`wss://event.panel.conse.app/subscribe/chatroomlp/1/03fe4d2c2eb9ab44971e01d9cd928b4707a9d014381d75ec19f946b78a28164cc6/8ef4637573c6ef6170c817ad22fc4e45de4eae1b86fbe26f19986d49e9c4e24a3fe7d5f6fef58b2ae6a160ca058c41c401401ecc509f8afffe30035e0ad7451f1c/b051b639719983d5062cb8bdb5f57afffb4a634c8c8a6b9e957f583ee1087ea1/?r1pubkey=0x554543320000002d6682f8f7030f89be91e75b5604e14c026d7ec893c4be6de1d221a9e329a59b8dee2fad3b16&r1signature=0x20260426e5000000470000007b22726563697069656e745f636964223a223078353534353433333230303030303032643636383266386637303330663839626539316537356235363034653134633032366437656338393363346265366465316432323161396533323961353962386465653266616433623136222c2266726f6d5f636964223a223078353534353433333230303030303032643636383266386637303330663839626539316537356235363034653134633032366437656338393363346265366465316432323161396533323961353962386465653266616433623136222c22616d6f756e74223a357d3045022100d49e8716ef150129b612c65ef8e798e8fac73577fc8df1d4664674488b89f86d02203f62c3c5776ed393a4d0a761714d9f1e52185c5b24c4a3afe03b7903aa5186af`
```

### 🗃️ Directory and Structure Explained

* `core`: hyper, tonic gRPC, actix web HTTP and actix WS servers.
    * `stripewh`: stripe webhook listener for checkout events.
    * `xbot`: X bot for twitter tasks verification.
    * `xcord`: discord bot to broadcast new twitter task defined by admin into a discord channel and role assginement based on user points.
    * `mailreq`: mail sender crate.
    * `multipartreq`: Multipart extractor.
    * `phonereq`: OTP code sender crate.
    * `s3req`: shared state storage crate.
    * `walletreq`: wallexerr crate.
    * `gastracker`: gastracker crate.
    * `grpc`: KYC grpc server.
    * `panel`: user, dev and admin dashboard panel APIs with actix web and actix WS server.
    * `rendezvous`: rendezvous service APIs
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

## 📘 Docs and Collections

* [Engines](https://github.com/wildonion/gem/wiki/Engines)

* [Crypto ID](https://github.com/wildonion/gem/wiki/Crypto-ID)

* [Reveal Role](https://github.com/wildonion/gem/wiki/Reveal-Role)

* [TLPS](https://github.com/wildonion/gem/wiki/TLPs)

* [HTTP Postman Collection](https://dewoloper.postman.co/workspace/dewo~9f34982c-dde5-4f77-9d5d-46872ed07d4a/collection/22927035-7a3bd80c-b40f-46ab-bc94-5fca466fe30b?action=share&creator=22927035)

* [Websocket Postman Collection](https://dewoloper.postman.co/workspace/dewo~9f34982c-dde5-4f77-9d5d-46872ed07d4a/collection/65619b4947e9207e30af90fa?action=share&creator=22927035)

* [gRPC Postman Collection](https://dewoloper.postman.co/workspace/dewo~9f34982c-dde5-4f77-9d5d-46872ed07d4a/collection/65619a9b26e3b575756a3ee5?action=share&creator=22927035)

## 🛠️ Development Setup

> you can download runtime crashing error logs throught the address `https://api.panel.conse.app/logs` also after setting up the `portainer`, each container logs can be downloaded inside the panel, also makre sure that you've installed the following packages on **MacOS M1**:

```bash
wget https://download.pastel.network/latest-release/pastelup/pastelup-linux-amd64
chmod 755 pastelup-linux-amd64
./pastelup install walletnode -n=testnet
./pastelup start walletnode --development-mode
cd pastel && ./pastel-cli getconnectioncount
./pastel-cli getnewaddress
./pastel-cli dumpprivkey <address you just printed>
./pastel-cli importprivkey <private key you just printed>
./pastel-cli pastelid newkey "<passphrase>"
./pastel-cli tickets register id personal <pastelid> <passphrase> <address>
brew tap cossacklabs/tap
brew install openjdk
brew install pkg-config
brew install openssl
brew install diesel
brew link --force openssl
brew install libpq && brew link --force libpq
brew install protobuf
brew install graphviz
brew tap cossacklabs/tap && brew install libthemis
cargo install spacetimedb-cli
sudo npm i wasm-opt -g
cargo clean
```
then run:

```bash
# 🥒 build proto files
cargo build
# 🏃 Run Conse Hyper Server
cargo run --bin rendezvous #---> cargo build --bin rendezvous --release
# 🏃🏽‍♀️ Run Conse Actix Panel Server
cargo run --bin panel #---> cargo build --bin panel --release
# 🏃🏽‍♀️ Run Conse KYC gRPC Server
cargo run --bin grpc #---> cargo build --bin grpc --release
# 🏃🏿 Run Conse Test Codes
cargo run --bin contest
```
    
## 🚀 Production Setup

> before going for production, make make sure that you have the `conse.app` domain and the following subdomains are enabled in DNS panel and is pointing to the machine where the `gem` services codes are hosted on, note that for every new (sub)domain inside the VPS there must be a new nginx config file and a new ssl certificate inside the `infra/docker/nginx` folder related to that (sub)domain name which can be setup by running `renew.sh` on every changes to the nginx config file like hosting new codes, services or adding a new domain to the VPS.

```bash
conse.app                    #---> this main domain is related to the home UI of the app
api.rendezvous.conse.app     #---> points to the conse rendezvous hyper APIs
api.panel.conse.app          #---> points to the conse actix APIs
panel.conse.app              #---> points to the panel UI
event.panel.conse.app        #---> points to the websocket chat and push notification server APIs
adminer.conse.app            #---> points to the adminer UI
jenkins.conse.app            #---> points to the jenkins UI
portainer.conse.app          #---> points to the portainer UI
api.panel.stripewh.conse.app #---> stripe webhook endpoint to receive checkout events
api.xbot.conse.app           #---> twitter bot to verify twitter tasks 
rpc.conse.app                #---> gRPC server actors
```

> keep in mind that multiple domains can point to a same VPS which their ssl-s and routes can be setup by nginx also multiple (sub)domains of different domains can point to multiple VPS-es which can be setup inside the DNS panel of those domains like the following:

**DNS records of conse.app domain**

```
Type	    Hostname	               Value	                TTL (seconds)	
A	    conse.app                   directs to 64.100.71.201	  3600
A	    api.rendezvous.conse.app   	directs to 68.200.137.151     3600 
A	    panel.conse.app    	        directs to 68.200.201.134     3600 
```
**DNS records of wildonion.io domain**

```
Type	    Hostname	               Value	          TTL (seconds)	
A	    wildonion.io           directs to 64.100.71.201	     3600
A	    api.wildonion.app      directs to 68.200.137.154     3600 
A	    admin.wildonion.app    directs to 68.200.201.129     3600 
```
in the above records `wildonion.io` and `conse.app` are pointing to a same VPS but their (sub)domains are pointing to different VPS-es.

finally run the followings: 

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
# ---------------
# ---- rebuild conse panel docker container only 
./rebuildpanel.sh
```
    
## 🪟 Schemas and ERDs

> Note that to regenerate the ERD from the postgres database just run ```sqlant postgresql://postgres:<PASSWORD>@localhost/conse > infra/panel.uml && java -jar infra/plantuml.jar infra/panel.uml```.

### 🥪 Conse Panel Postgres ERD Schema

<p align="center">
    <img src="https://github.com/wildonion/gem/blob/master/infra/panel.png">
</p>

### 🍢 Conse Rendezvous Mongodb ERD Schema

<p align="center">
    <img src="https://github.com/wildonion/gem/blob/master/infra/conse.schema.PNG">
</p>

### 🖼️ [Conse Panel](https://github.com/wildonion/gem/tree/master/core/panel) Push Notif Architecture Diagram

<p align="center">
    <img src="https://github.com/wildonion/gem/blob/master/infra/arch.jpg">
</p>

## 🧐 WrapUps 

* basically if you want to execute an sql file into a database you can run the following commands:
    - step 1: ```bash docker cp run.sql postgres:run.sql```
    - step 2: ```bash docker exec -it postgres psql -U postgres -d conse -f /run.sql```

* front-end can access the image of an event through the address `https://api.panel.conse.app/assets/images/events/{image_path}` like so: `https://api.panel.conse.app/assets/images/events/event64c93cc7d19645f57fd9f98d-img1692289627686439.jpg` and banner of a user through the address `https://api.panel.conse.app/assets/images/avatars/{image_path}` like so: `https://api.panel.conse.app/assets/images/avatars/avatar12-img1692289627686439.jpg` and `https://api.panel.conse.app/assets/images/banners/banner12-img1692289627686439.jpg`

* we must mount the `assets` directory from the `conse-panel-pg` and `conse-panel-mongo` containers into the host then into the `nginx` container and finally load the assets inside the `api.panel.conse.app.conf` file from `/etc/nginx/assets/` path, by doing the following, any changes made in the `/app/assets` directory of the conse panel containers will be reflected in the `$(pwd)/assets/` directory on the host and because this same host directory is also mounted to the nginx container, changes will also be reflected in the `/etc/nginx/assets` directory of the nginx container:
```bash
# mount from conse panel containers into host: 
... -v $(pwd)/assets/:/app/assets ...
# mount from host into the nginx container 
... -v $(pwd)/assets/:/etc/nginx/assets ...
```

* based on the last wrapup the `assets` directory can be accessible through the `https://api.panel.conse.app/assets/` address.

* the logic flow of an HTTP API body should be the following:

> make sure that you're passing the JWT into the request header, you've defined the request body strucutre and storage (diesel, mongodb or redis) schemas. 

```rust

//----> route: /a/sexy/route/wildonion/0x31A72ae35138A34BB1c3522d2aC8FFaC1a37EA8D/12/?from=0&to=10

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Limit{
    pub from: Option<i64>,
    pub to: Option<i64>
}

async fn api()
#[post("/a/sexy/route/{sexy-param}/{another-sexy-param-id}/?from=0&to=10")]
async fn api(
        req: HttpRequest,  
        app_storage: web::Data<Option<Arc<Storage>>>,
        req_body: web::Json<ReqBody>,
        limit: web::Path<Limit>,
        a_sexy_param: web::Path<(String, i32)>
        stream: web::Payload,
        payload: Multipart,
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
            // ...

                /* kyc verification process */
                // ...
                    
                    /* api body and code flow responses */ 
                    // ...

}
```