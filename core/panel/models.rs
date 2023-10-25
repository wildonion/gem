


/*   ---------------------------------------------------------------------------------------------
    | https://gist.github.com/steveh/7c7145409a5eed6b698ee8b609b6d1fc
    |
    |   users             ---> postgres users table 
    |   tasks             ---> postgres tasks table
    |   users_tasks       ---> postgres users_tasks table
    |   xbot              ---> the Xbot model and verification methods
    |   users_deposits    ---> postgres users_deposits table
    |   users_withdrawals ---> postgres users_deposits table
    |   users_mails       ---> postgres users_mails table
    |   users_phones      ---> postgres users_phones table
    |   users_checkouts   ---> postgres users_checkouts table
    |   users_galleries   ---> postgres users_galleries table
    |   users_fans        ---> postgres users_fans table
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
pub mod users_fans;