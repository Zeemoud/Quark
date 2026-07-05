mod block;
mod ledger;
mod network;
mod quark;
mod transaction;
mod validator;
mod wallet;

use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

use block::Blockchain;
use ledger::Ledger;
use network::{broadcast, fetch_chain, fetch_peers, start_api, start_server};
use quark::{QuarkType, forge_hadron};
use transaction::Transaction;
use validator::Validator;
use wallet::Wallet;

#[tokio::main]
async fn main() {
    let mut ledger = Ledger::new();
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let mut wallets: HashMap<String, Wallet> = HashMap::new();
    let mut validators: Vec<Validator> = vec![];
    let mut peers: Vec<String> = vec!["127.0.0.1:8081".to_string()];
    if let Some(discovered) = fetch_peers("127.0.0.1:8081").await {
        for p in discovered {
            if !peers.contains(&p) {
                peers.push(p);
            }
        }
    }

    tokio::spawn(start_server(
        8080,
        blockchain.clone(),
        "chain.json".to_string(),
    ));

    let peers_shared = Arc::new(Mutex::new(peers.clone()));
    tokio::spawn(start_api(3000, blockchain.clone(), peers_shared));

    loop {
        println!(
            "\n1. Créer wallet\n2. Voir Solde\n3. Envoyer transaction\n4. Forger bloc\n5. Forger Hadron\n6. Quitter\n7. Sync avec un pair"
        );
        print!("> ");
        io::stdout().flush().unwrap();
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                print!("Mot de passe: ");
                io::stdout().flush().unwrap();
                let mut password = String::new();
                io::stdin().read_line(&mut password).unwrap();
                let password = password.trim();

                let wallet = Wallet::new();
                let addr = wallet.public_key_hex();
                std::fs::create_dir_all("wallets").unwrap();
                wallet.save_encrypted(&format!("wallets/{}.key", addr), password);
                ledger.balances.insert(addr.clone(), 1000);
                wallets.insert(addr.clone(), wallet);
                validators.push(Validator {
                    address: addr.clone(),
                    staked_hadrons: vec![],
                    quarks_reward: vec![],
                    seed: rand::random(),
                });
                println!("Wallet créé: {}", addr);
            }
            "2" => {
                print!("Adresse: ");
                io::stdout().flush().unwrap();
                let mut addr = String::new();
                io::stdin().read_line(&mut addr).unwrap();
                let addr = addr.trim();
                println!("Solde: {}", ledger.balances.get(addr).unwrap_or(&0));
                if let Some(v) = validators.iter().find(|v| v.address == addr) {
                    println!("Quarks: {:?}", v.quarks_reward);
                }
            }
            "3" => {
                print!("Adresse expéditeur: ");
                io::stdout().flush().unwrap();
                let mut from = String::new();
                io::stdin().read_line(&mut from).unwrap();
                let from = from.trim().to_string();

                print!("Adresse destinataire: ");
                io::stdout().flush().unwrap();
                let mut to = String::new();
                io::stdin().read_line(&mut to).unwrap();
                let to = to.trim().to_string();

                print!("Montant: ");
                io::stdout().flush().unwrap();
                let mut amount = String::new();
                io::stdin().read_line(&mut amount).unwrap();
                let amount: u64 = amount.trim().parse().unwrap_or(0);

                print!("Frais: ");
                io::stdout().flush().unwrap();
                let mut fee = String::new();
                io::stdin().read_line(&mut fee).unwrap();
                let fee: u64 = fee.trim().parse().unwrap_or(0);

                if let Some(wallet) = wallets.get(&from) {
                    let tx = Transaction::new_signed(wallet, to, amount, fee);
                    blockchain
                        .lock()
                        .await
                        .add_block(vec![tx], &mut validators, &mut ledger);
                    broadcast(&peers, &blockchain.lock().await.chain).await;
                    println!("Transaction envoyée et bloc forgé.");
                } else {
                    println!("Wallet inconnu.");
                }
            }
            "4" => {
                blockchain
                    .lock()
                    .await
                    .add_block(vec![], &mut validators, &mut ledger);
                broadcast(&peers, &blockchain.lock().await.chain).await;
                println!("Bloc vide forgé.");
            }
            "5" => {
                print!("Adresse: ");
                io::stdout().flush().unwrap();
                let mut addr = String::new();
                io::stdin().read_line(&mut addr).unwrap();
                let addr = addr.trim().to_string();

                if let Some(v) = validators.iter_mut().find(|v| v.address == addr) {
                    if v.quarks_reward.len() < 3 {
                        println!("Pas assez de quarks.");
                    } else {
                        let selected: Vec<QuarkType> = v.quarks_reward.drain(0..3).collect();
                        match forge_hadron(selected.clone()) {
                            Some(h) => {
                                println!("Hadron forgé: {:?}", h);
                                v.staked_hadrons.push(h);
                            }
                            None => println!("Combinaison invalide, quarks perdus."),
                        }
                    }
                } else {
                    println!("Validateur inconnu.");
                }
            }
            "6" => {
                blockchain.lock().await.save("chain.json");
                break;
            }
            "7" => {
                print!("Adresse (ex 127.0.0.1:8080): ");
                io::stdout().flush().unwrap();
                let mut addr = String::new();
                io::stdin().read_line(&mut addr).unwrap();
                let addr = addr.trim();
                match fetch_chain(addr).await {
                    Some(chain) => {
                        blockchain.lock().await.chain = chain;
                        println!(
                            "Chain synchronisée, {} blocs",
                            blockchain.lock().await.chain.len()
                        );
                    }
                    None => println!("Échec de connexion."),
                }
            }
            _ => println!("Choix invalide."),
        }
    }
}
