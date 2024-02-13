


/*  > ---------------------------------------------------------------------------------------------
    | https://gist.github.com/steveh/7c7145409a5eed6b698ee8b609b6d1fc
    |
    |   users                         ---> methods of postgres users table 
    |   tasks                         ---> methods of postgres tasks table
    |   users_tasks                   ---> methods of postgres users_tasks table
    |   xbot                          ---> the Xbot model and verification methods
    |   users_deposits                ---> methods of postgres users_deposits table
    |   users_withdrawals             ---> methods of postgres users_deposits table
    |   users_mails                   ---> methods of postgres users_mails table
    |   users_phones                  ---> methods of postgres users_phones table
    |   users_checkouts               ---> methods of postgres users_checkouts table
    |   users_galleries               ---> methods of postgres users_galleries table
    |   users_nfts                    ---> methods of postgres users_nfts table
    |   users_collections             ---> methods of postgres users_collections table
    |   users_fans                    ---> methods of postgres users_fans table
    |   users_clps                    ---> methods of postgres users_clps table
    |   clp_events                    ---> methods of postgres clp_events table
    |   chatdb                        ---> methods of spacetimdb chatdb server
    |   token_stats                   ---> methods of postgres token_stats table
    |   nfts_likes                    ---> methods of postgres nfts_likes table
    |   nfts_comments                 ---> methods of postgres nfts_comments table
    |   galleries_invitation_requests ---> methods of postgres galleries_invitation_requests table
    |   users_friends                 ---> methods of postgres users_friends table
    |   users_logins                  ---> methods of postgres users_logins table
    |
*/

pub mod users;
pub mod tasks;
pub mod users_tasks;
pub mod xbot;
pub mod users_deposits;
pub mod users_withdrawals;
pub mod users_mails;
pub mod users_phones;
pub mod users_checkouts;
pub mod users_galleries;
pub mod users_nfts;
pub mod users_collections;
pub mod users_fans;
pub mod users_clps;
pub mod clp_events;
pub mod chatdb;
pub mod token_stats;
pub mod nfts_comments;
pub mod nfts_likes;
pub mod galleries_invitation_requests;
pub mod users_friends;
pub mod users_logins;