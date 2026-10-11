//! OpenID Connect against Pocket ID (decision 0007): start a login, finish it, read the identity.

use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreErrorResponseType,
    CoreGenderClaim, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm,
    CoreProviderMetadata, CoreRevocableToken, CoreRevocationErrorResponse,
    CoreTokenIntrospectionResponse, CoreTokenType,
};
use openidconnect::{
    AdditionalClaims, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, EndpointMaybeSet, EndpointNotSet, EndpointSet, IdTokenFields, IssuerUrl,
    Nonce, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, StandardErrorResponse,
    StandardTokenResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use team_core::Identity;

use openidconnect::ClaimsVerificationError;
use tokio::sync::RwLock;
use url::Url;

/// The `groups` claim that Pocket ID adds with the `groups` scope.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroupsClaim {
    #[serde(default)]
    pub groups: Vec<String>,
}

impl AdditionalClaims for GroupsClaim {}

pub type GroupsIdTokenFields = IdTokenFields<
    GroupsClaim,
    EmptyExtraTokenFields,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
>;
pub type GroupsTokenResponse = StandardTokenResponse<GroupsIdTokenFields, CoreTokenType>;

type GroupsClient = Client<
    GroupsClaim,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    GroupsTokenResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

#[derive(Debug, thiserror::Error)]
pub enum OidcError {
    /// Pocket ID cannot be reached or gives no usable description.
    #[error("login server is not available: {0}")]
    Unavailable(String),
    /// The answer does not belong to this login, or does not check out.
    #[error("login answer is not valid: {0}")]
    Invalid(String),
}

/// What the browser must remember (in the login cookie) and where it must go.
pub struct StartedLogin {
    pub authorize_url: Url,
    pub state: String,
    pub nonce: String,
    pub pkce_verifier: String,
}

pub struct Oidc {
    issuer: IssuerUrl,
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect: RedirectUrl,
    http: reqwest::Client,
    /// The issuer's description and signing keys: fetched at the first login and kept, and
    /// fetched again when an ID token's signature does not match the kept keys (key rotation).
    client: RwLock<Option<Arc<GroupsClient>>>,
}

impl Oidc {
    pub fn new(
        issuer: &Url,
        client_id: &str,
        client_secret: &str,
        origin: &Url,
    ) -> Result<Self, OidcError> {
        let redirect = origin
            .join("/auth/callback")
            .map_err(|error| OidcError::Invalid(error.to_string()))?;
        // No redirects: the login server answers directly (protects against being sent elsewhere).
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|error| OidcError::Unavailable(error.to_string()))?;
        Ok(Self {
            issuer: IssuerUrl::new(issuer.as_str().trim_end_matches('/').to_owned())
                .map_err(|error| OidcError::Invalid(error.to_string()))?,
            client_id: ClientId::new(client_id.to_owned()),
            client_secret: ClientSecret::new(client_secret.to_owned()),
            redirect: RedirectUrl::from_url(redirect),
            http,
            client: RwLock::new(None),
        })
    }

    /// The kept client, fetched on first use.
    async fn client(&self) -> Result<Arc<GroupsClient>, OidcError> {
        if let Some(client) = self.client.read().await.as_ref() {
            return Ok(Arc::clone(client));
        }
        self.refresh().await
    }

    /// Fetches the issuer's description and keys again, and keeps the result.
    async fn refresh(&self) -> Result<Arc<GroupsClient>, OidcError> {
        let metadata = CoreProviderMetadata::discover_async(self.issuer.clone(), &self.http)
            .await
            .map_err(|error| OidcError::Unavailable(error.to_string()))?;
        let client = Arc::new(
            Client::from_provider_metadata(
                metadata,
                self.client_id.clone(),
                Some(self.client_secret.clone()),
            )
            .set_redirect_uri(self.redirect.clone()),
        );
        *self.client.write().await = Some(Arc::clone(&client));
        Ok(client)
    }

    pub async fn start(&self) -> Result<StartedLogin, OidcError> {
        let client = self.client().await?;
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorize_url, state, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("profile".to_owned()))
            .add_scope(Scope::new("groups".to_owned()))
            .set_pkce_challenge(challenge)
            .url();
        Ok(StartedLogin {
            authorize_url,
            state: state.secret().clone(),
            nonce: nonce.secret().clone(),
            pkce_verifier: verifier.secret().clone(),
        })
    }

    /// Exchanges the code and checks the ID token (issuer, audience, nonce, signature, expiry).
    pub async fn finish(
        &self,
        code: String,
        nonce: String,
        pkce_verifier: String,
    ) -> Result<Identity, OidcError> {
        let client = self.client().await?;
        let response = client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|error| OidcError::Invalid(error.to_string()))?
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
            .request_async(&self.http)
            .await
            .map_err(|error| match error {
                openidconnect::RequestTokenError::Request(error) => {
                    OidcError::Unavailable(error.to_string())
                }
                other => OidcError::Invalid(other.to_string()),
            })?;

        let id_token = response
            .extra_fields()
            .id_token()
            .ok_or_else(|| OidcError::Invalid("no ID token in the answer".to_owned()))?;
        let nonce = Nonce::new(nonce);
        let claims = match id_token.claims(&client.id_token_verifier(), &nonce) {
            Ok(claims) => claims,
            // The login server may have a new signing key: fetch the keys again, once.
            Err(ClaimsVerificationError::SignatureVerification(first)) => {
                tracing::info!(error = %first, "ID token signature not verified; fetching the keys again");
                let refreshed = self.refresh().await?;
                id_token
                    .claims(&refreshed.id_token_verifier(), &nonce)
                    .map_err(|error| OidcError::Invalid(error.to_string()))?
            }
            Err(error) => return Err(OidcError::Invalid(error.to_string())),
        };

        let subject = claims.subject().as_str().to_owned();
        let name = claims
            .name()
            .and_then(|name| name.get(None))
            .map(|name| name.as_str().to_owned())
            .or_else(|| {
                claims
                    .preferred_username()
                    .map(|name| name.as_str().to_owned())
            })
            .unwrap_or_else(|| subject.clone());
        Ok(Identity {
            subject,
            name,
            groups: claims.additional_claims().groups.clone(),
        })
    }
}
