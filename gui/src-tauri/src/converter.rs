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
}
