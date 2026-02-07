// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// EIP-191: personal_sign (prefixed message signing)

use k256::ecdsa::{SigningKey, signature::hazmat::PrehashSigner};
use super::keccak256;

/// Sign a message with EIP-191 prefix.
///
/// Computes: keccak256("\x19Ethereum Signed Message:\n" + len(message) + message)
/// Returns 65 bytes: r (32) + s (32) + v (1, value 27 or 28)
pub fn personal_sign(message: &[u8], private_key: &[u8; 32]) -> Result<[u8; 65], String> {
    let hash = personal_sign_hash(message);

    let signing_key = SigningKey::from_bytes(private_key.into())
        .map_err(|e| format!("Chiave non valida: {}", e))?;
    let (signature, recovery_id) = signing_key
        .sign_prehash(&hash)
        .map_err(|e| format!("Errore firma: {}", e))?;

    let sig_bytes = signature.to_bytes();
    let mut result = [0u8; 65];
    result[..32].copy_from_slice(&sig_bytes[..32]); // r
    result[32..64].copy_from_slice(&sig_bytes[32..]); // s
    result[64] = recovery_id.to_byte() + 27; // v
    Ok(result)
}

/// Compute the EIP-191 hash without signing (useful for verification)
pub fn personal_sign_hash(message: &[u8]) -> [u8; 32] {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut data = Vec::with_capacity(prefix.len() + message.len());
    data.extend_from_slice(prefix.as_bytes());
    data.extend_from_slice(message);
    keccak256(&data)
}

/// Recover the signer address from a personal_sign signature
pub fn recover_address(message: &[u8], signature: &[u8; 65]) -> Result<[u8; 20], String> {
    use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

    let hash = personal_sign_hash(message);
    let v = signature[64];
    let recovery_id = RecoveryId::try_from(v.wrapping_sub(27))
        .map_err(|e| format!("Recovery ID non valido: {}", e))?;

    let sig = Signature::from_slice(&signature[..64])
        .map_err(|e| format!("Firma non valida: {}", e))?;

    let verifying_key = VerifyingKey::recover_from_prehash(&hash, &sig, recovery_id)
        .map_err(|e| format!("Recovery fallito: {}", e))?;

    // Public key → keccak256 → last 20 bytes = address
    let pubkey_bytes = verifying_key.to_encoded_point(false);
    let pubkey_uncompressed = &pubkey_bytes.as_bytes()[1..]; // skip 0x04 prefix
    let addr_hash = keccak256(pubkey_uncompressed);
    let mut address = [0u8; 20];
    address.copy_from_slice(&addr_hash[12..]);
    Ok(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test private key (DO NOT use in production)
    const TEST_KEY: [u8; 32] = [
        0xac, 0x09, 0x74, 0xbe, 0xc3, 0x9a, 0x17, 0xe3,
        0x6b, 0xa4, 0xa6, 0xb4, 0xd2, 0x38, 0xff, 0x94,
        0x4b, 0xac, 0xb3, 0x78, 0x50, 0x0f, 0xa5, 0x5e,
        0x4c, 0x4d, 0x47, 0x1b, 0x52, 0x6a, 0x98, 0x52,
    ];

    #[test]
    fn test_personal_sign_roundtrip() {
        let message = b"Hello, Rusby!";
        let sig = personal_sign(message, &TEST_KEY).unwrap();
        assert_eq!(sig.len(), 65);
        assert!(sig[64] == 27 || sig[64] == 28);

        // Recover address and verify it matches
        let recovered = recover_address(message, &sig).unwrap();
        // Sign again — should recover same address
        let sig2 = personal_sign(message, &TEST_KEY).unwrap();
        let recovered2 = recover_address(message, &sig2).unwrap();
        assert_eq!(recovered, recovered2);
    }

    #[test]
    fn test_personal_sign_empty_message() {
        let sig = personal_sign(b"", &TEST_KEY).unwrap();
        assert_eq!(sig.len(), 65);
        let recovered = recover_address(b"", &sig).unwrap();
        // Should still recover a valid address
        assert_ne!(recovered, [0u8; 20]);
    }

    #[test]
    fn test_personal_sign_utf8_message() {
        let message = "Ciao mondo! 🦀 Rust è fantastico".as_bytes();
        let sig = personal_sign(message, &TEST_KEY).unwrap();
        let recovered = recover_address(message, &sig).unwrap();
        assert_ne!(recovered, [0u8; 20]);
    }

    #[test]
    fn test_personal_sign_hash_deterministic() {
        let h1 = personal_sign_hash(b"test");
        let h2 = personal_sign_hash(b"test");
        assert_eq!(h1, h2);
        // Different message → different hash
        let h3 = personal_sign_hash(b"test2");
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_personal_sign_different_messages() {
        let sig1 = personal_sign(b"message1", &TEST_KEY).unwrap();
        let sig2 = personal_sign(b"message2", &TEST_KEY).unwrap();
        // Signatures should differ
        assert_ne!(sig1[..64], sig2[..64]);
        // But both recover to the same address
        let addr1 = recover_address(b"message1", &sig1).unwrap();
        let addr2 = recover_address(b"message2", &sig2).unwrap();
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_recover_address_wrong_message() {
        let sig = personal_sign(b"correct message", &TEST_KEY).unwrap();
        let addr_correct = recover_address(b"correct message", &sig).unwrap();
        let addr_wrong = recover_address(b"wrong message", &sig).unwrap();
        // Wrong message should recover a DIFFERENT address
        assert_ne!(addr_correct, addr_wrong);
    }
}
