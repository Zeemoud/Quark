use crate::transaction::Transaction;
use std::collections::HashMap;

pub struct Ledger {
    pub balances: HashMap<String, u64>,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            balances: HashMap::new(),
        }
    }

    pub fn apply_transactions(&mut self, tx: &Transaction) -> bool {
        if !tx.verify() {
            return false;
        }
        let total = tx.amount + tx.fee;
        let from_balance = *self.balances.get(&tx.from).unwrap_or(&0);
        if from_balance < total {
            return false;
        }
        *self.balances.entry(tx.from.clone()).or_insert(0) -= total;
        *self.balances.entry(tx.to.clone()).or_insert(0) += tx.amount;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;
    use crate::wallet::Wallet;

    #[test]
    fn transfer_updates_balances() {
        let wallet = Wallet::new();
        let addr = wallet.public_key_hex();
        let mut ledger = Ledger::new();
        ledger.balances.insert(addr.clone(), 1000);
        let tx = Transaction::new_signed(&wallet, "dest".into(), 500, 10);
        assert!(ledger.apply_transactions(&tx));
        assert_eq!(ledger.balances[&addr], 490);
        assert_eq!(ledger.balances["dest"], 500);
    }

    #[test]
    fn insufficient_balance_fails() {
        let wallet = Wallet::new();
        let addr = wallet.public_key_hex();
        let mut ledger = Ledger::new();
        ledger.balances.insert(addr.clone(), 10);
        let tx = Transaction::new_signed(&wallet, "dest".into(), 500, 10);
        assert!(!ledger.apply_transactions(&tx));
    }
}
