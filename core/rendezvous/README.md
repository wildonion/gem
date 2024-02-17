

conse rendezvous service, an event managing and reservation system with reveal-participant-role functionality, the user, dev and admin in-game APIs

> event managers can define event so players can reserve the event also each event contains a deck which contains some predefined roles which will be assigned randomly to each player once the event manager submit on the reveal role button.

## 🎢 Development Wrapups

* to update a user access level to `dev` first do a signup for the user using `/auth/signup` API then run the rendezvous binary server like so: `./rendezvous dev 0` or `cargo run --bin rendezvous dev 0` finally login with that user to register a new god for the game.

* the default `dev` and `conse` admin user passwords are `dev@1234%` and `conse@1234` respectively which will be imported to the mongodb automatically by running the `./redeploy.sh` script, also we can run the `./rendezvous dev 0` binary inside the VPS to update the access level of a user to dev, finally we can register a new god or admin for the rendezvous service APIs using the dev user token.

* to update a user access level of the conse rendezvous hyper server to dev, first signup the user using `/auth/signup` API then update the `access_level` field of the user to `0` manually inside the db in `mongodb` container using `portainer` finally login with dev user to register a new god for the game.