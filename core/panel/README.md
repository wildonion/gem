

> for a complete list of all APIs refer to the [postman collection](https://github.com/wildonion/gem/blob/master/infra/gem.http.api.json)

## 🔐 API Access

### 👤 User Access
- push notif subscriptions (mmr ranking and reveal role) `<---rendezvous jwt--->` rendezvous hyper server
- twitter account, otp, mail, identity and bank accounts verification process
- new login and check token flow based on cookie time hash
- building secp256k1 crypto wallet
- twitter tasks report
- update bio, wallet background, avatar and banner
- transferring in-app tokens by minting (`deposit`) and transfering (`withdraw`) nft 
- get related deposits and withdrawals
- chatroom launchpad calls to chat and mint nft 
- user gallery based markeplace calls
    - public and private galleries
    - fetch private and public gallery nfts and collections
    - add comment on and like the nft
    - polygon trade nft calls
    - add/remove/accept friend requests
    - add/remove/accept gallery friend requests
    - create nfts and upload their artworkds on ipfs network storage
    - mint, buy and transfer nfts on polygon network in the platform
- buy and sell in-app token (charge wallet and stripe apis)
- get token price and gas fee
- register, get and join latest chatroom launchpad event

### 👑 Admin Access
- publish reveal role topic of an event `<---rendezvous jwt--->` rendezvous hyper server
- update event image `<---rendezvous jwt--->` rendezvous hyper server
- add twitter account for the twitter bot
- register/delete/edit new twitter tasks and users
- get all withdrawals, deposits, checkouts, users, users tasks and twitter tasks
- create a new chatroom launchpad (generate `N` number of nfts using AI for future mintings)
- get a chatroom launchpad statistics info 
- advieh process for a collection
- chatroom launchpad endpoints (create, edit and fetch)

### 👨🏻‍💻 Dev Access
- get all data of a user `<---rendezvous jwt--->` rendezvous hyper server
- get all data of an admin `<---rendezvous jwt--->` rendezvous hyper server

### 🌎 Public Access
- user twitter task verification using twitter bot
- check user twitter task 
- tasks leaderboard
- search
- get ratelimit info of xbot
- get user wallet info
- top and all nfts

### 🥞 Health Routes
- check server status
- check token 
- logout
- is user kyced
- get all the tasks
- stripe update user balance webhook
- forgot password

## 🔑 Tiny KYC Identity Verification Process Before Creating CID

> in each verification process 5 token will be given to the user.

- first of all the `/user/login` API must be called to register a new user.
- the second step would be calling the `/user/request-mail-code/{mail}`, `/user/verify-mail-code`, `/user/request-phone-code/{phone}`, `/user/verify-phone-code` APIs to verify the user mail and phone number in order to create the **Crypto Id**.
- then the `/user/cid/build` API must be called to upsert the `username` and `region` fields, it'll create a new **Crypto Id** with the passed in `username` and `device_id`, on the first call and update `username` and `region` field (based on the location of the requested IP address) only on the second call.

## 🧬 Deposit and Withdrawal Process

- mail verification 
- phone verification 
- crypto id (username)
- charge wallet for in-app transactions
- depositor can call `deposit` API to mint nft by spending in-app tokens from his wallet
- withdrawer can call `withdraw` API to transfer nft to recipient to update his in-app token balance

## 🎢 Development Wrapups

* to regenerate the ERD from the postgres database, **from the root of the project**, just run ```sqlant postgresql://postgres:<PASSWORD>@localhost/conse > infra/panel.uml && java -jar infra/plantuml.jar infra/panel.uml```.

* two docker instances of panel service will be built, one contains the postgres and the other mongodb as their database storage framework which are accessible on port **7443** and port **7444** respectively.

* if you want to extend the last table fields first update its `up.sql` file then run ```diesel migration redo``` and finally ```diesel migration run```, to regenerate all tables run ```diesel migration redo -n 3``` which **3** refers to the number of tables we've created so far.

* before migrating any table, make sure that you've an already setup database using ```diesel setup && diesel migration run``` command.

* use ```diesel migration generate <MIGRAION_NAME>``` to create the migration file containing the postgres table setup, ```diesel migration redo``` to drop the table and ```diesel migration run``` to apply all migration tables to the database after submitting changes to the sql fiels.

* to generate a new password for admin and dev users just edit the `tests.rs` code inside the `test` folder then run ```cargo run --bin contest``` to generate those passwords finally update the `up.sql` inside the `migrations/2023-05-22-184005_users` folder to insert a new admin and dev user info into the table when you run ```diesel migration run```.

* two docker instances of panel service will be built, one contains the postgres and the other mongodb as their database storage framework which are accessible on **https://api.panel.conse.app** and **https://api.panel.conse.app/mongo** respectively.

* currently the `/bot/check-users-tasks` API will be called every day at **7 AM** via a setup crontab inside the `jobs` folder to avoid twitter rate limit issue, if you want to change the cron just run `crontab -e` command inside the `jobs` folder and edit the related cron file.

* in order to use twitter APIs you must have a paid developer account and you must use keys and tokens from a twitter developer App that is attached to a project also you can add new keys in `twitter-accounts.json` by calling the `/admin/add-twitter-accounts` API.

* make sure that we're using live stripe keys in `.env` file and we have `https://conse.app/stripe/checkout/success` and `https://conse.app/stripe/checkout/cancel` pages in front-end in order to redirect user to the related page either on a successful stripe checkout payment process or a cancel button event in checkout page, for more see [this](https://github.com/wildonion/gem/tree/master/core/stripewh) README.

* there is an API named `/user/get-token-price/{amount}` which returns the value of the passed in tokens in USD, remember that in order to show the exact amount, the value must gets divided by `10000000` to extract the floating point format.

* there is an API named `/user/get-gas-fee` which returns the current onchain gas fee based on the number of in-app token.

* every day at **7 AM** all the users tasks will be checked automatically using a cronjob to see that the user is still verified or not, this will be done by checking all the records of the `users_tasks` table inside the `/check-users-tasks` API. 

* there are access and refresh tokens in cookie response in form of `/accesstoken={access_token:}&accesstoken_time={time_hash_hex_string:}&refreshtoken={refresh_token:}` once the access token gets expired we can pass refresh token into the request header in place of access token to get a new set of keys on behalf of user, instead of redirecting client to the login page again.

* in order to reveal the roles of an event, admin **JWT** token generated by the conse rendezvous hyper server must be passed to the request header of the `/admin/notif/register/reveal-role/{event_objectid}` API inside the panel server, also same as for the dev APIs including `/get/admin/{admin_id}/data` and `/get/user/{user_id}/data`.

* push notification routes for **ECQ**, **MMR** and reveal role topics are `wss://event.panel.conse.app/subscribe/mmr-{event_objectid}`, `wss://event.panel.conse.app/subscribe/{user_objectid}/reveal-role-{event_objectid}` respectively and in order to receive realtime role and mmr notifs data users must use `/join-roles` and `/join-mmr` commands respectively which are used to join the ws channel to create a session once they gets connected, for listening on incoming events (mmr and reveal role), note that these routes **are guarded with conse rendezvous JWT** so remember to pass the conse rendezvous JWT to the header of these routes like `Bearer ...JWT...`, also there is an slash command called `/events` which shows the total online events to the player.

* chatroom launchpad endpoint is `wss://event.panel.conse.app/subscribe/chatroomlp/{chatroomlp_id}/{user_screen_cid}/{tx_signature}/{hash_data}/?r1pubkey={r1pubkey}&r1signature={r1signature}` and there must be JWT in header like: `Authorization`: `Bearer ...JWT...`.

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

#[derive(Clone, Debug, Serialize, Deserialize)]
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