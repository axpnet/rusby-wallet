// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// qr: QR code generation as inline SVG (pure Rust, zero JS dependencies)

use qrcode::QrCode;
use qrcode::render::svg;

/// Generate an SVG string containing a QR code for the given data
pub fn generate_qr_svg(data: &str, size: u32) -> Result<String, String> {
    let code = QrCode::new(data.as_bytes()).map_err(|e| format!("QR error: {}", e))?;
    let svg_string = code
        .render::<svg::Color>()
        .min_dimensions(size, size)
        .max_dimensions(size, size)
        .dark_color(svg::Color("#ffffff"))
        .light_color(svg::Color("#1a1a2e"))
        .quiet_zone(true)
        .build();
    Ok(svg_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_qr_svg() {
        let svg = generate_qr_svg("0x1234567890abcdef", 200).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(!svg.is_empty());
    }

    #[test]
    fn test_generate_qr_svg_solana_address() {
        let svg = generate_qr_svg("7EcDhSYGxXyscszYEp35KHN8vvw3svAuLKTzXwCFLtV", 200).unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn test_generate_qr_svg_empty_fails() {
        // Empty string should still work for QR
        let result = generate_qr_svg("", 200);
        assert!(result.is_ok());
    }
}
