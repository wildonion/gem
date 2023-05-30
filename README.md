


# 🤏 Conse

Conse is an AI based Crypto Game Event Manager Platform on top of [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) and Solana blockchain which uses: 
- an AI model like [STEM](https://github.com/wildonion/stem) which will suggests players the tips and tricks for a new game based on behavioural graph of the each player collected by the history of each event's `phases` field inside the game.
- [uniXerr](https://github.com/wildonion/uniXerr) coin generation AI model which players get rewarded based on their scores and positions, collected by each event manager inside the game, then update the balance field of the user based on those attributes.
- order matching engine to suggests players events and other games based on their past experiences, scores (MMR) and earned tokens also players can sell their minted roles based on highest or lowest order setup in conse order book.

<p align="center">
    <img width=350 height=350 src="https://github.com/wildonion/gem/blob/master/assets/conse.png"
</p>

```
🌍 MAIN SITE ==> https://conse.app/
👨🏻‍⚖️ ADMIN PANEL ==> https://admin.conse.app/
🛤️ ADMIN/DEV API ROUTE ==> https://api.panel.conse.app/
🗺️ MAIN API ROUTE ==> https://api.conse.app/
🎙️ HOSTED ON ==> ...
```

## 🛠️ Development Setup

> Note that to update a user access level to `dev` first do a signup for the user using `/auth/signup` API then run the binary server like so: `./cosne wildonion 0` or `cargo run --bin conse wildonion 0` finally login with that user to register a new god for the game.

```bash
# 🧪 Test Conse Hyper Server
cargo test --bin conse
# 🏃 Run Conse Hyper Server
cargo run --bin conse
# 🏃🏽‍♀️ Run Conse Actix Panel Server
cargo run --bin panel
# 🏃🏻‍♀️ Run Conse Discord Bot Server
cargo run --bin dis-bot
# 🏃🏿 Run Conse Test Codes
cargo run --bin tests
# 🏃🏿 Run Conse Argon2 Test Codes
cargo run --bin argon2test
```
    
## 🚀 Production Setup

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
./deploy.sh
# ---------------
# ---- renew nginx 
./renew.sh
```
- **NOTE**: Make sure that you have a domain up and running that is pointing to the machine where the `gem` is hosted on.

- **NOTE**: Rerun the `renew.sh` on every changes to the nginx config file like hosting new codes, services or adding a new domain to the VPS.

- **NOTE**: For every new (sub)domain inside the VPS there must be a new config file and a new ssl certificate inside the `infra/docker/nginx` folder related to that (sub)domain name.

- **NOTE**: There must be three registered (sub)domains in DNS panel of `conse.app`: `api.conse.app`, `api.panel.conse.app`, `panel.conse.app` which points to the conse hyper APIs, Actix APIs and the panel UI respectively.

- **NOTE**: To serve static files using nginx just make sure you copied the `build-{PROJECT-NAME}` folder of JS projects into `infra/docker/nginx/build` folder.   

- **NOTE**: Multiple domains can point to a same VPS which can be setup by nginx also multiple (sub)domains of different domains can point to multiple VPS-es which can be setup inside the DNS panel of those domains like the following:

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

## 🗃️ Directory Explained

* `core`: all in-game APIs which are related to the player app.
    * `contexts`: 
        * `bot`: Discord and Twitter bots .
        * `panel`: Dev and Admin panel app written in Yew.
        * `blockchain`: Solana Anchor smart contracts.
    * `controllers`: in-game API async controllers.
    * `routers`: in-game API routers.
    * `schemas`: in-game mongodb schemas.
* `infra`: all infrastructure configs and setup files.

> **NOTE**: All `conse`, `panel` and `bot` are just different binaries and sharing a same `Cargo.toml` setup.

## 🍟 Good 2 Know

* to access the `mongodb` container shell, login to the `portrainer` then fireup the `mongodb` container CMD and run ```mongosh``` or you can go inside using ```sudo docker exec -it mongodb mongosh``` command.

* remember to fill the `OPENAI_KEY` and `DISCORD_TOKEN` vars inside the `.env` with appropriate values using ```echo OPENAI_KEY=<TOKEN> >> .env``` and ```echo DISCORD_TOKEN=<TOKNE> >> .env``` commands.

* after updating application's `Dockerfile` files, we should rebuild our container images by running ```./deploy.sh``` script again.

* to update a user access level to dev, first signup the user using `/auth/signup` API then update the `access_level` field of the user to `0` manually inside the db in `mongodb` container using `portrainer` finally login with dev user to register a new god for the game.

* all docker container the mounted volumes are inside `infra/data` folder. 

* before migrating any table, make sure that you've an already setup database using ```diesel setup && diesel migration run``` command.

* use ```diesel migration generate <MIGRAION_NAME>``` to create the migration file containing the postgres table setup, ```diesel migration redo``` to drop the table and ```diesel migration run``` to apply all migration tables to the database after submitting changes to the sql fiels.

* in order to use docker containers inside another one by its DNS name, all of them must be inside the same network bridge like if we want to use the mongodb container inside the gem container they must be in the same network called `gem`. 

* clean docker cache using ```sudo docker buildx prune --all``` or ```docker system prune --all``` command.

* conse client can subscribes to the fired or emitted role reveal event and topics by sending websocket connections to the redis server docker on the VPS in the meanwhile we're sure that the `/reveal/roles` API has been called by the dev or the god inside the panel thus players can see their roles without refreshing the page.
