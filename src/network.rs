use crate::block::{Block, Blockchain};
use crate::ledger::Ledger;
use crate::transaction::Transaction;
use crate::validator::Validator;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use crate::wallet::Wallet;

pub async fn start_server(port: u16, blockchain: Arc<Mutex<Blockchain>>, chain_path: String) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    println!("Écoute sur {}", port);
    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let bc = blockchain.clone();
        let path = chain_path.clone();
        tokio::spawn(async move {
            let mut buf = String::new();
            socket.read_to_string(&mut buf).await.ok();
            if !buf.is_empty() {
                if let Ok(received_chain) = serde_json::from_str::<Vec<Block>>(&buf) {
                    let mut bc = bc.lock().await;
                    let candidate = Blockchain {
                        chain: received_chain,
                    };
                    if candidate.chain.len() > bc.chain.len() && candidate.is_valid() {
                        bc.chain = candidate.chain;
                        bc.save(&path);
                    }
                }
            } else {
                let bc = bc.lock().await;
                let json = serde_json::to_string(&bc.chain).unwrap();
                let _ = socket.write_all(json.as_bytes()).await;
            }
        });
    }
}

pub async fn fetch_chain(addr: &str) -> Option<Vec<Block>> {
    let mut stream = TcpStream::connect(addr).await.ok()?;
    stream.shutdown().await.ok();
    let mut buf = String::new();
    stream.read_to_string(&mut buf).await.ok()?;
    serde_json::from_str(&buf).ok()
}

pub async fn broadcast(peers: &[String], chain: &[Block]) {
    for peer in peers {
        if let Ok(mut stream) = TcpStream::connect(peer).await {
            let json = serde_json::to_string(chain).unwrap();
            let _ = stream.write_all(json.as_bytes()).await;
        }
    }
}

pub async fn fetch_peers(addr: &str) -> Option<Vec<String>> {
    let url = format!("http://{}/peers", addr);
    let resp = reqwest::get(&url).await.ok()?;
    resp.json::<Vec<String>>().await.ok()
}

#[derive(Clone)]
pub struct ApiState {
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub peers: Arc<Mutex<Vec<String>>>,
    pub ledger: Arc<Mutex<Ledger>>,
    pub validators: Arc<Mutex<Vec<Validator>>>,
}

pub async fn start_api(
    port: u16,
    blockchain: Arc<Mutex<Blockchain>>,
    peers: Arc<Mutex<Vec<String>>>,
    ledger: Arc<Mutex<Ledger>>,
    validators: Arc<Mutex<Vec<Validator>>>,
) {
    let state = ApiState {
        blockchain,
        peers,
        ledger,
        validators,
    };
    let app = Router::new()
        .route("/chain", get(get_chain))
        .route("/", get(explorer))
        .route("/peers", get(get_peers))
        .route("/balance/:address", get(get_balance))
        .route("/tx", post(post_tx))
        .route("/wallet", post(post_create_wallet))
        .route("/wallet/load", post(post_load_wallet))
        .route("/forge-block", post(post_forge_block))
        .route("/forge-hadron", post(post_forge_hadron))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    println!("API sur {}", port);
    axum::serve(listener, app).await.unwrap();
}

async fn get_chain(State(state): State<ApiState>) -> Json<Vec<Block>> {
    let bc = state.blockchain.lock().await;
    Json(bc.chain.clone())
}

async fn get_peers(State(state): State<ApiState>) -> Json<Vec<String>> {
    Json(state.peers.lock().await.clone())
}

#[derive(serde::Serialize)]
struct BalanceResponse {
    address: String,
    balance: u64,
    quarks: Vec<crate::quark::QuarkType>,
}

async fn get_balance(
    State(state): State<ApiState>,
    Path(address): Path<String>,
) -> Json<BalanceResponse> {
    let ledger = state.ledger.lock().await;
    let balance = *ledger.balances.get(&address).unwrap_or(&0);
    let validators = state.validators.lock().await;
    let quarks = validators
        .iter()
        .find(|v| v.address == address)
        .map(|v| v.quarks_reward.clone())
        .unwrap_or_default();
    Json(BalanceResponse {
        address,
        balance,
        quarks,
    })
}

async fn post_tx(
    State(state): State<ApiState>,
    Json(tx): Json<Transaction>,
) -> (StatusCode, Json<serde_json::Value>) {
    if !tx.verify() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"ok": false, "error": "signature invalide"})),
        );
    }
    let mut bc = state.blockchain.lock().await;
    let mut ledger = state.ledger.lock().await;
    let mut validators = state.validators.lock().await;
    bc.add_block(vec![tx], &mut validators, &mut ledger);
    (StatusCode::OK, Json(serde_json::json!({"ok": true})))
}

async fn explorer(State(state): State<ApiState>) -> axum::response::Html<String> {
    let bc = state.blockchain.lock().await;
    let mut html = String::from("<h1>Quark Explorer</h1>");
    for b in bc.chain.iter().rev() {
        html += &format!(
            "<p>#{} - {} - validator: {}</p>",
            b.index, b.hash, b.validator
        );
    }
    axum::response::Html(html)
}

#[derive(serde::Deserialize)]
pub struct CreateWalletRequest {
    pub password: String,
}

#[derive(serde::Serialize)]
struct CreateWalletResponse {
    address: String,
    private_key_hex: String,
}

async fn post_create_wallet(
    State(state): State<ApiState>,
    Json(req): Json<CreateWalletRequest>,
) -> Json<CreateWalletResponse> {
    let wallet = Wallet::new();
    let addr = wallet.public_key_hex();
    std::fs::create_dir_all("wallets").ok();
    wallet.save_encrypted(&format!("wallets/{}.key", addr), &req.password);

    state
        .ledger
        .lock()
        .await
        .balances
        .insert(addr.clone(), 1000);
    state.validators.lock().await.push(Validator {
        address: addr.clone(),
        staked_hadrons: vec![],
        quarks_reward: vec![],
        seed: rand::random(),
    });

    let private_key_hex = hex::encode(wallet.signing_key.to_bytes());
    Json(CreateWalletResponse {
        address: addr,
        private_key_hex,
    })
}

#[derive(serde::Deserialize)]
pub struct LoadWalletRequest {
    pub address: String,
    pub password: String,
}

async fn post_load_wallet(
    Json(req): Json<LoadWalletRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let path = format!("wallets/{}.key", req.address);
    match Wallet::load_encrypted(&path, &req.password) {
        Some(wallet) => {
            let private_key_hex = hex::encode(wallet.signing_key.to_bytes());
            (
                StatusCode::OK,
                Json(serde_json::json!({"ok": true, "private_key_hex": private_key_hex})),
            )
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(
                serde_json::json!({"ok": false, "error": "mot de passe incorrect ou wallet introuvable"}),
            ),
        ),
    }
}

async fn post_forge_block(State(state): State<ApiState>) -> Json<serde_json::Value> {
    let mut bc = state.blockchain.lock().await;
    let mut ledger = state.ledger.lock().await;
    let mut validators = state.validators.lock().await;
    bc.add_block(vec![], &mut validators, &mut ledger);
    Json(serde_json::json!({"ok": true, "height": bc.chain.len()}))
}

#[derive(serde::Deserialize)]
pub struct ForgeHadronRequest {
    pub address: String,
}

async fn post_forge_hadron(
    State(state): State<ApiState>,
    Json(req): Json<ForgeHadronRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut validators = state.validators.lock().await;
    let Some(v) = validators.iter_mut().find(|v| v.address == req.address) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"ok": false, "error": "validateur inconnu"})),
        );
    };
    if v.quarks_reward.len() < 3 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"ok": false, "error": "pas assez de quarks"})),
        );
    }
    let selected: Vec<_> = v.quarks_reward.drain(0..3).collect();
    match crate::quark::forge_hadron(selected) {
        Some(h) => {
            let kind = h.kind.clone();
            v.staked_hadrons.push(h);
            (
                StatusCode::OK,
                Json(serde_json::json!({"ok": true, "kind": kind})),
            )
        }
        None => (
            StatusCode::OK,
            Json(serde_json::json!({"ok": false, "error": "combinaison invalide, quarks perdus"})),
        ),
    }
}
