


## conse user, dev and admin dashboard panel APIs.

### 👤 User Access
- push notif subscriptions (mmr ranking, ecq and reveal role) `<---mafia jwt--->` mafia hyper server
- twitter, otp, mail, identity and bank accounts verification process
- new login and check token flow
- building crypto wallet
- do and get related tasks
- deposit and withdraw NFT `<---exchange token--->` ir and paypal servers
- get related deposits and withdrawals
- add comment on and like the post

### 👑 Admin Access
- advertise event using sms panel `<---mafia jwt--->` mafia hyper server
- publish reveal role topic of an event `<---mafia jwt--->` mafia hyper server
- publish ecq topic of an event `<---mafia jwt--->` mafia hyper server
- update event image `<---mafia jwt--->` mafia hyper server
- add twitter account for the twitter bot
- register/delete/edit new tasks, posts and users
- get all withdrawals, deposits, users and tasks

### 👨🏻‍💻 Dev Access
- get all data of a user `<---mafia jwt--->` mafia hyper server
- get all data of an admin `<---mafia jwt--->` mafia hyper server

### 🌎 Public Access
- user task verification using twitter bot
- check user task 
- get posts

### 🔑 Tiny KYC Identity Verification Process

- first of all the `/user/login` API must be called to register a new user.
- second of all the `/user/request-mail-code/{mail}` and `/user/verify-mail-code` APIs must be called to verify the user mail in order to create the **Crypto Id**.
- then the `/user/cid/build` API must be called to upsert the fields, it'll create a new **Crypto Id** with the passed in `username`, `region` and `device_id`, on the first call and update `username` and `region` fields only on the second call.
- finally we can call the `/user/request-phone-code/{phone}` and `/user/verify-phone-code` APIs to verify the user phone number which will send the **OTP** code from the IR or none-IR **OTP** provider based on the updated user region in previous step.