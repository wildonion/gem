


## 🎛️ conse user, dev and admin dashboard panel APIs based on their access are:

### 👤 User Access
- push notif subscriptions (mmr ranking, ecq and reveal role) `<---mafia jwt--->` mafia hyper server
- twitter account, otp, mail, identity and bank accounts verification process
- new login and check token flow
- building crypto wallet
- do and get related twitter tasks
- transferring in-app tokens by minting (`deposit`) and burning (`withdraw`) nft 
- get related deposits and withdrawals
- add comment on and like the post
- user gallery calls
    - mint, sell (listing), offer, auction, 
    - public and private room collections, 
    - advertising collection
    - add comment on and like the nft
    - add/remove friend
- buy in-app token (charge wallet)

### 👑 Admin Access
- advertise event `<---mafia jwt--->` mafia hyper server
- publish reveal role topic of an event `<---mafia jwt--->` mafia hyper server
- publish ecq topic of an event `<---mafia jwt--->` mafia hyper server
- update event image `<---mafia jwt--->` mafia hyper server
- add twitter account for the twitter bot
- register/delete/edit new twitter tasks, posts and users
- get all withdrawals, deposits, users and twitter tasks
- verify and get all posts and their comments and likes 

### 👨🏻‍💻 Dev Access
- get all data of a user `<---mafia jwt--->` mafia hyper server
- get all data of an admin `<---mafia jwt--->` mafia hyper server

### 🌎 Public Access
- user twitter task verification using twitter bot
- check user twitter task 
- get posts
- get token price
- gallery public calls
    - get collections
    - get main room nfts of collection

### 🥞 Health Routes
- check server status
- check token 
- logout
- get all the tasks

## 🔑 Tiny KYC Identity Verification Process

- first of all the `/user/login` API must be called to register a new user.
- second of all the `/user/request-mail-code/{mail}` and `/user/verify-mail-code` APIs must be called to verify the user mail in order to create the **Crypto Id**.
- then the `/user/cid/build` API must be called to upsert the fields, it'll create a new **Crypto Id** with the passed in `username` and `device_id`, on the first call and update `username` field only on the second call.
- finally we can call the `/user/request-phone-code/{phone}` and `/user/verify-phone-code` APIs to verify the user phone number which will send the **OTP** code from the IR or none-IR **OTP** provider based on the updated user region in previous step.

## 🧬 Deposit and Withdrawal Process

- mail verification 
- crypto id (username)
- phone verification 
- account number and PayPal verification 
- charge wallet for in-app transactions
- depositor can call `deposit` method to transfer nft by spending in-app tokens from his wallet
- withdrawer can call `withdraw` method to claim nft to update his in-app token balance
