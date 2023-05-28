
<p align="center">
    <img src="https://github.com/wildonion/gem/blob/master/assets/yewrustwasm.avif"
</p>

# functional, macro and access level based design pattern for gem admin and dev panel APIs.

**🦀 Rust flows in you 🌊**: write codes that are the matter-of-futer flows!

## 🧐 Good 2 Know

* this is a micro service in which all its APIs are designed based on access levels not the database models. 

* to use dev APIs Remember to run conse hyper server first.

* to generate a new password for admin and dev users just edit the `argon2test.rs` code inside the `tests` folder then run ```cargo run --bin argon2test``` to generate those passwords finally update the `up.sql` inside the `migrations/2023-05-22-184005_users` folder to insert a new admin and dev user info into the table when you run ```diesel migration run```. 

* current dev and admin passwords are `d3v@%$^$3hjsD` and `4dmin@%$^$3hjsD` respectively.

## 🍟 Features

* register new task 

* register new admin

* user task reports

* reveal role by admin

* register new event by sending SMS

* user login with wallet

* admin login with email

## 🛠️ Tools

* utoipa open api doc with swagger ui

* redis for realtime task and reveal role streaming using pubsub pattern

* postgres db to store data

* actix based http server

## 🚧 WIPs

* admin SMS panel to register new event

* create a proc macro attribute like `#[passport]` to put on top of the admin and dev apis, struct and their fields

* completing god and dev panel app using `yew`

* generating swagger doc with utoipa 

* custom error type inside `error.rs`

* publish reveal role and task topics  

* dev apis related to the conse hyper server 

* `ed25519` keypair for server checksum, verification using its commit (like ssh keys) and **SSL/TLS** certificate, updating app and time hash based (**`hash(user_id + time + ip + user agent)`**) locking api with rate limit feature to avoid api call spamming (like sleeping in thread) using `argon2`, `rust-crypto`, `noise` and `ring` tools, also see the one inside the [payma](https://github.com/wildonion/payma) repo.

* backend design pattern sketch using freeform and ERD schemas inside wiki.

* `MMQ` and order matching engine to suggests players events and other games based on their past experiences, scores (MMR) and earned tokens also and order matching engine for players to sell their minted roles based on highest or lowest order in order book.