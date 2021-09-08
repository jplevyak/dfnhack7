use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use serde_bytes::ByteBuf;
use std::cmp::Eq;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Record {
    pub hash: Hash,
    pub owner: Principal,
    pub datum: Datum,
    pub description: String,
    pub created: Timestamp,
}

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct Datum {
    pub content_type: String,
    pub content: ByteBuf,
}

pub type Hash = String;
pub type Timestamp = u64;
pub type SearchTerms = String;

#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct RecordResult {
    pub hash: Hash,
    pub owner: Principal,
    pub description: String,
    pub created: Timestamp,
}
