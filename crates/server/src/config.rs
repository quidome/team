//! Settings, read once at start. The names match the cluster contract (see `README.md`).

use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

use url::Url;

const MIN_SESSION_SECRET_LEN: usize = 32;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("setting {0} is required")]
    Missing(&'static str),
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
    pub origin: Url,
    pub session_ttl: Duration,
    pub access_role: String,
    pub host: IpAddr,
    pub port: u16,
    pub web_dir: PathBuf,
    /// Only ever true in debug builds (decision 0007).
    pub dev_auth_bypass: bool,
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
            .field("dev_auth_bypass", &self.dev_auth_bypass)
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
        let dev_auth_bypass = cfg!(debug_assertions) && dev_auth_bypass_setting;
        if dev_auth_bypass_setting && !dev_auth_bypass {
            tracing::warn!(
                "DEV_AUTH_BYPASS is set but ignored: release builds always require login"
            );
        }

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            oidc_issuer_url: parse_url("OIDC_ISSUER_URL", &required("OIDC_ISSUER_URL")?)?,
            oidc_client_id: required("OIDC_CLIENT_ID")?,
            oidc_client_secret: required("OIDC_CLIENT_SECRET")?,
            session_secret,
            origin: parse_url("ORIGIN", &required("ORIGIN")?)?,
            session_ttl: Duration::from_secs(session_ttl_seconds),
            access_role: optional("TEAM_ACCESS_ROLE")
                .unwrap_or_else(|| "team_coordinator".to_owned()),
            host,
            port,
            web_dir: optional("WEB_DIR").map_or_else(|| PathBuf::from("/app/web"), PathBuf::from),
            dev_auth_bypass,
        })
    }
}

fn parse_url(name: &'static str, value: &str) -> Result<Url, ConfigError> {
    Url::parse(value).map_err(|error| ConfigError::Invalid {
        name,
        reason: error.to_string(),
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
        assert!(!config.dev_auth_bypass);
        Ok(())
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
    fn debug_output_hides_secrets() -> Result<(), ConfigError> {
        let output = format!("{:?}", settings(&[])?);
        assert!(!output.contains("client-secret"));
        assert!(!output.contains("0123456789abcdef"));
        assert!(!output.contains("postgresql://"));
        Ok(())
    }
}
