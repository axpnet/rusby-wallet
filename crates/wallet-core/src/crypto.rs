// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// crypto: AES-256-GCM symmetric encryption with PBKDF2 key derivation
//
// Types:
//   EncryptedData  — Container for salt + nonce + ciphertext
// Functions:
//   encrypt()      — Encrypt plaintext with password (random salt + nonce)
//   decrypt()      — Decrypt ciphertext with password

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use zeroize::Zeroize;

// PBKDF2-HMAC-SHA256: 600,000 iterations (OWASP 2023 recommendation).
// In release WASM (~3-5x slower than native), completes in ~1-3 seconds.
// UI stays responsive: encrypt/decrypt runs inside Timeout callbacks
// (login.rs Phase 1, onboarding.rs Phase 2), yielding to the browser event loop.
const PBKDF2_ITERATIONS: u32 = 600_000;
const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

/// Encrypted data container
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct EncryptedData {
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

/// Custom Debug: redacts ciphertext to prevent leaking sensitive data in logs
impl std::fmt::Debug for EncryptedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptedData")
            .field("salt", &format!("[{} bytes]", self.salt.len()))
            .field("nonce", &format!("[{} bytes]", self.nonce.len()))
            .field("ciphertext", &format!("[{} bytes REDACTED]", self.ciphertext.len()))
            .finish()
    }
}

/// Derive an AES-256 key from password using PBKDF2-HMAC-SHA256
/// Returns a key that must be zeroized after use
fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

/// Encrypt data with AES-256-GCM using a password
pub fn encrypt(plaintext: &[u8], password: &str) -> Result<EncryptedData, String> {
    let mut salt = vec![0u8; SALT_LEN];
    let mut nonce_bytes = vec![0u8; NONCE_LEN];

    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut nonce_bytes);

    let mut key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| { key.zeroize(); format!("Cipher init error: {}", e) })?;
    key.zeroize();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption error: {}", e))?;

    Ok(EncryptedData {
        salt,
        nonce: nonce_bytes,
        ciphertext,
    })
}

/// Decrypt data with AES-256-GCM using a password
pub fn decrypt(encrypted: &EncryptedData, password: &str) -> Result<Vec<u8>, String> {
    if encrypted.nonce.len() != NONCE_LEN {
        return Err(format!("Invalid nonce length: expected {}, got {}", NONCE_LEN, encrypted.nonce.len()));
    }
    if encrypted.salt.len() != SALT_LEN {
        return Err(format!("Invalid salt length: expected {}, got {}", SALT_LEN, encrypted.salt.len()));
    }
    let mut key = derive_key(password, &encrypted.salt);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| { key.zeroize(); format!("Cipher init error: {}", e) })?;
    key.zeroize();
    let nonce = Nonce::from_slice(&encrypted.nonce);

    cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|_| "Decryption failed: wrong password or corrupted data".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let data = b"my secret seed phrase";
        let password = "strongpassword123";

        let encrypted = encrypt(data, password).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();

        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_wrong_password_fails() {
        let data = b"secret data";
        let encrypted = encrypt(data, "correct_password").unwrap();
        let result = decrypt(&encrypted, "wrong_password");
        assert!(result.is_err());
    }

    #[test]
    fn test_different_encryptions_different_output() {
        let data = b"same data";
        let password = "same_password";

        let enc1 = encrypt(data, password).unwrap();
        let enc2 = encrypt(data, password).unwrap();

        // Different salt/nonce means different ciphertext
        assert_ne!(enc1.ciphertext, enc2.ciphertext);

        // But both decrypt to same plaintext
        assert_eq!(decrypt(&enc1, password).unwrap(), data);
        assert_eq!(decrypt(&enc2, password).unwrap(), data);
    }

    #[test]
    fn test_serialization() {
        let data = b"test data";
        let encrypted = encrypt(data, "pass").unwrap();

        let json = serde_json::to_string(&encrypted).unwrap();
        let deserialized: EncryptedData = serde_json::from_str(&json).unwrap();

        let decrypted = decrypt(&deserialized, "pass").unwrap();
        assert_eq!(decrypted, data);
    }
}
