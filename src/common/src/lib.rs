use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use serde_bytes::ByteBuf;
use std::cell::RefCell;
use std::cmp::Eq;

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct Record {
    pub hash: Hash,
    pub owner: RefCell<Option<Principal>>,
    pub datum: Option<ByteBuf>,
    pub description: String,
    pub created: Timestamp,
}

pub type Hash = String;
pub type Timestamp = u64;
pub type SearchTerms = String;

#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct RecordResult {
    pub hash: Hash,
    pub owner: RefCell<Option<Principal>>,
    pub description: String,
    pub created: Timestamp,
}
