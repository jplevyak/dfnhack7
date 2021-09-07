#[macro_use]
extern crate log;
extern crate serde_json;
use candid::{Decode, Encode};
use dfnhack7_common;
use dotenv::dotenv;
use hyper::{
    header,
    server::Server,
    service::{make_service_fn, service_fn},
    Body, Request, Response, StatusCode,
};
use ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport;
use ic_agent::Agent;
use ic_types::Principal;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const IC_ROOT_KEY: &[u8; 133] = b"\x30\x81\x82\x30\x1d\x06\x0d\x2b\x06\x01\x04\x01\x82\xdc\x7c\x05\x03\x01\x02\x01\x06\x0c\x2b\x06\x01\x04\x01\x82\xdc\x7c\x05\x03\x02\x01\x03\x61\x00\x81\x4c\x0e\x6e\xc7\x1f\xab\x58\x3b\x08\xbd\x81\x37\x3c\x25\x5c\x3c\x37\x1b\x2e\x84\x86\x3c\x98\xa4\xf1\xe0\x8b\x74\x23\x5d\x14\xfb\x5d\x9c\x0c\xd5\x46\xd9\x68\x5f\x91\x3a\x0c\x0b\x2c\xc5\x34\x15\x83\xbf\x4b\x43\x92\xe4\x67\xdb\x96\xd6\x5b\x9b\xb4\xcb\x71\x71\x12\xf8\x47\x2e\x0d\x5a\x4d\x14\x50\x5f\xfd\x74\x84\xb0\x12\x91\x09\x1c\x5f\x87\xb9\x88\x83\x46\x3f\x98\x09\x1a\x0b\xaa\xae";
const CANISTER_URL_TEMPLATE: &str = "https://{}.ic0.app";

gflags::define! {
    --canister-id: &str
}
gflags::define! {
    --port: u16
}
gflags::define! {
    --get-root-key = false
}

struct State {
    data: HashMap<String, String>,
    checked: u64,
}

#[tokio::main]
async fn main() -> () {
    dotenv().ok();
    pretty_env_logger::init();
    let _args = gflags::parse();
    if !PORT.is_present() || PORT.flag == 0 {
        error!("port flag missing or empty");
        std::process::exit(1);
    }
    if !CANISTER_ID.is_present() || CANISTER_ID.flag == "" {
        error!("canister_id flag missing or empty: {}", CANISTER_ID.flag);
        std::process::exit(1);
    }

    let canister_url = CANISTER_URL_TEMPLATE.replace("{}", CANISTER_ID.flag);
    let agent = Agent::builder()
        .with_transport(ReqwestHttpReplicaV2Transport::create(canister_url.clone()).unwrap())
        .build()
        .expect("build agent");
    if GET_ROOT_KEY.flag {
        warn!("fetching root key");
        agent.fetch_root_key().await.expect("get root key");
    } else {
        agent
            .set_root_key(IC_ROOT_KEY.to_vec())
            .expect("set root key");
    };
    let state: Arc<Mutex<State>> = Arc::new(Mutex::new(State {
        data: HashMap::new(),
        checked: 0,
    }));
    update_data(state.clone(), agent.clone()).await;

    let addr = "127.0.0.1:3100".parse().unwrap();
    let service = {
        let state = state.clone();
        make_service_fn(move |_conn| {
            let state = state.clone();
            let canister_url = canister_url.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    serve_data(req, canister_url.clone(), state.clone())
                }))
            }
        })
    };
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            update_data(state.clone(), agent.clone()).await;
        }
    });
    // Start the server
    let server = Server::bind(&addr).serve(service);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

pub fn make_response(status_code: StatusCode, body: &str) -> Response<Body> {
    let mut resp = Response::new(Body::from(body.to_string()));
    *resp.status_mut() = status_code;
    resp
}

async fn serve_data(
    req: Request<Body>,
    canister_uri: String,
    state: Arc<Mutex<State>>,
) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();
    if path == "" {
        return redirect(&canister_uri);
    }
    let path = &path[1..path.len()];
    if path == "" {
        return redirect(&canister_uri);
    }
    match state.lock().unwrap().data.get(path) {
        Some(r) => {
            eprintln!("data {} to {}", path, r);
            let response = Response::new(Body::from(r.to_string()));
            Ok(response)
        }
        None => {
            eprintln!("missing data {}", path);
            Ok(make_response(StatusCode::NOT_FOUND, "Not Found"))
        }
    }
}

fn redirect(uri: &str) -> Result<Response<Body>, Infallible> {
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, uri)
        .body(Body::from(""))
        .unwrap())
}

async fn update_data(state: Arc<Mutex<State>>, agent: Agent) {
    let waiter = delay::Delay::builder()
        .throttle(std::time::Duration::from_millis(500))
        .timeout(std::time::Duration::from_secs(60))
        .build();
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let canister_id = Principal::from_text(CANISTER_ID.flag).expect("Principal::from_text");
    let checked = Encode!(&state.lock().unwrap().checked).unwrap();
    let response = agent
        .update(&canister_id, "get_updated_links")
        .with_arg(&checked)
        .call_and_wait(waiter)
        .await
        .expect("response");
    let result = Decode!(
        response.as_slice(),
        Vec<dfnhack7_common::UpdatedRecordResult>
    )
    .expect("result");
    {
        let mut state = state.lock().unwrap();
        for r in result {
            if let Some(canister_id) = r.canister_id {
                state.data.insert(r.datum, canister_id);
            }
        }
        // Play it safe for clock skew.
        state.checked = (since_the_epoch - std::time::Duration::from_secs(10)).as_nanos() as u64;
    }
}
