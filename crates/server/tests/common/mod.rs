//! Shared helpers for the integration tests: settings, a settable clock, a fake Pocket ID, and
//! request helpers.

#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

pub mod api;
pub mod game_list;

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, header};
use chrono::{DateTime, TimeDelta, Utc};
use http_body_util::BodyExt;
use openidconnect::core::{
    CoreGenderClaim, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm,
    CoreRsaPrivateSigningKey,
};
use openidconnect::{
    Audience, EndUserName, IdToken, IdTokenClaims, IssuerUrl, JsonWebKeyId, LocalizedClaim, Nonce,
    PrivateSigningKey, StandardClaims, SubjectIdentifier,
};
use sqlx::PgPool;
use team::auth::oidc::{GroupsClaim, Oidc};
use team::clock::Clock;
use team::config::Config;
use team::source::Source;
use team::web::Web;
use team::{AppState, app};
use tower::ServiceExt;
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub const ORIGIN: &str = "https://team.example.test";
pub const CLIENT_ID: &str = "team-coordinator";
pub const CLIENT_SECRET: &str = "client-secret-for-tests";
pub const SESSION_SECRET: &str = "0123456789abcdef0123456789abcdef";
pub const SIGNING_KEY: &str = include_str!("../data/test-signing-key.pem");
pub const OTHER_KEY: &str = include_str!("../data/test-other-key.pem");
/// The inline start script of the test front end; its hash must be in the security policy.
pub const START_SCRIPT: &str = "window.started = true";
pub const INDEX_HTML: &str = "<!doctype html><html><head><script src=\"/_app/start.js\"></script></head>\
     <body><div id=\"app\"></div><script>window.started = true</script></body></html>";

/// A small built front end in a temporary folder.
pub fn front_end() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), INDEX_HTML).unwrap();
    std::fs::create_dir(dir.path().join("_app")).unwrap();
    std::fs::write(dir.path().join("_app/start.js"), "console.log('start');").unwrap();
    dir
}

pub fn config_with(issuer: &str, overrides: &[(&str, &str)]) -> Config {
    let issuer = issuer.to_owned();
    let overrides: Vec<(String, String)> = overrides
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    Config::from_lookup(move |name| {
        if let Some((_, value)) = overrides.iter().find(|(key, _)| key == name) {
            return Some(value.clone());
        }
        let value = match name {
            "DATABASE_URL" => "postgresql://unused-in-tests",
            "OIDC_ISSUER_URL" => issuer.as_str(),
            "OIDC_CLIENT_ID" => CLIENT_ID,
            "OIDC_CLIENT_SECRET" => CLIENT_SECRET,
            "SESSION_SECRET" => SESSION_SECRET,
            "ORIGIN" => ORIGIN,
            _ => return None,
        };
        Some(value.to_owned())
    })
    .unwrap_or_else(|error| panic!("test settings are valid: {error}"))
}

/// A clock that tests move by hand.
pub struct TestClock(Mutex<DateTime<Utc>>);

impl TestClock {
    pub fn new() -> Arc<Self> {
        Arc::new(Self(Mutex::new(Utc::now())))
    }

    pub fn set(&self, at: DateTime<Utc>) {
        *self.0.lock().unwrap() = at;
    }

    pub fn advance(&self, delta: TimeDelta) {
        let mut now = self.0.lock().unwrap();
        *now += delta;
    }
}

impl Clock for TestClock {
    fn now(&self) -> DateTime<Utc> {
        *self.0.lock().unwrap()
    }
}

/// The router with a fake Pocket ID behind it.
pub struct Harness {
    pub router: Router,
    /// Kept so the temporary front end lives as long as the harness.
    pub web_dir: tempfile::TempDir,
    pub clock: Arc<TestClock>,
    pub pocket_id: MockServer,
    /// The fake game list source; each test mounts the answers it needs.
    pub source: MockServer,
    pub pool: PgPool,
}

/// What `/auth/login` gave: the login cookie, and the state and nonce from the redirect.
pub struct StartedLogin {
    pub login_cookie: String,
    pub state: String,
    pub nonce: String,
    pub location: Url,
}

impl Harness {
    pub async fn start(pool: PgPool) -> Self {
        Self::start_with(pool, &[]).await
    }

    pub async fn start_with(pool: PgPool, overrides: &[(&str, &str)]) -> Self {
        let pocket_id = MockServer::start().await;
        let issuer = pocket_id.uri();
        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(discovery(&issuer)))
            .mount(&pocket_id)
            .await;
        Mock::given(method("GET"))
            .and(path("/jwks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(jwks()))
            .mount(&pocket_id)
            .await;

        let source = MockServer::start().await;
        let mut config = config_with(&issuer, overrides);
        config.game_list_url = Url::parse(&source.uri()).unwrap();
        let clock = TestClock::new();
        let oidc = Oidc::new(
            &config.oidc_issuer_url,
            &config.oidc_client_id,
            &config.oidc_client_secret,
            &config.origin,
        )
        .unwrap();
        let reader = Source::new(config.game_list_url.clone(), &config.origin).unwrap();
        let web_dir = front_end();
        let router = app(AppState {
            pool: pool.clone(),
            config: Arc::new(config),
            oidc: Arc::new(oidc),
            clock: clock.clone(),
            web: Arc::new(Web::load(web_dir.path())),
            source: Arc::new(reader),
        });
        Self {
            router,
            web_dir,
            clock,
            pocket_id,
            source,
            pool,
        }
    }

    pub fn issuer(&self) -> String {
        self.pocket_id.uri()
    }

    pub async fn send(&self, request: Request<Body>) -> Response<Body> {
        self.router.clone().oneshot(request).await.unwrap()
    }

    pub async fn get(&self, path: &str, cookie: Option<&str>) -> Response<Body> {
        let mut request = Request::get(path);
        if let Some(cookie) = cookie {
            request = request.header(header::COOKIE, cookie);
        }
        self.send(request.body(Body::empty()).unwrap()).await
    }

    pub async fn start_login(&self) -> StartedLogin {
        let response = self.get("/auth/login", None).await;
        assert_eq!(response.status(), 303, "login redirects");
        let location = Url::parse(response.headers()[header::LOCATION].to_str().unwrap()).unwrap();
        let query = |name: &str| {
            location
                .query_pairs()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.into_owned())
                .unwrap()
        };
        StartedLogin {
            login_cookie: cookie_value(&response, "team_login").unwrap(),
            state: query("state"),
            nonce: query("nonce"),
            location,
        }
    }

    /// Makes the fake Pocket ID answer the next code exchange with this ID token.
    pub async fn answer_token(&self, id_token: &str) {
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "access-token-for-tests",
                "token_type": "Bearer",
                "expires_in": 3600,
                "id_token": id_token,
            })))
            .up_to_n_times(1)
            .mount(&self.pocket_id)
            .await;
    }

    pub async fn callback(&self, login: &StartedLogin, state: &str) -> Response<Body> {
        let path = format!("/auth/callback?code=code-for-tests&state={state}");
        self.get(&path, Some(&format!("team_login={}", login.login_cookie)))
            .await
    }

    /// A full login with these groups. Returns the session cookie value.
    pub async fn log_in(&self, groups: &[&str]) -> String {
        let login = self.start_login().await;
        let token = TokenSpec::new(&self.issuer(), &login.nonce, groups).sign();
        self.answer_token(&token).await;
        let response = self.callback(&login, &login.state).await;
        assert_eq!(response.status(), 303, "callback redirects");
        cookie_value(&response, "team_session").unwrap()
    }
}

fn discovery(issuer: &str) -> serde_json::Value {
    serde_json::json!({
        "issuer": issuer,
        "authorization_endpoint": format!("{issuer}/authorize"),
        "token_endpoint": format!("{issuer}/token"),
        "jwks_uri": format!("{issuer}/jwks"),
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "scopes_supported": ["openid", "profile", "groups"],
    })
}

fn signing_key(pem: &str) -> CoreRsaPrivateSigningKey {
    CoreRsaPrivateSigningKey::from_pem(pem, Some(JsonWebKeyId::new("test-key".to_owned()))).unwrap()
}

fn jwks() -> serde_json::Value {
    jwks_for(SIGNING_KEY)
}

/// The key set that the fake Pocket ID publishes, for one private key.
pub fn jwks_for(pem: &str) -> serde_json::Value {
    serde_json::json!({ "keys": [signing_key(pem).as_verification_key()] })
}

/// An ID token to sign; tests change fields to make it wrong.
pub struct TokenSpec {
    pub issuer: String,
    pub audience: String,
    pub nonce: String,
    pub subject: String,
    pub name: String,
    pub groups: Vec<String>,
    pub key: &'static str,
}

impl TokenSpec {
    pub fn new(issuer: &str, nonce: &str, groups: &[&str]) -> Self {
        Self {
            issuer: issuer.to_owned(),
            audience: CLIENT_ID.to_owned(),
            nonce: nonce.to_owned(),
            subject: "pocket-id-user-1".to_owned(),
            name: "Quinn Coordinator".to_owned(),
            groups: groups.iter().map(|group| (*group).to_owned()).collect(),
            key: SIGNING_KEY,
        }
    }

    pub fn sign(&self) -> String {
        let now = Utc::now();
        let claims = IdTokenClaims::new(
            IssuerUrl::new(self.issuer.clone()).unwrap(),
            vec![Audience::new(self.audience.clone())],
            now + TimeDelta::minutes(5),
            now,
            StandardClaims::new(SubjectIdentifier::new(self.subject.clone())).set_name(Some(
                LocalizedClaim::from(EndUserName::new(self.name.clone())),
            )),
            GroupsClaim {
                groups: self.groups.clone(),
            },
        )
        .set_nonce(Some(Nonce::new(self.nonce.clone())));
        let token = IdToken::<
            GroupsClaim,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
        >::new(
            claims,
            &signing_key(self.key),
            CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
            None,
            None,
        )
        .unwrap();
        token.to_string()
    }
}

/// All `Set-Cookie` headers of a response.
pub fn set_cookies(response: &Response<Body>) -> Vec<String> {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| value.to_str().unwrap().to_owned())
        .collect()
}

/// The `Set-Cookie` header for one cookie.
pub fn set_cookie(response: &Response<Body>, name: &str) -> Option<String> {
    set_cookies(response)
        .into_iter()
        .find(|cookie| cookie.starts_with(&format!("{name}=")))
}

/// The value of a cookie that a response sets (not one it removes).
pub fn cookie_value(response: &Response<Body>, name: &str) -> Option<String> {
    let cookie = set_cookie(response, name)?;
    let value = cookie
        .split(';')
        .next()?
        .strip_prefix(&format!("{name}="))?
        .to_owned();
    (!value.is_empty()).then_some(value)
}

pub async fn body_text(response: Response<Body>) -> String {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Captures log output written while the guard lives. A test that reads logs needs its own test
/// file (its own process): tracing caches per process which log lines are enabled.
#[derive(Clone, Default)]
pub struct LogBuffer(Arc<Mutex<Vec<u8>>>);

impl LogBuffer {
    /// Captures every log line of this thread until the guard is dropped.
    pub fn capture() -> (Self, tracing::subscriber::DefaultGuard) {
        let buffer = Self::default();
        let writer = buffer.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_writer(move || writer.clone())
            .with_ansi(false)
            .finish();
        (buffer, tracing::subscriber::set_default(subscriber))
    }

    pub fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

impl std::io::Write for LogBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
