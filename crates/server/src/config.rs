//! Settings, read once at start. The names match the cluster contract (see `README.md`).

use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

#[cfg(debug_assertions)]
use chrono::{DateTime, Utc};
use url::Url;

use crate::cipher::NameKey;

const MIN_SESSION_SECRET_LEN: usize = 32;
const DEFAULT_GAME_LIST_URL: &str = "https://db.basketball.nl";
/// A new key cannot read a database made with another key, so this speaks of a new database only.
const PLAYER_NAME_KEY_HINT: &str =
    "for a new database, make a key with `head -c 32 /dev/urandom | base64`";

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("setting {0} is required")]
    Missing(&'static str),
    #[error("setting PLAYER_NAME_KEY is required; {}", PLAYER_NAME_KEY_HINT)]
    MissingPlayerNameKey,
    #[error("setting {name} is not valid: {reason}")]
    Invalid { name: &'static str, reason: String },
}

/// Settings for `team serve`.
#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub oidc_issuer_url: Url,
    pub oidc_client_id: String,
    pub oidc_client_secret: String,
    pub session_secret: String,
    /// The key of the encrypted player names.
    pub player_name_key: NameKey,
    pub origin: Url,
    pub session_ttl: Duration,
    pub access_role: String,
    pub host: IpAddr,
    pub port: u16,
    pub web_dir: PathBuf,
    /// The root of the game list source's site.
    pub game_list_url: Url,
    /// Exists only in debug builds, so a release build cannot contain the bypass (decision 0007).
    #[cfg(debug_assertions)]
    pub dev_auth_bypass: bool,
    /// The moment the clock starts at, for tests that must not depend on the real date. Exists
    /// only in debug builds, as the login bypass.
    #[cfg(debug_assertions)]
    pub dev_now: Option<DateTime<Utc>>,
}

// Secrets stay out of debug output and logs.
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("oidc_issuer_url", &self.oidc_issuer_url.as_str())
            .field("oidc_client_id", &self.oidc_client_id)
            .field("origin", &self.origin.as_str())
            .field("session_ttl", &self.session_ttl)
            .field("access_role", &self.access_role)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("web_dir", &self.web_dir)
            .field("game_list_url", &self.game_list_url.as_str())
            .finish_non_exhaustive()
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Reads the settings through `lookup`, so tests do not touch the process environment.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let required = |name: &'static str| {
            lookup(name)
                .filter(|value| !value.trim().is_empty())
                .ok_or(ConfigError::Missing(name))
        };
        let optional = |name: &'static str| lookup(name).filter(|value| !value.trim().is_empty());

        let session_secret = required("SESSION_SECRET")?;
        if session_secret.chars().count() < MIN_SESSION_SECRET_LEN {
            return Err(ConfigError::Invalid {
                name: "SESSION_SECRET",
                reason: format!("must be at least {MIN_SESSION_SECRET_LEN} characters"),
            });
        }

        let player_name_key = player_name_key(lookup("PLAYER_NAME_KEY"))?;

        let session_ttl_seconds = match optional("SESSION_TTL_SECONDS") {
            None => 28_800,
            Some(value) => parse_positive("SESSION_TTL_SECONDS", &value)?,
        };

        let host = match optional("HOST") {
            None => IpAddr::from([0, 0, 0, 0]),
            Some(value) => value.parse().map_err(|_| ConfigError::Invalid {
                name: "HOST",
                reason: "must be an IP address".to_owned(),
            })?,
        };

        let port = match optional("PORT") {
            None => 3000,
            Some(value) => value.parse().map_err(|_| ConfigError::Invalid {
                name: "PORT",
                reason: "must be a port number".to_owned(),
            })?,
        };

        let dev_auth_bypass_setting =
            optional("DEV_AUTH_BYPASS").is_some_and(|value| value == "true");
        #[cfg(not(debug_assertions))]
        if dev_auth_bypass_setting {
            tracing::warn!(
                "DEV_AUTH_BYPASS is set but ignored: release builds always require login"
            );
        }

        let dev_now_setting = optional("DEV_NOW");
        #[cfg(not(debug_assertions))]
        if dev_now_setting.is_some() {
            tracing::warn!("DEV_NOW is set but ignored: release builds use the real clock");
        }
        #[cfg(debug_assertions)]
        let dev_now = dev_now_setting
            .map(|value| parse_time("DEV_NOW", &value))
            .transpose()?;

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            oidc_issuer_url: parse_url("OIDC_ISSUER_URL", &required("OIDC_ISSUER_URL")?)?,
            oidc_client_id: required("OIDC_CLIENT_ID")?,
            oidc_client_secret: required("OIDC_CLIENT_SECRET")?,
            session_secret,
            player_name_key,
            origin: parse_url("ORIGIN", &required("ORIGIN")?)?,
            session_ttl: Duration::from_secs(session_ttl_seconds),
            access_role: optional("TEAM_ACCESS_ROLE")
                .unwrap_or_else(|| "team_coordinator".to_owned()),
            host,
            port,
            web_dir: optional("WEB_DIR").map_or_else(|| PathBuf::from("/app/web"), PathBuf::from),
            game_list_url: parse_url(
                "GAME_LIST_URL",
                &optional("GAME_LIST_URL").unwrap_or_else(|| DEFAULT_GAME_LIST_URL.to_owned()),
            )?,
            #[cfg(debug_assertions)]
            dev_auth_bypass: dev_auth_bypass_setting,
            #[cfg(debug_assertions)]
            dev_now,
        })
    }
}

/// Reads `PLAYER_NAME_KEY`; the error never contains the value.
pub fn player_name_key(value: Option<String>) -> Result<NameKey, ConfigError> {
    let value = value
        .filter(|value| !value.trim().is_empty())
        .ok_or(ConfigError::MissingPlayerNameKey)?;
    NameKey::from_base64(&value).map_err(|error| ConfigError::Invalid {
        name: "PLAYER_NAME_KEY",
        reason: format!("{error}; {PLAYER_NAME_KEY_HINT}"),
    })
}

fn parse_url(name: &'static str, value: &str) -> Result<Url, ConfigError> {
    Url::parse(value).map_err(|error| ConfigError::Invalid {
        name,
        reason: error.to_string(),
    })
}

#[cfg(debug_assertions)]
fn parse_time(name: &'static str, value: &str) -> Result<DateTime<Utc>, ConfigError> {
    DateTime::parse_from_rfc3339(value)
        .map(|time| time.with_timezone(&Utc))
        .map_err(|_| ConfigError::Invalid {
            name,
            reason: "must be a time such as 2026-10-12T10:00:00Z".to_owned(),
        })
}

fn parse_positive(name: &'static str, value: &str) -> Result<u64, ConfigError> {
    match value.parse::<u64>() {
        Ok(number) if number > 0 => Ok(number),
        _ => Err(ConfigError::Invalid {
            name,
            reason: "must be a positive whole number".to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// 32 bytes of 7 in base64.
    const PLAYER_NAME_KEY: &str = "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=";

    fn settings(overrides: &[(&str, Option<&str>)]) -> Result<Config, ConfigError> {
        let mut values: HashMap<String, String> = [
            (
                "DATABASE_URL",
                "postgresql://team:team@localhost:5432/team_v2",
            ),
            ("OIDC_ISSUER_URL", "https://id.example.test"),
            ("OIDC_CLIENT_ID", "team-coordinator"),
            ("OIDC_CLIENT_SECRET", "client-secret"),
            ("SESSION_SECRET", "0123456789abcdef0123456789abcdef"),
            ("PLAYER_NAME_KEY", PLAYER_NAME_KEY),
            ("ORIGIN", "https://team.example.test"),
        ]
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .collect();
        for (name, value) in overrides {
            match value {
                Some(value) => values.insert((*name).to_owned(), (*value).to_owned()),
                None => values.remove(*name),
            };
        }
        Config::from_lookup(|name| values.get(name).cloned())
    }

    #[test]
    fn applies_defaults() -> Result<(), ConfigError> {
        let config = settings(&[])?;
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, IpAddr::from([0, 0, 0, 0]));
        assert_eq!(config.session_ttl, Duration::from_secs(28_800));
        assert_eq!(config.access_role, "team_coordinator");
        assert_eq!(config.web_dir, PathBuf::from("/app/web"));
        assert_eq!(config.game_list_url.as_str(), "https://db.basketball.nl/");
        #[cfg(debug_assertions)]
        assert!(!config.dev_auth_bypass);
        #[cfg(debug_assertions)]
        assert_eq!(config.dev_now, None);
        Ok(())
    }

    #[cfg(debug_assertions)]
    #[test]
    fn reads_the_start_of_the_development_clock() -> Result<(), ConfigError> {
        use chrono::TimeZone;

        let config = settings(&[("DEV_NOW", Some("2026-10-12T10:00:00Z"))])?;
        let expected = Utc.with_ymd_and_hms(2026, 10, 12, 10, 0, 0).single();
        assert_eq!(config.dev_now, expected);
        Ok(())
    }

    #[cfg(debug_assertions)]
    #[test]
    fn names_a_development_clock_that_is_not_a_time() {
        let error = settings(&[("DEV_NOW", Some("12 October 2026"))]).err();
        assert!(matches!(
            error,
            Some(ConfigError::Invalid {
                name: "DEV_NOW",
                ..
            })
        ));
    }

    #[test]
    fn names_a_missing_database_url() {
        let error = settings(&[("DATABASE_URL", None)]).err();
        assert_eq!(error, Some(ConfigError::Missing("DATABASE_URL")));
        assert!(error.is_some_and(|error| error.to_string().contains("DATABASE_URL")));
    }

    #[test]
    fn refuses_a_short_session_secret() {
        let error = settings(&[("SESSION_SECRET", Some("0123456789abcdef0123456789abcde"))]).err();
        assert!(matches!(
            error,
            Some(ConfigError::Invalid {
                name: "SESSION_SECRET",
                ..
            })
        ));
    }

    #[test]
    fn refuses_a_session_ttl_that_is_not_a_number() {
        let error = settings(&[("SESSION_TTL_SECONDS", Some("abc"))]).err();
        assert!(matches!(
            error,
            Some(ConfigError::Invalid {
                name: "SESSION_TTL_SECONDS",
                ..
            })
        ));
    }

    #[test]
    fn refuses_a_game_list_url_that_is_not_a_url() {
        let error = settings(&[("GAME_LIST_URL", Some("db.basketball.nl"))]).err();
        assert!(matches!(
            error,
            Some(ConfigError::Invalid {
                name: "GAME_LIST_URL",
                ..
            })
        ));
    }

    #[test]
    fn debug_output_hides_secrets() -> Result<(), ConfigError> {
        let output = format!("{:?}", settings(&[])?);
        assert!(!output.contains("client-secret"));
        assert!(!output.contains("0123456789abcdef"));
        assert!(!output.contains("postgresql://"));
        assert!(!output.contains(PLAYER_NAME_KEY));
        assert!(!output.contains("BwcH"));
        Ok(())
    }

    #[test]
    fn names_a_missing_player_name_key() {
        for value in [None, Some("  ")] {
            let error = settings(&[("PLAYER_NAME_KEY", value)]).err();
            assert_eq!(error, Some(ConfigError::MissingPlayerNameKey));
            assert!(error.is_some_and(|error| {
                error
                    .to_string()
                    .starts_with("setting PLAYER_NAME_KEY is required")
                    && error.to_string().contains(PLAYER_NAME_KEY_HINT)
            }));
        }
    }

    #[test]
    fn refuses_a_player_name_key_that_is_not_32_bytes_of_base64() {
        // 31 bytes, then text that is not base64.
        for value in [
            "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBw==",
            "secret-not-base64!",
        ] {
            let error = settings(&[("PLAYER_NAME_KEY", Some(value))]).err();
            assert!(matches!(
                &error,
                Some(ConfigError::Invalid {
                    name: "PLAYER_NAME_KEY",
                    ..
                })
            ));
            assert!(error.is_some_and(|error| {
                !error.to_string().contains(value)
                    && error.to_string().contains(PLAYER_NAME_KEY_HINT)
            }));
        }
    }
}
