//! The short-lived `team_login` cookie: it ties a login callback to the login that started it.
//!
//! It holds `state`, `nonce` and the PKCE verifier, signed with HMAC-SHA256 using `SESSION_SECRET`.
//! It is valid for 10 minutes (decision 0007).

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, TimeDelta, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

pub const NAME: &str = "team_login";
pub const MAX_AGE: TimeDelta = TimeDelta::minutes(10);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginAttempt {
    pub state: String,
    pub nonce: String,
    pub pkce_verifier: String,
    /// Unix seconds.
    pub created_at: i64,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LoginCookieError {
    #[error("login cookie is malformed")]
    Malformed,
    #[error("login cookie signature is wrong")]
    BadSignature,
    #[error("login cookie is too old")]
    Expired,
    #[error("cannot sign the login cookie: {0}")]
    Signing(String),
}

fn mac(secret: &str) -> Result<Hmac<Sha256>, LoginCookieError> {
    Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|error| LoginCookieError::Signing(error.to_string()))
}

pub fn encode(attempt: &LoginAttempt, secret: &str) -> Result<String, LoginCookieError> {
    let payload = serde_json::to_vec(attempt)
        .map_err(|error| LoginCookieError::Signing(error.to_string()))?;
    let mut signer = mac(secret)?;
    signer.update(&payload);
    let signature = signer.finalize().into_bytes();
    Ok(format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(payload),
        URL_SAFE_NO_PAD.encode(signature)
    ))
}

pub fn decode(
    value: &str,
    secret: &str,
    now: DateTime<Utc>,
) -> Result<LoginAttempt, LoginCookieError> {
    let (payload, signature) = value.split_once('.').ok_or(LoginCookieError::Malformed)?;
    let payload = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| LoginCookieError::Malformed)?;
    let signature = URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|_| LoginCookieError::Malformed)?;

    let mut verifier = mac(secret)?;
    verifier.update(&payload);
    verifier
        .verify_slice(&signature)
        .map_err(|_| LoginCookieError::BadSignature)?;

    let attempt: LoginAttempt =
        serde_json::from_slice(&payload).map_err(|_| LoginCookieError::Malformed)?;
    let created_at =
        DateTime::from_timestamp(attempt.created_at, 0).ok_or(LoginCookieError::Malformed)?;
    if now - created_at > MAX_AGE || created_at > now + TimeDelta::minutes(1) {
        return Err(LoginCookieError::Expired);
    }
    Ok(attempt)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    fn attempt(created_at: DateTime<Utc>) -> LoginAttempt {
        LoginAttempt {
            state: "state".to_owned(),
            nonce: "nonce".to_owned(),
            pkce_verifier: "verifier".to_owned(),
            created_at: created_at.timestamp(),
        }
    }

    #[test]
    fn round_trips() {
        let now = Utc::now();
        let value = encode(&attempt(now), SECRET).unwrap();
        assert_eq!(decode(&value, SECRET, now), Ok(attempt(now)));
    }

    #[test]
    fn refuses_a_changed_payload() {
        let now = Utc::now();
        let value = encode(&attempt(now), SECRET).unwrap();
        let (_, signature) = value.split_once('.').unwrap();
        let mut other = attempt(now);
        other.state = "forged".to_owned();
        let forged_payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&other).unwrap());
        let forged = format!("{forged_payload}.{signature}");
        assert_eq!(
            decode(&forged, SECRET, now),
            Err(LoginCookieError::BadSignature)
        );
    }

    #[test]
    fn refuses_another_secret() {
        let now = Utc::now();
        let value = encode(&attempt(now), SECRET).unwrap();
        assert_eq!(
            decode(&value, "another-secret-another-secret-xx", now),
            Err(LoginCookieError::BadSignature)
        );
    }

    #[test]
    fn refuses_an_attempt_older_than_ten_minutes() {
        // The cookie keeps whole seconds.
        let start = DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap();
        let value = encode(&attempt(start), SECRET).unwrap();
        assert!(decode(&value, SECRET, start + TimeDelta::minutes(10)).is_ok());
        assert_eq!(
            decode(
                &value,
                SECRET,
                start + TimeDelta::minutes(10) + TimeDelta::seconds(1)
            ),
            Err(LoginCookieError::Expired)
        );
    }

    #[test]
    fn refuses_garbage() {
        assert_eq!(
            decode("not-a-cookie", SECRET, Utc::now()),
            Err(LoginCookieError::Malformed)
        );
    }
}
