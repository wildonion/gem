// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN RUST INSTEAD.

#[allow(unused)]
use spacetimedb_sdk::{
    anyhow::{anyhow, Result},
    identity::Identity,
    reducer::{Reducer, ReducerCallbackId, Status},
    sats::{de::Deserialize, ser::Serialize},
    spacetimedb_lib,
    table::{TableIter, TableType, TableWithPrimaryKey},
    Address,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct User {
    pub screen_cid: String,
}

impl TableType for User {
    const TABLE_NAME: &'static str = "User";
    type ReducerEvent = super::ReducerEvent;
}

impl User {
    #[allow(unused)]
    pub fn filter_by_screen_cid(screen_cid: String) -> TableIter<Self> {
        Self::filter(|row| row.screen_cid == screen_cid)
    }
}