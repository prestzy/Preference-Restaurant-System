//! Validation and normalization for customer/order web input.
//!
//! These rules belong at the web/domain boundary rather than in templates or
//! recommendation scoring. Returning field-specific messages lets handlers
//! preserve form values and explain recoverable input errors.

pub(crate) fn customer_name(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.len() < 2 {
        return Err("Customer name is required.".to_string());
    }
    if value.len() > 60 {
        return Err("Customer name is too long.".to_string());
    }
    Ok(value.to_string())
}

/// Normalizes common Malaysian local/international phone input to `60...`.
pub(crate) fn customer_phone(value: &str) -> Result<String, String> {
    let raw = value.trim().replace([' ', '-'], "");
    let digits = if let Some(rest) = raw.strip_prefix("+60") {
        format!("60{rest}")
    } else if let Some(rest) = raw.strip_prefix('0') {
        format!("60{rest}")
    } else {
        raw
    };

    if digits.len() < 10 || digits.len() > 13 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err("Enter a valid Malaysian phone number.".to_string());
    }
    Ok(digits)
}

pub(crate) fn table_number(value: &str) -> Result<String, String> {
    let value = value.trim().to_uppercase();
    if value.is_empty() {
        return Err("Table number is required for this dine-in prototype.".to_string());
    }
    if value.len() > 16 {
        return Err("Table number is too long.".to_string());
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err("Table number can use letters, numbers, dash, or underscore only.".to_string());
    }
    Ok(value)
}

pub(crate) fn optional_short_text(
    value: &Option<String>,
    max_len: usize,
    label: &str,
) -> Result<Option<String>, String> {
    let Some(value) = value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    if value.len() > max_len {
        return Err(format!("{label} is too long."));
    }
    Ok(Some(value.to_string()))
}

/// Returns only the final digits needed for a non-sensitive staff/session label.
pub(crate) fn masked_phone_suffix(phone: &str) -> String {
    phone
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn customer_fields_are_normalized_and_bounded() {
        assert_eq!(customer_name("  Aina  ").unwrap(), "Aina");
        assert_eq!(customer_phone("012-345 6789").unwrap(), "60123456789");
        assert_eq!(table_number(" t-05 ").unwrap(), "T-05");
        assert!(customer_name("A").is_err());
        assert!(customer_phone("not-a-phone").is_err());
        assert!(table_number("table / 1").is_err());
    }

    #[test]
    fn optional_text_preserves_empty_as_none_and_enforces_limit() {
        assert_eq!(
            optional_short_text(&Some("  ".to_string()), 10, "Note").unwrap(),
            None
        );
        assert_eq!(
            optional_short_text(&Some(" less spicy ".to_string()), 20, "Note").unwrap(),
            Some("less spicy".to_string())
        );
        assert!(optional_short_text(&Some("too long".to_string()), 3, "Note").is_err());
    }
}
