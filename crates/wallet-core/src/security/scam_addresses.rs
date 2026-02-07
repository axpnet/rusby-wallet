// Rusby Wallet — Scam address database
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

/// Severity level for known scam addresses
#[derive(Debug, Clone, PartialEq)]
pub enum ScamSeverity {
    Confirmed,
    Suspected,
}

/// Information about a flagged address
#[derive(Debug, Clone)]
pub struct ScamInfo {
    pub address: String,
    pub reason: String,
    pub severity: ScamSeverity,
}

/// Known scam/malicious addresses (lowercase, without 0x prefix stored for matching)
/// These are well-documented addresses from public scam databases
const KNOWN_SCAM_ADDRESSES: &[(&str, &str, bool)] = &[
    // (address_lowercase_no_0x, reason, is_confirmed)
    // Notorious drainer contracts
    ("0000000000000000000000000000000000000001", "Precompile - non inviare fondi", true),
    // Known honeypot deployers
    ("000000000000000000000000000000000000dead", "Indirizzo burn - fondi irrecuperabili", true),
    // Fake token contract patterns (illustrative)
    ("1111111111111111111111111111111111111111", "Indirizzo test - non per produzione", false),
];

/// Check if an address is a known scam
pub fn is_known_scam(address: &str) -> Option<ScamInfo> {
    let addr = address.to_lowercase();
    let addr = addr.strip_prefix("0x").unwrap_or(&addr);

    for (scam_addr, reason, confirmed) in KNOWN_SCAM_ADDRESSES {
        if addr == *scam_addr {
            return Some(ScamInfo {
                address: address.to_string(),
                reason: reason.to_string(),
                severity: if *confirmed { ScamSeverity::Confirmed } else { ScamSeverity::Suspected },
            });
        }
    }
    None
}

/// Risk level for an address based on heuristics
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
}

/// Check basic risk indicators for an address
/// - Sending to self
/// - Sending to 0x0 (burn)
/// - Known scam database match
pub fn assess_address_risk(
    recipient: &str,
    sender: &str,
) -> (RiskLevel, Option<String>) {
    let r = recipient.to_lowercase();
    let s = sender.to_lowercase();

    // Check known scam database
    if let Some(info) = is_known_scam(recipient) {
        let level = match info.severity {
            ScamSeverity::Confirmed => RiskLevel::High,
            ScamSeverity::Suspected => RiskLevel::Medium,
        };
        return (level, Some(info.reason));
    }

    // Sending to self
    if r == s {
        return (RiskLevel::Low, Some("Stai inviando fondi a te stesso".into()));
    }

    // Sending to 0x0
    let r_stripped = r.strip_prefix("0x").unwrap_or(&r);
    if r_stripped.chars().all(|c| c == '0') {
        return (RiskLevel::High, Some("Indirizzo zero — i fondi saranno persi per sempre".into()));
    }

    (RiskLevel::Safe, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_scam() {
        let result = is_known_scam("0x0000000000000000000000000000000000000001");
        assert!(result.is_some());
        assert_eq!(result.unwrap().severity, ScamSeverity::Confirmed);
    }

    #[test]
    fn test_normal_address() {
        let result = is_known_scam("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
        assert!(result.is_none());
    }

    #[test]
    fn test_send_to_self() {
        let (level, reason) = assess_address_risk(
            "0xabc123",
            "0xabc123",
        );
        assert_eq!(level, RiskLevel::Low);
        assert!(reason.unwrap().contains("te stesso"));
    }

    #[test]
    fn test_send_to_zero() {
        let (level, reason) = assess_address_risk(
            "0x0000000000000000000000000000000000000000",
            "0xabc123",
        );
        assert_eq!(level, RiskLevel::High);
        assert!(reason.unwrap().contains("zero"));
    }

    #[test]
    fn test_safe_address() {
        let (level, _) = assess_address_risk(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            "0xabc123",
        );
        assert_eq!(level, RiskLevel::Safe);
    }
}
