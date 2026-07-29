//! Shared session and cookie primitives for the HTTP layer.
//!
//! Customer and admin sessions have separate cookie names and server-side
//! stores, but they share the same security requirements. Keeping token
//! generation and cookie parsing here prevents either authentication flow from
//! drifting to weaker settings.

use axum::http::HeaderMap;
use axum::http::header::COOKIE;
use std::env;

const TOKEN_BYTES: usize = 24;
const HEX: &[u8; 16] = b"0123456789abcdef";

/// Generates a cryptographically random, cookie-safe session identifier.
///
/// The prefix is only a non-sensitive diagnostic label. The 192 random bits
/// come from the operating system and make customer/admin session identifiers
/// impractical to predict.
pub(crate) fn generate_session_id(prefix: &str) -> Result<String, String> {
    let mut random = [0_u8; TOKEN_BYTES];
    getrandom::fill(&mut random).map_err(|_| {
        "The server could not create a secure session. Please try again.".to_string()
    })?;

    let mut token = String::with_capacity(prefix.len() + 1 + TOKEN_BYTES * 2);
    token.push_str(prefix);
    token.push('-');
    for byte in random {
        token.push(char::from(HEX[usize::from(byte >> 4)]));
        token.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Ok(token)
}

/// Reads one named cookie without exposing unrelated cookie values.
pub(crate) fn cookie_value(headers: &HeaderMap, expected_name: &str) -> Option<String> {
    headers
        .get(COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let (name, value) = cookie.trim().split_once('=')?;
                (name == expected_name).then(|| value.to_string())
            })
        })
}

/// Builds an HTTP-only, same-site session cookie.
pub(crate) fn session_cookie(name: &str, value: &str, max_age_seconds: u32) -> String {
    format!(
        "{name}={value}; HttpOnly; SameSite=Lax; Path=/; Max-Age={max_age_seconds}{}",
        secure_cookie_attribute()
    )
}

/// Builds a partitioned cookie for embedded mobile-preview webviews.
///
/// Real phones and normal browser tabs use the first-party `SameSite=Lax`
/// cookie above. Some IDE preview extensions render localhost inside a
/// cross-site frame, where that cookie may not be replayed. CHIPS-capable
/// browsers can use this separate `Secure; SameSite=None; Partitioned` cookie
/// without changing the normal local-network cookie policy.
pub(crate) fn partitioned_session_cookie(name: &str, value: &str, max_age_seconds: u32) -> String {
    format!(
        "{name}={value}; HttpOnly; Secure; SameSite=None; Partitioned; Path=/; Max-Age={max_age_seconds}"
    )
}

/// Expires one named session cookie without affecting the other user role.
pub(crate) fn expired_session_cookie(name: &str) -> String {
    format!(
        "{name}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0{}",
        secure_cookie_attribute()
    )
}

/// Expires the optional partitioned preview cookie.
pub(crate) fn expired_partitioned_session_cookie(name: &str) -> String {
    format!("{name}=; HttpOnly; Secure; SameSite=None; Partitioned; Path=/; Max-Age=0")
}

/// Local HTTP remains usable by default; HTTPS deployments opt into `Secure`.
fn secure_cookie_attribute() -> &'static str {
    match env::var("APP_COOKIE_SECURE") {
        Ok(value)
            if matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            ) =>
        {
            "; Secure"
        }
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_session_ids_are_distinct_and_cookie_safe() {
        let first = generate_session_id("customer").expect("OS randomness should be available");
        let second = generate_session_id("customer").expect("OS randomness should be available");

        assert_ne!(first, second);
        assert!(first.starts_with("customer-"));
        assert_eq!(first.len(), "customer-".len() + TOKEN_BYTES * 2);
        assert!(
            first
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
        );
    }

    #[test]
    fn partitioned_cookie_supports_embedded_preview_without_weakening_http_only() {
        let cookie = partitioned_session_cookie("customer_preview_session", "opaque", 60);

        assert!(cookie.starts_with("customer_preview_session=opaque"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("SameSite=None"));
        assert!(cookie.contains("Partitioned"));
        assert!(cookie.contains("Path=/"));
    }
}
