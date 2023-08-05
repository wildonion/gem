


use crate::*;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DepositMetadata{
    pub from: Id,
    pub recipient: Id,
    pub amount: u64,
    pub iat: i64, // deposited at
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WithdrawMetadata{
    pub deposit_metadata: DepositMetadata,
    pub cat: i64, // claimed at
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Id{
    pub paypal_id: String,
    pub account_number: String,
    pub social_id: String,
    pub username: String,
    pub snowflake_id: Option<i64>,
    pub unique_id: Option<String>, /* pubkey */
    pub signer: Option<String>, /* prvkey */
    pub signature: Option<String>, /* this is the final unique id */
}