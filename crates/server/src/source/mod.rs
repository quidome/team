//! Reads the game list source: the club list and the games of one club in one season.
//!
//! The source's field names stay in this module (see `parse.rs`). Reading is strict: data that is
//! missing or not understood makes the reading fail with an error that names the place.

mod parse;

use std::time::Duration;

use team_core::{Club, ClubNumber, Game, Season};
use url::Url;

const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    /// No connection, or no answer in time.
    #[error("game list cannot be reached")]
    Unreachable(#[source] reqwest::Error),
    #[error("game list answered {0}")]
    Status(reqwest::StatusCode),
    /// The answer has data the tool does not understand. `place` names the club or game and field.
    #[error("game list format: {place}: {reason}")]
    Format { place: String, reason: String },
    #[error("game list client: {0}")]
    Client(#[source] reqwest::Error),
}

pub struct Source {
    client: reqwest::Client,
    /// The root of the source's site, for example `https://db.basketball.nl`.
    base: Url,
}

impl Source {
    /// `origin` goes into the User-Agent, so the owner of the source can see who reads.
    pub fn new(base: Url, origin: &Url) -> Result<Self, SourceError> {
        Self::with_timeout(base, origin, TIMEOUT)
    }

    /// As [`Source::new`], with another timeout (for tests).
    pub fn with_timeout(base: Url, origin: &Url, timeout: Duration) -> Result<Self, SourceError> {
        let user_agent = format!(
            "team-coordinator/{} (+{})",
            env!("CARGO_PKG_VERSION"),
            origin.as_str().trim_end_matches('/')
        );
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(timeout)
            .build()
            .map_err(SourceError::Client)?;
        Ok(Self { client, base })
    }

    /// All clubs of the source.
    pub async fn clubs(&self) -> Result<Vec<Club>, SourceError> {
        self.get("db/json/club.pl", &[])
            .await
            .and_then(|body| parse::clubs(&body))
            .inspect_err(|error| tracing::warn!(error = %error, "reading the club list failed"))
    }

    /// All games of `club` in `season`: home and away games of all its teams.
    pub async fn club_games(
        &self,
        club: ClubNumber,
        season: Season,
    ) -> Result<Vec<Game>, SourceError> {
        self.get(
            "db/json/wedstrijd.pl",
            &[
                ("seizoen", season.to_string()),
                ("clb_ID", club.0.to_string()),
            ],
        )
        .await
        .and_then(|body| parse::games(&body, season))
        .inspect_err(|error| {
            tracing::warn!(club = club.0, season = %season, error = %error, "reading club games failed");
        })
    }

    async fn get(&self, path: &str, query: &[(&str, String)]) -> Result<Vec<u8>, SourceError> {
        let url = self.base.join(path).map_err(|error| SourceError::Format {
            place: "source address".to_owned(),
            reason: error.to_string(),
        })?;
        let response = self
            .client
            .get(url)
            .query(query)
            .send()
            .await
            .map_err(SourceError::Unreachable)?;
        let status = response.status();
        if !status.is_success() {
            return Err(SourceError::Status(status));
        }
        let body = response.bytes().await.map_err(SourceError::Unreachable)?;
        Ok(body.to_vec())
    }
}
