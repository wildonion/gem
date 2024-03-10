

use crate::*;

pub struct Passport;


// https://docs.rs/actix-telepathy/latest/actix_telepathy/#:~:text=To%20send%20messages%20between%20remote,RemoteAddr%20that%20the%20ClusterListener%20receives.&text=Now%2C%20every%20new%20member%20receives,every%20ClusterListener%20in%20the%20cluster.
// check the request header containing jwt before proceeding further with handling the request
