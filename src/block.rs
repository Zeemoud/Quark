use crate::ledger::Ledger;
use crate::quark::random_quark;
use crate::transaction::Transaction;
use crate::validator::{Validator, select_validator};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub validator: String,
}

impl Block {
    pub fn new(
        index: u64,
        transactions: Vec<Transaction>,
        previous_hash: String,
        validator: String,
    ) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            validator,
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{:?}{}{}",
            self.index, self.timestamp, self.transactions, self.previous_hash, self.validator
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block::new(0, vec![], "0".to_string(), "genesis".to_string());
        Blockchain {
            chain: vec![genesis],
        }
    }

    pub fn latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    pub fn add_block(
        &mut self,
        transactions: Vec<Transaction>,
        validators: &mut [Validator],
        ledger: &mut Ledger,
    ) {
        for tx in &transactions {
            ledger.apply_transactions(tx);
        }
        let previous_hash_for_selection = self.latest_block().hash.clone();
        let validator_addr = select_validator(validators, &previous_hash_for_selection)
            .map(|v| v.address.clone())
            .unwrap_or("none".to_string());

        let halvings = self.chain.len() as u64 / 10;
        let reward = 1000u64 >> halvings.min(20);

        if let Some(v) = validators.iter_mut().find(|v| v.address == validator_addr) {
            for _ in 0..3 {
                v.quarks_reward.push(random_quark());
            }
        }
        if validator_addr != "none" {
            *ledger.balances.entry(validator_addr.clone()).or_insert(0) += reward;
        }

        let previous_hash = self.latest_block().hash.clone();
        let index = self.latest_block().index + 1;
        let block = Block::new(index, transactions, previous_hash, validator_addr);
        self.chain.push(block);
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];
            if current.previous_hash != previous.hash {
                return false;
            }
            if current.hash != current.calculate_hash() {
                return false;
            }
        }
        true
    }

    pub fn save(&self, path: &str) {
        let json = serde_json::to_string_pretty(&self.chain).unwrap();
        fs::write(path, json).unwrap();
    }

    pub fn load(path: &str) -> Self {
        let json = fs::read_to_string(path).unwrap();
        let chain: Vec<Block> = serde_json::from_str(&json).unwrap();
        Blockchain { chain }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::Ledger;
    use crate::validator::Validator;

    #[test]
    fn genesis_is_valid() {
        let bc = Blockchain::new();
        assert!(bc.is_valid());
        assert_eq!(bc.chain.len(), 1);
    }

    #[test]
    fn add_block_increases_length() {
        let mut bc = Blockchain::new();
        let mut ledger = Ledger::new();
        let mut validators = vec![Validator::new("v1".into(), 42)];
        bc.add_block(vec![], &mut validators, &mut ledger);
        assert_eq!(bc.chain.len(), 2);
        assert!(bc.is_valid());
    }

    #[test]
    fn tampered_chain_is_invalid() {
        let mut bc = Blockchain::new();
        let mut ledger = Ledger::new();
        let mut validators = vec![Validator::new("v1".into(), 42)];
        bc.add_block(vec![], &mut validators, &mut ledger);
        bc.chain[1].previous_hash = "fake".to_string();
        assert!(!bc.is_valid());
    }
}
