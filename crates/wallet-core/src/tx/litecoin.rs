// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tx/litecoin: P2WPKH transaction construction and signing for Litecoin
//
// Litecoin uses the same BIP-143 SegWit format as Bitcoin.
// This module re-exports Bitcoin TX types and adds Litecoin-specific helpers.

use super::SignedTransaction;
use crate::chains::ChainId;

// Re-export Bitcoin TX types (identical format for Litecoin)
pub use super::bitcoin::{BitcoinTransaction, Utxo, TxOutput, p2wpkh_script};

/// Parse LTC amount string to litoshi (1 LTC = 10^8 litoshi)
pub fn parse_ltc_to_litoshi(amount: &str) -> Result<u64, String> {
    super::bitcoin::parse_btc_to_satoshi(amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ltc_to_litoshi() {
        assert_eq!(parse_ltc_to_litoshi("1").unwrap(), 100_000_000);
        assert_eq!(parse_ltc_to_litoshi("0.001").unwrap(), 100_000);
        assert_eq!(parse_ltc_to_litoshi("0.00000001").unwrap(), 1);
    }

    #[test]
    fn test_litecoin_sign() {
        use crate::bip39_utils;
        use crate::chains::litecoin;

        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let privkey = litecoin::get_private_key(&seed).unwrap();
        let pubkey = litecoin::get_public_key(&seed).unwrap();
        let pubkey_hash = litecoin::hash160_pubkey(&pubkey);

        let utxo = Utxo {
            txid: [0xcc; 32],
            vout: 0,
            value: 100_000,
            script_pubkey: p2wpkh_script(&pubkey_hash),
        };

        let tx = BitcoinTransaction::build_p2wpkh(
            vec![utxo],
            &[0xdd; 20],
            50_000,
            &pubkey_hash,
            1_000,
        ).unwrap();

        let signed = tx.sign_for_chain(&privkey, ChainId::Litecoin).unwrap();
        assert!(!signed.raw_bytes.is_empty());
        assert_eq!(signed.chain_id, ChainId::Litecoin);
    }
}
