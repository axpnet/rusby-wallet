// Rusby Wallet — Phishing domain detection
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

/// Known phishing domains (lowercase)
const PHISHING_DOMAINS: &[&str] = &[
    "uniswap-airdrop.com",
    "uniswap-claim.com",
    "uniswap-app.org",
    "uniswaps.org",
    "pancakeswap-finance.com",
    "pancakeswaps.finance",
    "opensea-nft.org",
    "opensea-claim.com",
    "metamask-io.com",
    "metamask-wallet.org",
    "metamask-support.com",
    "phantom-wallet.org",
    "phantom-airdrop.com",
    "aave-finance.com",
    "aave-app.org",
    "compound-finance.org",
    "sushiswap-app.com",
    "curve-fi.org",
    "lido-finance.org",
    "convex-finance.org",
    "1inch-exchange.org",
    "dydx-exchange.org",
    "arbitrum-airdrop.com",
    "optimism-claim.com",
    "zksync-airdrop.com",
    "layerzero-claim.com",
    "starknet-airdrop.com",
    "blur-nft.org",
    "raydium-exchange.com",
    "jupiter-airdrop.com",
    "etherscan-verify.com",
    "bscscan-verify.com",
    "polygonscan-verify.com",
    "solscan-verify.com",
    "connect-wallet.org",
    "walletconnect-bridge.org",
    "dapp-connect.org",
    "defi-wallet-connect.com",
    "free-nft-mint.com",
    "free-token-airdrop.com",
    "claim-rewards-defi.com",
    "revoke-approval.com",
];

/// Known legitimate domains for typosquatting comparison
const LEGITIMATE_DOMAINS: &[&str] = &[
    "uniswap.org",
    "app.uniswap.org",
    "pancakeswap.finance",
    "opensea.io",
    "metamask.io",
    "phantom.app",
    "aave.com",
    "compound.finance",
    "sushi.com",
    "curve.fi",
    "lido.fi",
    "1inch.io",
    "dydx.exchange",
    "arbitrum.io",
    "optimism.io",
    "etherscan.io",
    "bscscan.com",
    "polygonscan.com",
    "solscan.io",
    "walletconnect.com",
    "raydium.io",
    "jup.ag",
];

/// Suspicious TLDs commonly used in phishing
const SUSPICIOUS_TLDS: &[&str] = &[".xyz", ".tk", ".ml", ".ga", ".cf", ".gq", ".top", ".buzz", ".icu"];

/// Crypto keywords that indicate potential phishing when in suspicious domains
const CRYPTO_KEYWORDS: &[&str] = &[
    "wallet", "airdrop", "claim", "mint", "swap", "defi", "nft",
    "connect", "bridge", "stake", "reward", "token", "approve", "revoke",
];

/// Check if a domain is in the known phishing blocklist
pub fn is_phishing_domain(domain: &str) -> bool {
    let d = domain.to_lowercase();
    let d = d.strip_prefix("https://").unwrap_or(&d);
    let d = d.strip_prefix("http://").unwrap_or(d);
    let d = d.split('/').next().unwrap_or(d);
    PHISHING_DOMAINS.contains(&d)
}

/// Check if a domain is suspicious (heuristic analysis)
/// Returns Some(reason) if suspicious, None if appears safe
pub fn check_suspicious_domain(domain: &str) -> Option<String> {
    let d = domain.to_lowercase();
    let d = d.strip_prefix("https://").unwrap_or(&d);
    let d = d.strip_prefix("http://").unwrap_or(d);
    let d = d.split('/').next().unwrap_or(d);

    // 1. Exact match on blocklist
    if PHISHING_DOMAINS.contains(&d) {
        return Some("Dominio presente nella blocklist di phishing".into());
    }

    // 2. Typosquatting check (Levenshtein distance)
    for legit in LEGITIMATE_DOMAINS {
        let dist = levenshtein(d, legit);
        if dist > 0 && dist <= 2 {
            return Some(format!("Dominio simile a {} (possibile typosquatting)", legit));
        }
    }

    // 3. Suspicious TLD + crypto keyword
    let has_suspicious_tld = SUSPICIOUS_TLDS.iter().any(|tld| d.ends_with(tld));
    if has_suspicious_tld {
        let has_keyword = CRYPTO_KEYWORDS.iter().any(|kw| d.contains(kw));
        if has_keyword {
            return Some("Dominio con TLD sospetto e keyword crypto".into());
        }
    }

    // 4. Digit substitution (e.g. un1swap, meta4ask)
    let digits_in_word = d.split('.').next().unwrap_or(d);
    let has_mixed = digits_in_word.chars().any(|c| c.is_ascii_digit())
        && digits_in_word.chars().any(|c| c.is_ascii_alphabetic())
        && CRYPTO_KEYWORDS.iter().any(|kw| {
            let stripped: String = digits_in_word.chars().filter(|c| c.is_ascii_alphabetic()).collect();
            stripped.contains(kw) || kw.contains(&stripped)
        });
    if has_mixed {
        return Some("Dominio con sostituzione cifre/lettere sospetta".into());
    }

    None
}

/// Simple Levenshtein distance
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let la = a.len();
    let lb = b.len();

    let mut matrix = vec![vec![0usize; lb + 1]; la + 1];
    for i in 0..=la { matrix[i][0] = i; }
    for j in 0..=lb { matrix[0][j] = j; }

    for i in 1..=la {
        for j in 1..=lb {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }
    matrix[la][lb]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_phishing() {
        assert!(is_phishing_domain("uniswap-airdrop.com"));
        assert!(is_phishing_domain("https://metamask-io.com"));
        assert!(is_phishing_domain("metamask-support.com/verify"));
    }

    #[test]
    fn test_legitimate_not_flagged() {
        assert!(!is_phishing_domain("uniswap.org"));
        assert!(!is_phishing_domain("metamask.io"));
        assert!(!is_phishing_domain("app.uniswap.org"));
    }

    #[test]
    fn test_typosquatting_detection() {
        // "uniswap.rog" is 1 edit from "uniswap.org"
        let result = check_suspicious_domain("uniswap.rog");
        assert!(result.is_some());
        assert!(result.unwrap().contains("typosquatting"));
    }

    #[test]
    fn test_suspicious_tld_crypto() {
        let result = check_suspicious_domain("defi-wallet.xyz");
        assert!(result.is_some());
        assert!(result.unwrap().contains("TLD sospetto"));
    }

    #[test]
    fn test_safe_domain() {
        assert!(check_suspicious_domain("google.com").is_none());
        assert!(check_suspicious_domain("github.com").is_none());
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("abc", "abd"), 1);
        assert_eq!(levenshtein("abc", "abcd"), 1);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn test_blocklist_exact_match_via_suspicious() {
        let result = check_suspicious_domain("free-nft-mint.com");
        assert!(result.is_some());
        assert!(result.unwrap().contains("blocklist"));
    }
}
