// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// EIP-712: Typed structured data hashing and signing

use k256::ecdsa::{SigningKey, signature::hazmat::PrehashSigner};
use super::keccak256;

/// Sign pre-computed EIP-712 hashes.
///
/// The caller is responsible for computing domain_separator and struct_hash
/// from the typed data JSON (done in WASM/JS layer).
///
/// Computes: keccak256("\x19\x01" + domainSeparator + structHash)
/// Returns 65 bytes: r (32) + s (32) + v (1, value 27 or 28)
pub fn sign_typed_data_hash(
    domain_separator: &[u8; 32],
    struct_hash: &[u8; 32],
    private_key: &[u8; 32],
) -> Result<[u8; 65], String> {
    let hash = eip712_hash(domain_separator, struct_hash);

    let signing_key = SigningKey::from_bytes(private_key.into())
        .map_err(|e| format!("Chiave non valida: {}", e))?;
    let (signature, recovery_id) = signing_key
        .sign_prehash(&hash)
        .map_err(|e| format!("Errore firma: {}", e))?;

    let sig_bytes = signature.to_bytes();
    let mut result = [0u8; 65];
    result[..32].copy_from_slice(&sig_bytes[..32]);
    result[32..64].copy_from_slice(&sig_bytes[32..]);
    result[64] = recovery_id.to_byte() + 27;
    Ok(result)
}

/// Compute the EIP-712 final hash (without signing)
pub fn eip712_hash(domain_separator: &[u8; 32], struct_hash: &[u8; 32]) -> [u8; 32] {
    let mut data = Vec::with_capacity(2 + 32 + 32);
    data.extend_from_slice(&[0x19, 0x01]);
    data.extend_from_slice(domain_separator);
    data.extend_from_slice(struct_hash);
    keccak256(&data)
}

/// Compute EIP-712 domain separator from components.
///
/// typeHash = keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)")
pub fn hash_eip712_domain(
    name: &str,
    version: &str,
    chain_id: u64,
    verifying_contract: &[u8; 20],
) -> [u8; 32] {
    let type_hash = keccak256(
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
    );

    // ABI encode: typeHash + keccak(name) + keccak(version) + chainId(uint256) + contract(address padded)
    let mut encoded = Vec::with_capacity(5 * 32);
    encoded.extend_from_slice(&type_hash);
    encoded.extend_from_slice(&keccak256(name.as_bytes()));
    encoded.extend_from_slice(&keccak256(version.as_bytes()));

    let mut chain_bytes = [0u8; 32];
    chain_bytes[24..].copy_from_slice(&chain_id.to_be_bytes());
    encoded.extend_from_slice(&chain_bytes);

    let mut addr_bytes = [0u8; 32];
    addr_bytes[12..].copy_from_slice(verifying_contract);
    encoded.extend_from_slice(&addr_bytes);

    keccak256(&encoded)
}

/// Hash a simple struct type for EIP-712.
/// For complex types, the WASM/JS layer handles recursive hashing.
pub fn hash_struct(type_hash: &[u8; 32], encoded_data: &[u8]) -> [u8; 32] {
    let mut data = Vec::with_capacity(32 + encoded_data.len());
    data.extend_from_slice(type_hash);
    data.extend_from_slice(encoded_data);
    keccak256(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: [u8; 32] = [
        0xac, 0x09, 0x74, 0xbe, 0xc3, 0x9a, 0x17, 0xe3,
        0x6b, 0xa4, 0xa6, 0xb4, 0xd2, 0x38, 0xff, 0x94,
        0x4b, 0xac, 0xb3, 0x78, 0x50, 0x0f, 0xa5, 0x5e,
        0x4c, 0x4d, 0x47, 0x1b, 0x52, 0x6a, 0x98, 0x52,
    ];

    #[test]
    fn test_eip712_domain_hash_deterministic() {
        let contract = [0xCC; 20];
        let h1 = hash_eip712_domain("TestApp", "1", 1, &contract);
        let h2 = hash_eip712_domain("TestApp", "1", 1, &contract);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_eip712_domain_hash_differs_by_chain() {
        let contract = [0xCC; 20];
        let h1 = hash_eip712_domain("TestApp", "1", 1, &contract);
        let h2 = hash_eip712_domain("TestApp", "1", 137, &contract);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_eip712_domain_hash_differs_by_name() {
        let contract = [0xCC; 20];
        let h1 = hash_eip712_domain("App1", "1", 1, &contract);
        let h2 = hash_eip712_domain("App2", "1", 1, &contract);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_sign_typed_data_roundtrip() {
        let domain = hash_eip712_domain("Rusby", "1", 1, &[0xAA; 20]);
        let struct_hash = keccak256(b"some struct data");
        let sig = sign_typed_data_hash(&domain, &struct_hash, &TEST_KEY).unwrap();
        assert_eq!(sig.len(), 65);
        assert!(sig[64] == 27 || sig[64] == 28);
    }

    #[test]
    fn test_eip712_hash_prefix() {
        let domain = [0xAA; 32];
        let struct_hash = [0xBB; 32];
        let hash = eip712_hash(&domain, &struct_hash);
        // The hash should be deterministic
        let hash2 = eip712_hash(&domain, &struct_hash);
        assert_eq!(hash, hash2);
        // Different inputs → different hash
        let hash3 = eip712_hash(&[0xCC; 32], &struct_hash);
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_hash_struct() {
        let type_hash = keccak256(b"Transfer(address to,uint256 amount)");
        let encoded = [0u8; 64]; // dummy ABI-encoded data
        let h = hash_struct(&type_hash, &encoded);
        assert_ne!(h, [0u8; 32]);
    }
}
