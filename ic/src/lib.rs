mod assets;
mod rc_bytes;

use candid::{CandidType, Deserialize};
use dfnhack7_common::*;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ic_cdk::api::{caller, time};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

thread_local! {
    static STATE: State = State::default();
}

const CYCLES_PER_UNIT_TIME: u64 = 1_000_000_000_000; // 1T
const UNIT_TIME_NANOS: u64 = 2628_000_000_000_000; // 1/12 of a year.
const MIN_LINK_LENGTH: usize = 4;
const MAX_LINK_LENGTH: usize = 50;
const MAX_DESCRIPTION_LENGTH: usize = 200;
const MAX_SEARCH_RESULTS: usize = 20;
const LEGAL_LINK_CHARS: &str = "=";

#[derive(Default)]
struct State {
    data: RefCell<HashMap<Datum, Record>>,
    matcher: RefCell<SkimMatcherV2>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct StableState {
    data: HashMap<Datum, Record>,
    assets: crate::assets::StableState,
}

fn to_result(datum: &Datum, r: &Record) -> RecordResult {
    RecordResult {
        datum: datum.clone(),
        expires: r.expires,
        canister_id: r.canister_id.clone(),
        description: r.description.clone(),
    }
}

fn to_updated_result(datum: &Datum, r: &Record) -> UpdatedRecordResult {
    UpdatedRecordResult {
        datum: datum.clone(),
        canister_id: r.canister_id.clone(),
    }
}

#[query]
fn get_datum(data: Datum) -> Option<RecordResult> {
    STATE.with(|s| match s.data.borrow().get(&data) {
        Some(r) => Some(to_result(&data, &r)),
        None => None,
    })
}

#[query]
fn get_data() -> Vec<RecordResult> {
    STATE.with(|s| {
        s.data
            .borrow()
            .iter()
            .map(|(key, record)| to_result(&key, &record))
            .collect::<Vec<_>>()
    })
}

#[query]
fn search(search_terms: SearchTerms) -> Vec<RecordResult> {
    STATE.with(|s| {
        let matcher = s.matcher.borrow();
        let mut matches = s
            .data
            .borrow()
            .iter()
            .map(|(key, record)| {
                (
                    matcher.fuzzy_match(&key.to_lowercase(), &search_terms.to_lowercase()),
                    matcher.fuzzy_match(
                        &record.description.to_lowercase(),
                        &search_terms.to_lowercase(),
                    ),
                    to_result(&key, &record),
                )
            })
            .collect::<Vec<_>>();
        matches.sort_by(|a, b| match (a.0, b.0) {
            (Some(a_score), Some(b_score)) => a_score.cmp(&b_score).reverse(),
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => Ordering::Equal,
        });
        let end = std::cmp::min(MAX_SEARCH_RESULTS, matches.len());
        let mut top_data: Vec<(Option<i64>, RecordResult)> =
            matches[..end].iter().map(|x| (x.0, x.2.clone())).collect();
        matches.sort_by(|a, b| match (a.1, b.1) {
            (Some(a_score), Some(b_score)) => a_score.cmp(&b_score).reverse(),
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => Ordering::Equal,
        });
        let mut top_descriptions: Vec<(Option<i64>, RecordResult)> =
            matches[..end].iter().map(|x| (x.1, x.2.clone())).collect();
        top_data.append(&mut top_descriptions);
        let mut uniques = HashSet::new();
        top_data.retain(|x| x.0.is_some() && uniques.insert(x.1.datum.clone()));
        top_data.sort_by(|a, b| match (a.0, b.0) {
            (Some(a_score), Some(b_score)) => a_score.cmp(&b_score).reverse(),
            _ => Ordering::Equal, // Not happening.
        });
        let end = std::cmp::min(MAX_SEARCH_RESULTS, top_data.len());
        let top_data: Vec<RecordResult> = top_data[..end].iter().map(|x| x.1.clone()).collect();
        top_data
    })
}

#[query]
fn get_updated_links(from: u64) -> Vec<UpdatedRecordResult> {
    STATE.with(|s| {
        s.data
            .borrow()
            .iter()
            .filter(|(_, r)| r.updated >= from)
            .map(|(key, record)| to_updated_result(&key, &record))
            .collect::<Vec<_>>()
    })
}

fn cycles_to_time(cycles: u64) -> u64 {
    ((cycles as f64 / CYCLES_PER_UNIT_TIME as f64) * UNIT_TIME_NANOS as f64) as u64
}

#[update]
fn claim_data(arg: ClaimDatumArg) {
    let caller = caller();
    let amount = ic_cdk::api::call::msg_cycles_available();
    assert!(amount > 0);
    let amount: u64 = amount as u64;
    assert!(amount as u64 >= CYCLES_PER_UNIT_TIME);
    assert!(arg.datum.len() >= MIN_LINK_LENGTH);
    assert!(arg.datum.len() <= MAX_LINK_LENGTH);
    if let Some(description) = &arg.description {
        assert!(description.len() < MAX_DESCRIPTION_LENGTH);
    }
    assert!(arg
        .datum
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || LEGAL_LINK_CHARS.find(c).is_some()));
    let now = time() as u64;
    STATE.with(move |s| {
        match s.data.borrow_mut().get_mut(&arg.datum) {
            Some(r) => {
                if *r.owner.borrow() == Some(caller.clone()) {
                    // Reset the data, extend the time.
                    r.canister_id = arg.canister_id;
                    if let Some(description) = arg.description {
                        r.description = description;
                    }
                    r.updated = now;
                    let amount = amount - CYCLES_PER_UNIT_TIME;
                    r.expires += cycles_to_time(amount);
                    return;
                }
                assert!(now > r.expires);
            }
            None => (),
        }
        let r = Record {
            canister_id: arg.canister_id,
            owner: RefCell::new(Some(caller)),
            description: arg.description.unwrap_or("".to_string()),
            updated: now,
            expires: now + cycles_to_time(amount),
        };
        s.data.borrow_mut().insert(arg.datum, r);
    });
}

#[query(guard = "is_authorized")]
fn get_data_reserved() -> Vec<RecordReservedResult> {
    STATE.with(|s| {
        s.data
            .borrow()
            .iter()
            .map(|(key, record)| RecordReservedResult {
                record: to_result(&key, &record),
                owner: record.owner.borrow().clone(),
            })
            .collect::<Vec<RecordReservedResult>>()
    })
}

#[update(guard = "is_authorized")]
fn set_data_reserved(arg: SetDatumArg) {
    let caller = caller();
    let now = time() as u64;
    STATE.with(move |s| {
        let r = Record {
            canister_id: arg.canister_id,
            owner: RefCell::new(Some(caller)),
            description: arg.description.unwrap_or("".to_string()),
            updated: now,
            expires: arg.expires,
        };
        s.data.borrow_mut().insert(arg.datum, r);
    });
}

fn is_authorized() -> Result<(), String> {
    crate::assets::is_authorized()
}

fn do_clear() {
    STATE.with(|s| {
        s.data.borrow_mut().clear();
    })
}

#[init]
fn init() {
    do_clear();
    crate::assets::init();
}

#[pre_upgrade]
fn pre_upgrade() {
    let stable_state = STATE.with(|s| StableState {
        data: s.data.take(),
        assets: crate::assets::pre_upgrade(),
    });
    ic_cdk::storage::stable_save((stable_state,)).expect("failed to save stable state");
}

#[post_upgrade]
fn post_upgrade() {
    do_clear();
    let (stable_state,): (StableState,) =
        ic_cdk::storage::stable_restore().expect("failed to restore stable state");
    STATE.with(|s| {
        s.data.replace(stable_state.data);
        crate::assets::post_upgrade(stable_state.assets);
    });
}
