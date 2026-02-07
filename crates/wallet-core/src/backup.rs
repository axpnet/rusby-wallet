// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// backup: Export/Import encrypted wallet backups
//
// Types:
//   BackupPayload — JSON container with version, app name, timestamp, encrypted data
// Functions:
//   export_backup()   — Encrypt wallet JSON → base64 → BackupPayload JSON
//   import_backup()   — BackupPayload JSON → base64-decode → decrypt → wallet JSON
//   validate_backup() — Check format without decrypting

use serde::{Deserialize, Serialize};
use crate::crypto;

const BACKUP_VERSION: u8 = 1;
const APP_NAME: &str = "rusby-wallet";

/// Backup payload — serialized as .rusby JSON file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPayload {
    pub version: u8,
    pub app: String,
    pub created_at: u64,
    pub encrypted_data: String,
}

/// Export wallet data as an encrypted backup
///
/// 1. Encrypts `wallet_json` with AES-256-GCM using `password`
/// 2. Serializes EncryptedData to JSON, then base64-encodes it
/// 3. Wraps in BackupPayload with metadata
pub fn export_backup(wallet_json: &str, password: &str) -> Result<String, String> {
    let encrypted = crypto::encrypt(wallet_json.as_bytes(), password)?;
    let encrypted_json = serde_json::to_string(&encrypted)
        .map_err(|e| format!("Serialization error: {}", e))?;
    let encoded = base64_encode(encrypted_json.as_bytes());

    let payload = BackupPayload {
        version: BACKUP_VERSION,
        app: APP_NAME.to_string(),
        created_at: current_timestamp(),
        encrypted_data: encoded,
    };

    serde_json::to_string_pretty(&payload)
        .map_err(|e| format!("Serialization error: {}", e))
}

/// Import wallet data from an encrypted backup
///
/// 1. Parses BackupPayload JSON
/// 2. Validates version and app name
/// 3. Base64-decodes → EncryptedData → AES-256-GCM decrypt
/// 4. Returns the original wallet JSON
pub fn import_backup(backup_json: &str, password: &str) -> Result<String, String> {
    let payload = validate_backup(backup_json)?;

    let encrypted_json_bytes = base64_decode(&payload.encrypted_data)?;
    let encrypted_json = String::from_utf8(encrypted_json_bytes)
        .map_err(|e| format!("Invalid UTF-8 in backup: {}", e))?;

    let encrypted: crypto::EncryptedData = serde_json::from_str(&encrypted_json)
        .map_err(|e| format!("Invalid encrypted data: {}", e))?;

    let decrypted = crypto::decrypt(&encrypted, password)?;
    String::from_utf8(decrypted)
        .map_err(|e| format!("Invalid UTF-8 in decrypted data: {}", e))
}

/// Validate backup format without decrypting
pub fn validate_backup(backup_json: &str) -> Result<BackupPayload, String> {
    let payload: BackupPayload = serde_json::from_str(backup_json)
        .map_err(|e| format!("Invalid backup format: {}", e))?;

    if payload.app != APP_NAME {
        return Err(format!("Not a Rusby Wallet backup (app: {})", payload.app));
    }
    if payload.version > BACKUP_VERSION {
        return Err(format!(
            "Backup version {} not supported (max: {})",
            payload.version, BACKUP_VERSION
        ));
    }

    Ok(payload)
}

/// Simple base64 encoder (no external dependency needed)
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Simple base64 decoder
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    fn char_to_val(c: u8) -> Result<u8, String> {
        match c {
            b'A'..=b'Z' => Ok(c - b'A'),
            b'a'..=b'z' => Ok(c - b'a' + 26),
            b'0'..=b'9' => Ok(c - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            b'=' => Ok(0),
            _ => Err(format!("Invalid base64 character: {}", c as char)),
        }
    }

    let bytes: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if bytes.len() % 4 != 0 {
        return Err("Invalid base64 length".into());
    }

    let mut result = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let a = char_to_val(chunk[0])? as u32;
        let b = char_to_val(chunk[1])? as u32;
        let c = char_to_val(chunk[2])? as u32;
        let d = char_to_val(chunk[3])? as u32;
        let triple = (a << 18) | (b << 12) | (c << 6) | d;
        result.push(((triple >> 16) & 0xFF) as u8);
        if chunk[2] != b'=' {
            result.push(((triple >> 8) & 0xFF) as u8);
        }
        if chunk[3] != b'=' {
            result.push((triple & 0xFF) as u8);
        }
    }
    Ok(result)
}

fn current_timestamp() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, Rusby Wallet!";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_empty() {
        let encoded = base64_encode(b"");
        let decoded = base64_decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_export_import_roundtrip() {
        let wallet_json = r#"{"wallets":[],"active_index":0}"#;
        let password = "test_password_123!";

        let backup = export_backup(wallet_json, password).unwrap();
        let restored = import_backup(&backup, password).unwrap();

        assert_eq!(restored, wallet_json);
    }

    #[test]
    fn test_import_wrong_password() {
        let wallet_json = r#"{"wallets":[]}"#;
        let backup = export_backup(wallet_json, "correct").unwrap();
        let result = import_backup(&backup, "wrong");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_backup() {
        let wallet_json = r#"{"test": true}"#;
        let backup = export_backup(wallet_json, "pass").unwrap();
        let payload = validate_backup(&backup).unwrap();
        assert_eq!(payload.version, 1);
        assert_eq!(payload.app, "rusby-wallet");
    }

    #[test]
    fn test_validate_invalid_json() {
        let result = validate_backup("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_wrong_app() {
        let json = r#"{"version":1,"app":"other-wallet","created_at":0,"encrypted_data":""}"#;
        let result = validate_backup(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not a Rusby Wallet"));
    }

    #[test]
    fn test_validate_future_version() {
        let json = r#"{"version":99,"app":"rusby-wallet","created_at":0,"encrypted_data":""}"#;
        let result = validate_backup(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not supported"));
    }
}
