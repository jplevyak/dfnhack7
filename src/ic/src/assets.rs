use crate::rc_bytes::RcBytes;
use ic_cdk::api::{caller, data_certificate, set_certified_data, time, trap};
use ic_cdk::export::candid::{CandidType, Deserialize, Func, Nat, Principal};
use ic_cdk_macros::update;
use ic_certified_map::{AsHashTree, Hash, HashTree, RbTree};
use num_traits::ToPrimitive;
use serde::Serialize;
use serde_bytes::ByteBuf;
use sha2::Digest;
use std::cell::RefCell;
use std::collections::HashMap;

/// The order in which we pick encodings for certification.
const ENCODING_CERTIFICATION_ORDER: &[&str] = &["identity", "gzip", "compress", "deflate", "br"];

/// The file to serve if the requested file wasn't found.
const INDEX_FILE: &str = "/index.html";

thread_local! {
    static STATE: State = State::default();
    static ASSET_HASHES: RefCell<AssetHashes> = RefCell::new(RbTree::new());
}

type AssetHashes = RbTree<Key, Hash>;

#[derive(Default)]
struct State {
    assets: RefCell<HashMap<Key, Asset>>,
    authorized: RefCell<Vec<Principal>>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StableState {
    authorized: Vec<Principal>,
    stable_assets: HashMap<String, Asset>,
}

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
struct AssetEncoding {
    modified: Timestamp,
    content_chunks: Vec<RcBytes>,
    total_length: usize,
    certified: bool,
    sha256: [u8; 32],
}

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
struct Asset {
    content_type: String,
    encodings: HashMap<String, AssetEncoding>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct EncodedAsset {
    content: RcBytes,
    content_type: String,
    content_encoding: String,
    total_length: Nat,
    sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct AssetDetails {
    key: String,
    content_type: String,
    encodings: Vec<AssetEncodingDetails>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct AssetEncodingDetails {
    modified: Timestamp,
    content_encoding: String,
    sha256: Option<ByteBuf>,
    length: Nat,
}
type Timestamp = u64;
pub type Key = String;

// HTTP interface

type HeaderField = (String, String);

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpRequest {
    method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    body: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse {
    status_code: u16,
    headers: Vec<HeaderField>,
    body: RcBytes,
    streaming_strategy: Option<StreamingStrategy>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Token {
    pub key: String,
    content_encoding: String,
    index: Nat,
    // We don't care about the sha, we just want to be backward compatible.
    sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum StreamingStrategy {
    Callback { callback: Func, token: Token },
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StreamingCallbackHttpResponse {
    body: RcBytes,
    token: Option<Token>,
}

#[update]
fn authorize(other: Principal) {
    let caller = caller();
    STATE.with(|s| {
        let caller_autorized = s.authorized.borrow().iter().any(|p| *p == caller);
        if caller_autorized {
            s.authorized.borrow_mut().push(other);
        }
    })
}

pub fn do_get(key: Key) -> RcBytes {
    STATE.with(|s| {
        let assets = s.assets.borrow();
        let asset = assets.get(&key).unwrap_or_else(|| trap("asset not found"));
        let id_enc = asset
            .encodings
            .get("identity")
            .unwrap_or_else(|| trap("no identity encoding"));
        if id_enc.content_chunks.len() > 1 {
            trap("Asset too large. Use get() and get_chunk() instead.");
        }
        id_enc.content_chunks[0].clone()
    })
}

pub fn do_put(key: Key, hash: Hash, content_type: String, content: ByteBuf) {
    let content_encoding = "identity".to_string();
    STATE.with(move |s| {
        let mut assets = s.assets.borrow_mut();
        let asset = assets.entry(key.clone()).or_default();
        asset.content_type = content_type;
        let encoding = asset.encodings.entry(content_encoding).or_default();
        encoding.total_length = content.len();
        encoding.content_chunks = vec![RcBytes::from(content)];
        encoding.modified = time() as u64;
        encoding.sha256 = hash;

        on_asset_change(&key, asset);
    });
}

fn create_token(
    _asset: &Asset,
    enc_name: &str,
    enc: &AssetEncoding,
    key: &str,
    chunk_index: usize,
) -> Option<Token> {
    if chunk_index + 1 >= enc.content_chunks.len() {
        None
    } else {
        Some(Token {
            key: key.to_string(),
            content_encoding: enc_name.to_string(),
            index: Nat::from(chunk_index + 1),
            sha256: Some(ByteBuf::from(enc.sha256)),
        })
    }
}

fn create_strategy(
    asset: &Asset,
    enc_name: &str,
    enc: &AssetEncoding,
    key: &str,
    chunk_index: usize,
) -> Option<StreamingStrategy> {
    create_token(asset, enc_name, enc, key, chunk_index).map(|token| StreamingStrategy::Callback {
        callback: ic_cdk::export::candid::Func {
            method: "http_request_streaming_callback".to_string(),
            principal: ic_cdk::id(),
        },
        token,
    })
}

fn build_200(
    asset: &Asset,
    enc_name: &str,
    enc: &AssetEncoding,
    key: &str,
    chunk_index: usize,
    certificate_header: Option<HeaderField>,
) -> HttpResponse {
    let mut headers = vec![("Content-Type".to_string(), asset.content_type.to_string())];
    if enc_name != "identity" {
        headers.push(("Content-Encoding".to_string(), enc_name.to_string()));
    }
    if let Some(head) = certificate_header {
        headers.push(head);
    }

    let streaming_strategy = create_strategy(asset, enc_name, enc, key, chunk_index);

    HttpResponse {
        status_code: 200,
        headers,
        body: enc.content_chunks[chunk_index].clone(),
        streaming_strategy,
    }
}

fn build_404(certificate_header: HeaderField) -> HttpResponse {
    HttpResponse {
        status_code: 404,
        headers: vec![certificate_header],
        body: RcBytes::from(ByteBuf::from("not found")),
        streaming_strategy: None,
    }
}

pub fn build_http_response(path: &str, encodings: Vec<String>, index: usize) -> HttpResponse {
    STATE.with(|s| {
        let assets = s.assets.borrow();

        let index_redirect_certificate = ASSET_HASHES.with(|t| {
            let tree = t.borrow();
            if tree.get(path.as_bytes()).is_none() && tree.get(INDEX_FILE.as_bytes()).is_some() {
                let absence_proof = tree.witness(path.as_bytes());
                let index_proof = tree.witness(INDEX_FILE.as_bytes());
                let combined_proof = merge_hash_trees(absence_proof, index_proof);
                Some(witness_to_header(combined_proof))
            } else {
                None
            }
        });

        if let Some(certificate_header) = index_redirect_certificate {
            if let Some(asset) = assets.get(INDEX_FILE) {
                for (enc_name, enc) in asset.encodings.iter() {
                    if enc.certified {
                        return build_200(
                            asset,
                            enc_name,
                            enc,
                            INDEX_FILE,
                            index,
                            Some(certificate_header),
                        );
                    }
                }
            }
        }

        let certificate_header =
            ASSET_HASHES.with(|t| witness_to_header(t.borrow().witness(path.as_bytes())));

        if let Some(asset) = assets.get(path) {
            for enc_name in encodings.iter() {
                if let Some(enc) = asset.encodings.get(enc_name) {
                    if enc.certified {
                        return build_200(
                            asset,
                            enc_name,
                            enc,
                            path,
                            index,
                            Some(certificate_header),
                        );
                    } else {
                        // Find if identity is certified, if it's not.
                        if let Some(id_enc) = asset.encodings.get("identity") {
                            if id_enc.certified {
                                return build_200(
                                    asset,
                                    enc_name,
                                    enc,
                                    path,
                                    index,
                                    Some(certificate_header),
                                );
                            }
                        }
                    }
                }
            }
        }

        build_404(certificate_header)
    })
}

/// An iterator-like structure that decode a URL.
struct UrlDecode<'a> {
    bytes: std::slice::Iter<'a, u8>,
}

fn convert_percent(iter: &mut std::slice::Iter<u8>) -> Option<u8> {
    let mut cloned_iter = iter.clone();
    let result = match cloned_iter.next()? {
        b'%' => b'%',
        h => {
            let h = char::from(*h).to_digit(16)?;
            let l = char::from(*cloned_iter.next()?).to_digit(16)?;
            h as u8 * 0x10 + l as u8
        }
    };
    // Update this if we make it this far, otherwise "reset" the iterator.
    *iter = cloned_iter;
    Some(result)
}

impl<'a> Iterator for UrlDecode<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let b = self.bytes.next()?;
        match b {
            b'%' => Some(char::from(convert_percent(&mut self.bytes).expect(
                "error decoding url: % must be followed by '%' or two hex digits",
            ))),
            b'+' => Some(' '),
            x => Some(char::from(*x)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let bytes = self.bytes.len();
        (bytes / 3, Some(bytes))
    }
}

pub fn url_decode(url: &str) -> String {
    UrlDecode {
        bytes: url.as_bytes().into_iter(),
    }
    .collect()
}

#[test]
fn check_url_decode() {
    assert_eq!(url_decode("/%"), "/%");
    assert_eq!(url_decode("/%%"), "/%");
    assert_eq!(url_decode("/%20a"), "/ a");
    assert_eq!(url_decode("/%%+a%20+%@"), "/% a  %@");
    assert_eq!(url_decode("/has%percent.txt"), "/has%percent.txt");
    assert_eq!(url_decode("/%e6"), "/æ");
}

pub fn http_request_streaming_callback(
    Token {
        key,
        content_encoding,
        index,
        ..
    }: Token,
) -> StreamingCallbackHttpResponse {
    STATE.with(|s| {
        let assets = s.assets.borrow();
        let asset = assets
            .get(&key)
            .expect("Invalid token on streaming: key not found.");
        let enc = asset
            .encodings
            .get(&content_encoding)
            .expect("Invalid token on streaming: encoding not found.");
        // MAX is good enough. This means a chunk would be above 64-bits, which is impossible...
        let chunk_index = index.0.to_usize().unwrap_or(usize::MAX);

        StreamingCallbackHttpResponse {
            body: enc.content_chunks[chunk_index].clone(),
            token: create_token(&asset, &content_encoding, enc, &key, chunk_index),
        }
    })
}

pub fn do_delete(key: &str) {
    STATE.with(|s| {
        let mut assets = s.assets.borrow_mut();
        assets.remove(key);
    });
    delete_asset_hash(key);
}

pub fn do_clear() {
    STATE.with(|s| {
        s.assets.borrow_mut().clear();
    })
}

pub fn is_authorized() -> Result<(), String> {
    STATE.with(|s| {
        s.authorized
            .borrow()
            .contains(&caller())
            .then(|| ())
            .ok_or("Caller is not authorized".to_string())
    })
}

fn on_asset_change(key: &str, asset: &mut Asset) {
    // If the most preferred encoding is present and certified,
    // there is nothing to do.
    for enc_name in ENCODING_CERTIFICATION_ORDER.iter() {
        if let Some(enc) = asset.encodings.get(*enc_name) {
            if enc.certified {
                return;
            }
        }
    }

    if asset.encodings.is_empty() {
        delete_asset_hash(key);
        return;
    }

    // An encoding with a higher priority was added, let's certify it
    // instead.

    for enc in asset.encodings.values_mut() {
        enc.certified = false;
    }

    for enc_name in ENCODING_CERTIFICATION_ORDER.iter() {
        if let Some(enc) = asset.encodings.get_mut(*enc_name) {
            certify_asset(key.to_string(), &enc.sha256);
            enc.certified = true;
            return;
        }
    }

    // No known encodings found. Just pick the first one. The exact
    // order is hard to predict because we use a hash map. Should
    // almost never happen anyway.
    if let Some(enc) = asset.encodings.values_mut().next() {
        certify_asset(key.to_string(), &enc.sha256);
        enc.certified = true;
    }
}

fn certify_asset(key: Key, content_hash: &Hash) {
    ASSET_HASHES.with(|t| {
        let mut tree = t.borrow_mut();
        tree.insert(key, *content_hash);
        set_root_hash(&*tree);
    });
}

fn delete_asset_hash(key: &str) {
    ASSET_HASHES.with(|t| {
        let mut tree = t.borrow_mut();
        tree.delete(key.as_bytes());
        set_root_hash(&*tree);
    });
}

fn set_root_hash(tree: &AssetHashes) {
    use ic_certified_map::labeled_hash;
    let full_tree_hash = labeled_hash(b"http_assets", &tree.root_hash());
    set_certified_data(&full_tree_hash);
}

fn witness_to_header(witness: HashTree) -> HeaderField {
    use ic_certified_map::labeled;

    let hash_tree = labeled(b"http_assets", witness);
    let mut serializer = serde_cbor::ser::Serializer::new(vec![]);
    serializer.self_describe().unwrap();
    hash_tree.serialize(&mut serializer).unwrap();

    let certificate = data_certificate().unwrap_or_else(|| trap("no data certificate available"));

    (
        "IC-Certificate".to_string(),
        String::from("certificate=:")
            + &base64::encode(&certificate)
            + ":, tree=:"
            + &base64::encode(&serializer.into_inner())
            + ":",
    )
}

fn merge_hash_trees<'a>(lhs: HashTree<'a>, rhs: HashTree<'a>) -> HashTree<'a> {
    use HashTree::{Empty, Fork, Labeled, Leaf, Pruned};

    match (lhs, rhs) {
        (Pruned(l), Pruned(r)) => {
            if l != r {
                trap("merge_hash_trees: inconsistent hashes");
            }
            Pruned(l)
        }
        (Pruned(_), r) => r,
        (l, Pruned(_)) => l,
        (Fork(l), Fork(r)) => Fork(Box::new((
            merge_hash_trees(l.0, r.0),
            merge_hash_trees(l.1, r.1),
        ))),
        (Labeled(l_label, l), Labeled(r_label, r)) => {
            if l_label != r_label {
                trap("merge_hash_trees: inconsistent hash tree labels");
            }
            Labeled(l_label, Box::new(merge_hash_trees(*l, *r)))
        }
        (Empty, Empty) => Empty,
        (Leaf(l), Leaf(r)) => {
            if l != r {
                trap("merge_hash_trees: inconsistent leaves");
            }
            Leaf(l)
        }
        (_l, _r) => {
            trap("merge_hash_trees: inconsistent tree structure");
        }
    }
}

pub fn hash_bytes(bytes: &[u8]) -> Hash {
    let mut hash = sha2::Sha256::new();
    hash.update(bytes);
    hash.finalize().into()
}

pub fn init() {
    do_clear();
    STATE.with(|s| s.authorized.borrow_mut().push(caller()));
}

pub fn pre_upgrade() -> StableState {
    STATE.with(|s| StableState {
        authorized: s.authorized.take(),
        stable_assets: s.assets.take(),
    })
}

pub fn post_upgrade(stable_state: StableState) {
    do_clear();
    STATE.with(|s| {
        s.authorized.replace(stable_state.authorized);
        s.assets.replace(stable_state.stable_assets);

        for (asset_name, asset) in s.assets.borrow_mut().iter_mut() {
            for enc in asset.encodings.values_mut() {
                enc.certified = false;
            }
            on_asset_change(asset_name, asset);
        }
    });
}
