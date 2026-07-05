use crate::wallet::Wallet;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub signature: Option<Vec<u8>>,
}

impl Transaction {
    pub fn new_signed(wallet: &Wallet, to: String, amount: u64, fee: u64) -> Self {
        let from = wallet.public_key_hex();
        let message = format!("{}{}{}{}", from, to, amount, fee);
        let signature = wallet.sign(message.as_bytes()).to_bytes().to_vec();
        Transaction {
            from,
            to,
            amount,
            fee,
            signature: Some(signature),
        }
    }

    pub fn verify(&self) -> bool {
        let Some(sig_bytes) = &self.signature else {
            return false;
        };
        let Ok(pub_bytes) = bs58::decode(&self.from).into_vec() else {
            return false;
        };
        let Ok(pub_bytes_arr): Result<[u8; 32], _> = pub_bytes.try_into() else {
            return false;
        };
        let Ok(verifying_key) = VerifyingKey::from_bytes(&pub_bytes_arr) else {
            return false;
        };
        let Ok(sig_arr): Result<[u8; 64], _> = sig_bytes.clone().try_into() else {
            return false;
        };
        let signature = Signature::from_bytes(&sig_arr);
        let message = format!("{}{}{}{}", self.from, self.to, self.amount, self.fee);
        verifying_key.verify(message.as_bytes(), &signature).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::Wallet;

    #[test]
    fn valid_signature_passes() {
        let wallet = Wallet::new();
        let tx = Transaction::new_signed(&wallet, "dest".into(), 100, 1);
        assert!(tx.verify());
    }

    #[test]
    fn tampered_amount_fails() {
        let wallet = Wallet::new();
        let mut tx = Transaction::new_signed(&wallet, "dest".into(), 100, 1);
        tx.amount = 999999;
        assert!(!tx.verify());
    }

    #[test]
    fn wrong_sender_fails() {
        let wallet = Wallet::new();
        let other = Wallet::new();
        let mut tx = Transaction::new_signed(&wallet, "dest".into(), 100, 1);
        tx.from = other.public_key_hex();
        assert!(!tx.verify());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::wallet::Wallet;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn verify_never_panics(amount in any::<u64>(), fee in any::<u64>(), to in ".*") {
            let wallet = Wallet::new();
            let tx = Transaction::new_signed(&wallet, to, amount, fee);
            let _ = tx.verify();
        }

        #[test]
        fn tampered_amount_never_verifies(amount in any::<u64>(), fee in any::<u64>(), bad_amount in any::<u64>()) {
            let wallet = Wallet::new();
            let mut tx = Transaction::new_signed(&wallet, "dest".into(), amount, fee);
            if bad_amount != amount {
                tx.amount = bad_amount;
                assert!(!tx.verify());
            }
        }
    }
}
