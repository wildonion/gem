

conse mafia game the user, dev and admin in-game APIs


## 🎢 Development Wrapups

* to update a user access level to `dev` first do a signup for the user using `/auth/signup` API then run the mafia binary server like so: `./mafia dev 0` or `cargo run --bin mafia dev 0` finally login with that user to register a new god for the game.

* the default `dev` and `conse` user passwords are `dev@1234%` and `conse@1234` respectively which will be imported to the mongodb automatically by running the `./redeploy.sh` script, also we can run the `./mafia dev 0` binary inside the VPS to update the access level of a user to dev, finally we can register a new god or admin for the mafia game APIs using the dev user token.

* to update a user access level of the conse mafia hyper server to dev, first signup the user using `/auth/signup` API then update the `access_level` field of the user to `0` manually inside the db in `mongodb` container using `portainer` finally login with dev user to register a new god for the game.