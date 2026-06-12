//! Byte-exact port of crdb_to_zoho.py conversion logic.

pub fn parse_date(raw: &str) -> Result<String, String> {
    let token = raw
        .split_whitespace()
        .next()
        .ok_or_else(|| format!("empty date: {raw:?}"))?;
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("unexpected date format: {token:?}"));
    }
    Ok(format!("{}-{:0>2}-{:0>2}", parts[2], parts[1], parts[0]))
}

pub fn parse_amount(raw: &str) -> Result<f64, String> {
    raw.trim()
        .replace(',', "")
        .parse::<f64>()
        .map_err(|e| format!("invalid amount {raw:?}: {e}"))
}

pub fn clean_reference(raw: &str) -> String {
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.chars().take(99).collect()
}

/// Format a float to match Python's `str(float)` — shortest round-trip
/// representation with at least one decimal digit (e.g. `977000.0`, not
/// `977000`). Rust's `{:?}` produces this; do NOT change to `{}` (Display),
/// which would drop the trailing `.0` and break the byte-exact CSV contract.
/// Python switches to scientific notation at extreme magnitudes (~1e15+)
/// where Rust does not match exactly; bank amounts never reach that range,
/// and the byte-exact reference test is the arbiter.
pub fn format_amount(v: f64) -> String {
    format!("{v:?}")
}

pub fn csv_field(field: &str) -> String {
    if field.contains(';') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_date_standard() {
        assert_eq!(parse_date(" 02.01.2026 14:33:12").unwrap(), "2026-01-02");
    }

    #[test]
    fn parse_date_pads_single_digits() {
        assert_eq!(parse_date("3.7.2026 00:00:00").unwrap(), "2026-07-03");
    }

    #[test]
    fn parse_date_rejects_garbage() {
        assert!(parse_date("not a date").is_err());
        assert!(parse_date("   ").is_err());
    }

    #[test]
    fn parse_amount_with_thousands_separator() {
        assert_eq!(parse_amount(" 977,000.00").unwrap(), 977000.0);
    }

    #[test]
    fn parse_amount_zero_and_plain() {
        assert_eq!(parse_amount(" 0.00").unwrap(), 0.0);
        assert_eq!(parse_amount("304.92").unwrap(), 304.92);
    }

    #[test]
    fn parse_amount_rejects_garbage() {
        assert!(parse_amount("abc").is_err());
    }

    #[test]
    fn clean_reference_collapses_whitespace() {
        assert_eq!(
            clean_reference("  E-COM   Purchase\t VISA \n POS  "),
            "E-COM Purchase VISA POS"
        );
    }

    #[test]
    fn clean_reference_truncates_to_99_chars() {
        let long = "x".repeat(150);
        assert_eq!(clean_reference(&long).chars().count(), 99);
    }

    #[test]
    fn clean_reference_truncates_by_chars_not_bytes() {
        let umlauts = "ä".repeat(150); // 2 bytes per char in UTF-8
        let result = clean_reference(&umlauts);
        assert_eq!(result.chars().count(), 99);
        assert_eq!(result, "ä".repeat(99));
    }

    #[test]
    fn format_amount_matches_python_str() {
        assert_eq!(format_amount(977000.0), "977000.0");
        assert_eq!(format_amount(0.0), "0.0");
        assert_eq!(format_amount(304.92), "304.92");
        assert_eq!(format_amount(1694.0), "1694.0");
    }

    #[test]
    fn csv_field_plain_passthrough() {
        assert_eq!(csv_field("Transfer"), "Transfer");
        assert_eq!(csv_field(""), "");
    }

    #[test]
    fn csv_field_quotes_when_needed() {
        assert_eq!(csv_field("a;b"), "\"a;b\"");
        assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn clean_reference_empty_and_whitespace_only() {
        assert_eq!(clean_reference(""), "");
        assert_eq!(clean_reference("   "), "");
    }

    #[test]
    fn csv_field_quotes_newline_and_carriage_return() {
        assert_eq!(csv_field("a\nb"), "\"a\nb\"");
        assert_eq!(csv_field("a\rb"), "\"a\rb\"");
    }

    #[test]
    fn format_amount_large_realistic_values() {
        // TZS amounts reach billions; Python str() and Rust {:?} agree on
        // plain decimal notation in this range (divergence starts ~1e15)
        assert_eq!(format_amount(1000000000.0), "1000000000.0");
        assert_eq!(format_amount(999999999999.99), "999999999999.99");
    }
}
