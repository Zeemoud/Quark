use crate::block::{Block, Blockchain};
use axum::{Json, Router, extract::State, routing::get};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub async fn start_server(port: u16, blockchain: Arc<Mutex<Blockchain>>, chain_path: String) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
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
}

pub async fn start_api(
    port: u16,
    blockchain: Arc<Mutex<Blockchain>>,
    peers: Arc<Mutex<Vec<String>>>,
) {
    let state = ApiState { blockchain, peers };
    let app = Router::new()
        .route("/chain", get(get_chain))
        .route("/", get(explorer))
        .route("/peers", get(get_peers))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
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
