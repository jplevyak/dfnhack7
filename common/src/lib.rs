use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use std::cell::RefCell;
use std::cmp::Eq;
use std::hash::Hash;

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct Record {
    pub canister_id: Option<CanisterId>,
    pub owner: RefCell<Option<Principal>>,
    pub description: String,
    pub updated: Timestamp,
    pub expires: Timestamp,
}

pub type Timestamp = u64;
pub type Datum = String;
pub type CanisterId = String;
pub type SearchTerms = String;

#[derive(Clone, Debug, Hash, PartialEq, Eq, CandidType, Deserialize)]
pub struct RecordResult {
    pub datum: Datum,
    pub canister_id: Option<CanisterId>,
    pub description: String,
    pub expires: Timestamp,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct UpdatedRecordResult {
    pub datum: Datum,
    pub canister_id: Option<CanisterId>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct RecordReservedResult {
    pub record: RecordResult,
    pub owner: Option<Principal>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ClaimDatumArg {
    pub datum: Datum,
    pub canister_id: Option<CanisterId>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct SetDatumArg {
    pub datum: Datum,
    pub canister_id: Option<CanisterId>,
    pub description: Option<String>,
    pub expires: Timestamp,
}
