

<img src="https://github.com/wildonion/gem/blob/master/assets/conse.png" width="150"/>

is a crypto based friendly gathering **Game Event Manager**, NFT based galleries on top of **Polygon** with the following design patterns as its core engine: 
- actor based component APIs, a reliable design pattern facilitating the communication between each API without sending HTTP requests, only by utilising actor message sending concept.
- actor based pubsub push notification strategies to notify different parts of the app clearly the user, on every db updates using redis pubsub pattern ([more](https://github.com/wildonion/gem/wiki/Realtime-Push-Notification-Strategy)).
- behavioural graph virtual machine (**[GVM](https://github.com/wildonion/gvm/)**) by utilising top users to rank and match them as well as suggest users to each other based on their stars.
- empowering caching strategies to speed up performance in calling heavy computational APIs.
- **SSE** technology to inform clients by receiving automatic updates from the server via an HTTP connection. 

### 🍔 Features

- an event managing and reservation system with reveal-participant-role functionality

- **X (twitter)** task verification system based on pre-defined tasks by admins.

- broadcasting **X** tasks registered by admins into discord channel.

- crypto id, an **EVM** based wallet with **secp256k1** algorithm to store users' onchain digital assets and sign in-app API calls.

- the most secure logic for authentication process with **JWT** per device and **secp256k1** per each user for API signing.

- buying, receiving and selling in-app **$TOKEN** using **Stripe** payment gateway and minting **NFT** gift cards.

- creating multiple private galleries to create, mint and transfer **NFTs** in various art work collections by inviting friends.

- participating in chatroom launchpad events to mint **AI** generated **NFTs**, based on summarization of participants' chats in each event.

## 🥪 Panel Architecture

<p align="center">
    <img src="https://github.com/wildonion/gem/blob/master/infra/arch.jpg">
</p>


## 🚟 Infra Route and API Endpoints

> Ensure that all the vars inside `.env.prod` are filled with their appropriate values.

> Ensure that any self-signed SSL certificates used by gRPC server are valid and issued by a recognized certificate authority, if you are using a self-signed certificate, it may not be trusted by default by clients.

```bash
# ---------------------------------------------------------------------------------------------------
# conse panel dev username/password              : devdevy/d3v@%$^$3hjsD
# conse panel admin username/password            : adminy/4dmin@%$^$3hjsD
# postgres adminer username/password/server      : postgres/geDteDd0Ltg2135FJYQ6rjNYHYkGQa70/postgres
# ---------------------------------------------------------------------------------------------------
🎙️ HOSTED ON                                    ==> Digitalocean
🥛 WEBSOCKET PUSH NOTIFICATION ROUTE            ==> wss://event.panel.conse.app/subscribe/
🥛 WEBSOCKET HHTP PUSH NOTIFICATION ROUTE       ==> https://event.panel.conse.app/subscribe/
🌍 MAIN SITE                                    ==> https://conse.app/
👨🏻‍⚖️ ADMIN PANEL                                  ==> https://panel.conse.app/
🛤️ ADMIN/DEV API ROUTE WITH POSTGRES DB STORAGE ==> https://api.panel.conse.app/
🛣 ADMIN/DEV API ROUTE WITH MONGO DB STORAGE    ==> https://api.panel.conse.app/mongo
🗺️ RENDEZVOUS API ROUTE                         ==> https://api.rendezvous.conse.app/
📡 SWAGGER DOC                                  ==> https://api.panel.conse.app/swagger/
🛢️ ADMINER PANEL                                ==> https://adminer.conse.app
🛎️ JENKINS PANEL                                ==> https://jenkins.conse.app
⛵ PORTAINER PANEL                              ==> https://portainer.conse.app
📊 GRAFANA PANEL                                ==> https://grafana.conse.app
🏦 STRIPE WEBHOOK ENDPOINT                      ==> https://api.panel.stripewh.conse.app
🤖 X BOT                                        ==> https://api.xbot.conse.app
🗞️ PANEL AND XCORD ERROR LOGS                   ==> https://api.panel.conse.app/logs
🗂️ PANEL ASSETS FOLDER                          ==> https://api.panel.conse.app/assets
🧙‍♂️ KYC gRPC SERVER                              ==> grpcs://rpc.conse.app/kyc.KycService/Verify

# Push Notification WS Routes
wss://event.panel.conse.app/subscribe/64b827fad916781c6d68948a/reveal-role-64b82757d916781c6d689488
wss://event.panel.conse.app/subscribe/64b827fad916781c6d68948a/mmr-64b82757d916781c6d689488

# Chatroom Launchpad WS Route
wss://event.panel.conse.app/subscribe/chatroomlp/1/03fe4d2c2eb9ab44971e01d9cd928b4707a9d014381d75ec19f946b78a28164cc6/8ef4637573c6ef6170c817ad22fc4e45de4eae1b86fbe26f19986d49e9c4e24a3fe7d5f6fef58b2ae6a160ca058c41c401401ecc509f8afffe30035e0ad7451f1c/b051b639719983d5062cb8bdb5f57afffb4a634c8c8a6b9e957f583ee1087ea1/?r1pubkey=0x554543320000002d6682f8f7030f89be91e75b5604e14c026d7ec893c4be6de1d221a9e329a59b8dee2fad3b16&r1signature=0x20260426e5000000470000007b22726563697069656e745f636964223a223078353534353433333230303030303032643636383266386637303330663839626539316537356235363034653134633032366437656338393363346265366465316432323161396533323961353962386465653266616433623136222c2266726f6d5f636964223a223078353534353433333230303030303032643636383266386637303330663839626539316537356235363034653134633032366437656338393363346265366465316432323161396533323961353962386465653266616433623136222c22616d6f756e74223a357d3045022100d49e8716ef150129b612c65ef8e798e8fac73577fc8df1d4664674488b89f86d02203f62c3c5776ed393a4d0a761714d9f1e52185c5b24c4a3afe03b7903aa5186af
```

### 🗃️ Directory and Structure Explained

* `core`: hyper, tonic gRPC, actix web HTTP and actix WS servers.
    * `chatdb`: spacetimedb wasm methods used in chatroom launchpad section in conse panel.
    * `stripewh`: stripe webhook listener for checkout events.
    * `xbot`: X bot for twitter tasks verification.
    * `xcord`: discord bot to broadcast new twitter task defined by admin into a discord channel.
    * `mailreq`: mail sender crate.
    * `multipartreq`: multipart extractor crate.
    * `phonereq`: OTP code sender crate.
    * `s3req`: shared state storage crate.
    * `walletreq`: wallexerr crate.
    * `gastracker`: gastracker crate.
    * `grpc`: KYC grpc server.
    * `panel`: user, dev and admin dashboard panel health, public and secured APIs with actix web and actix WS server.
    * `rendezvous`: rendezvous service upsert event and deck, reserve event, reveal role and other in-game APIs.
        * `controllers`: in-game async controllers related to hyper server.
        * `routers`: in-game API routers related to hyper server.
        * `schemas`: in-game mongodb schemas related to hyper server.
* `infra`: all infrastructure and devops configs.
* `jobs`: gem crontab jobs
* `logs`: gem log files generated by conse panel and other parts of the app.
* `migrations`: diesel postgres sql files
* `scripts`: deployment scripts
* `test`: gem test codes like admin and dev password generator script

## 📘 Docs, ERDs, Schemas and Collections

* Read More About [Crypto ID](https://github.com/wildonion/gem/wiki/Crypto-ID)

* Read More About [Push Notification Strategy](https://github.com/wildonion/gem/wiki/Realtime-Push-Notification-Strategy)

* [HTTP Postman Collection](https://dewoloper.postman.co/workspace/dewo~9f34982c-dde5-4f77-9d5d-46872ed07d4a/collection/22927035-7a3bd80c-b40f-46ab-bc94-5fca466fe30b?action=share&creator=22927035)

* [Websocket Postman Collection](https://dewoloper.postman.co/workspace/dewo~9f34982c-dde5-4f77-9d5d-46872ed07d4a/collection/65619b4947e9207e30af90fa?action=share&creator=22927035)

* [gRPC Postman Collection](https://dewoloper.postman.co/workspace/dewo~9f34982c-dde5-4f77-9d5d-46872ed07d4a/collection/65619a9b26e3b575756a3ee5?action=share&creator=22927035)

* [Conse Rendezvous Mongodb ERD Schema](https://github.com/wildonion/gem/blob/master/infra/conse.schema.PNG)

* [Conse Panel Postgres ERD Schema](https://github.com/wildonion/gem/blob/master/infra/panel.png)

## 🛠️ Development Setup

> you can download runtime crashing error logs throught the address `https://api.panel.conse.app/logs` also after setting up the `portainer`, each container logs can be downloaded inside the panel, also makre sure that you've installed the following packages on **MacOS M1**:

```bash
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
grafana.conse.app            #---> points to the grafana UI
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
