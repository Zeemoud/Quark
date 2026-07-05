use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::Argon2;
use ed25519_dalek::{Signature, Signer, SigningKey};
use rand::rngs::OsRng;
use std::fs;

pub struct Wallet {
    pub signing_key: SigningKey,
}

impl Wallet {
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        Wallet { signing_key }
    }

    pub fn public_key_hex(&self) -> String {
        bs58::encode(self.signing_key.verifying_key().to_bytes()).into_string()
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    pub fn save_encrypted(&self, path: &str, password: &str) {
        let salt = rand::random::<[u8; 16]>();
        let mut key_bytes = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), &salt, &mut key_bytes)
            .unwrap();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce_bytes = rand::random::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, self.signing_key.to_bytes().as_ref())
            .unwrap();

        let mut data = Vec::new();
        data.extend_from_slice(&salt);
        data.extend_from_slice(&nonce_bytes);
        data.extend_from_slice(&ciphertext);
        fs::write(path, data).unwrap();
    }

    pub fn load_encrypted(path: &str, password: &str) -> Option<Self> {
        let data = fs::read(path).ok()?;
        let salt = &data[0..16];
        let nonce_bytes = &data[16..28];
        let ciphertext = &data[28..];

        let mut key_bytes = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), salt, &mut key_bytes)
            .ok()?;
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher.decrypt(nonce, ciphertext).ok()?;
        let arr: [u8; 32] = plaintext.try_into().ok()?;
        Some(Wallet {
            signing_key: SigningKey::from_bytes(&arr),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let wallet = Wallet::new();
        let addr = wallet.public_key_hex();
        let path = format!("test_{}.key", addr);
        wallet.save_encrypted(&path, "password123");
        let loaded = Wallet::load_encrypted(&path, "password123").unwrap();
        assert_eq!(loaded.public_key_hex(), addr);
        fs::remove_file(&path).ok();
    }

    #[test]
    fn wrong_password_fails() {
        let wallet = Wallet::new();
        let addr = wallet.public_key_hex();
        let path = format!("test2_{}.key", addr);
        wallet.save_encrypted(&path, "password123");
        let loaded = Wallet::load_encrypted(&path, "wrongpass");
        assert!(loaded.is_none());
        fs::remove_file(&path).ok();
    }
}
