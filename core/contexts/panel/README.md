
<p align="center">
    <img src="https://github.com/wildonion/gem/blob/master/assets/yewrustwasm.avif"
</p>

# gem admin and dev panel macro based APIs using Actix, Redis, Mongodb and Postgres

**🦀 Rust flows in you 🌊**: Write codes that you want to care about them in the future!

## 🧐 Good 2 Know

* to use dev APIs Remember to run conse hyper server first.

* to generate a new password for admin and dev users just edit the `argon2test.rs` code inside the `tests` folder then run ```cargo run --bin argon2test``` to generate those passwords finally update the `up.sql` inside the `migrations/2023-05-22-184005_users` folder to insert a new admin and dev user info into the table when you run ```diesel migration run```. 

* current dev and admin passwords are `d3v@%$^$3hjsD` and `4dmin@%$^$3hjsD` respectively.

## 🍟 Features

* register new task 

* register new admin

* user task reports

* reveal role

* register new event by sending SMS

* user login with wallet

* admin login with email

## 🛠️ Tools

* utoipa open api doc

* redis for realtime task and reveal role streaming using pubsub pattern

* postgres db to store data

* actix based http server

## 🚧 WIPs

* completing god and dev panel app using `yew`