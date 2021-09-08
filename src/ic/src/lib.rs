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
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

thread_local! {
    static STATE: State = State::default();
}

const MAX_DESCRIPTION_LENGTH: usize = 200;
const MAX_SEARCH_RESULTS: usize = 20;

#[derive(Default)]
struct State {
    data: RefCell<HashMap<Hash, Record>>,
    matcher: RefCell<SkimMatcherV2>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct StableState {
    data: HashMap<Hash, Record>,
    assets: crate::assets::StableState,
}

fn to_result(r: &Record) -> RecordResult {
    RecordResult {
        hash: r.hash.clone(),
        owner: r.owner.clone(),
        description: r.description.clone(),
        created: r.created,
    }
}

#[update]
fn notarize(datum: Datum, description: String) -> Option<RecordResult> {
    let hash = crate::assets::hash_bytes(&datum.content);
    let key = hex::encode(hash);
    STATE.with(move |s| match s.data.borrow_mut().entry(key.clone()) {
        Entry::Occupied(_e) => None,
        Entry::Vacant(e) => {
            let now = time() as u64;
            crate::assets::do_put(
                "/".to_owned() + &key,
                hash,
                datum.content_type.clone(),
                datum.content.clone(),
            );
            let record = Record {
                hash: key,
                owner: caller(),
                datum: datum,
                description: description,
                created: now,
            };
            let result = to_result(&record);
            e.insert(record);
            Some(result)
        }
    })
}

#[query]
fn get_datum(hash: Hash) -> Option<Datum> {
    STATE.with(|s| match s.data.borrow().get(&hash) {
        Some(r) => Some(r.datum.clone()),
        None => None,
    })
}

#[query]
fn get_data() -> Vec<RecordResult> {
    STATE.with(|s| {
        s.data
            .borrow()
            .iter()
            .map(|(_key, record)| to_result(&record))
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
                    to_result(&record),
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
        top_data.retain(|x| x.0.is_some() && uniques.insert(x.1.hash.clone()));
        top_data.sort_by(|a, b| match (a.0, b.0) {
            (Some(a_score), Some(b_score)) => a_score.cmp(&b_score).reverse(),
            _ => Ordering::Equal, // Not happening.
        });
        let end = std::cmp::min(MAX_SEARCH_RESULTS, top_data.len());
        let top_data: Vec<RecordResult> = top_data[..end].iter().map(|x| x.1.clone()).collect();
        top_data
    })
}

fn is_authorized() -> Result<(), String> {
    crate::assets::is_authorized()
}

fn do_clear() {
    STATE.with(|s| {
        s.data.borrow_mut().clear();
    })
}

#[update(guard = "is_authorized")]
fn clear() {
    do_clear();
    crate::assets::do_clear();
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
