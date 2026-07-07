mod block;
mod ledger;
mod network;
mod quark;
mod transaction;
mod validator;
mod wallet;

use std::sync::Arc;
use tokio::sync::Mutex;

use block::Blockchain;
use ledger::Ledger;
use network::{fetch_peers, start_api, start_server};
use transaction::Transaction;
use validator::Validator;

#[tokio::main]
async fn main() {
    let ledger = Arc::new(Mutex::new(Ledger::new()));
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let validators: Arc<Mutex<Vec<Validator>>> = Arc::new(Mutex::new(vec![]));
    let mempool: Arc<Mutex<Vec<Transaction>>> = Arc::new(Mutex::new(vec![]));

    let mut peers: Vec<String> = vec!["127.0.0.1:8081".to_string()];
    if let Some(discovered) = fetch_peers("127.0.0.1:8081").await {
        for p in discovered {
            if !peers.contains(&p) {
                peers.push(p);
            }
        }
    }

    let peers_shared = Arc::new(Mutex::new(peers));

    println!("Nœud Quark démarré.");
    println!("Serveur P2P : 127.0.0.1:8080");
    println!("API + dashboard : http://127.0.0.1:3000");

    let server = tokio::spawn(start_server(
        8080,
        blockchain.clone(),
        "chain.json".to_string(),
    ));
    let api = tokio::spawn(start_api(
        3000,
        blockchain,
        peers_shared,
        ledger,
        validators,
        mempool,
    ));

    let _ = tokio::join!(server, api);
}
