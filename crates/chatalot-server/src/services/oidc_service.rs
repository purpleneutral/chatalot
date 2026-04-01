use std::time::{Duration, Instant};

use dashmap::DashMap;
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};

/// Information extracted from OIDC ID token claims.
#[derive(Debug, Clone)]
pub struct OidcUserInfo {
    pub subject: String,
    pub email: Option<String>,
    pub preferred_username: Option<String>,
    pub name: Option<String>,
}

/// Pending OIDC authorization state (PKCE + nonce + TTL).
struct PendingState {
    pkce_verifier: PkceCodeVerifier,
    nonce: Nonce,
    created_at: Instant,
}

/// Max age for pending OIDC states (5 minutes).
const STATE_TTL: Duration = Duration::from_secs(300);

pub struct OidcService {
    client: CoreClient,
    pending_states: DashMap<String, PendingState>,
}

impl OidcService {
    /// Create a new OIDC service by fetching the provider's discovery document.
    pub async fn new(
        issuer_url: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
    ) -> Result<Self, String> {
        let issuer = IssuerUrl::new(issuer_url.to_string())
            .map_err(|e| format!("invalid OIDC issuer URL: {e}"))?;

        let metadata = CoreProviderMetadata::discover_async(issuer, async_http_client)
            .await
            .map_err(|e| format!("OIDC discovery failed: {e}"))?;

        let redirect = RedirectUrl::new(redirect_uri.to_string())
            .map_err(|e| format!("invalid OIDC redirect URI: {e}"))?;

        let client = CoreClient::from_provider_metadata(
            metadata,
            ClientId::new(client_id.to_string()),
            Some(ClientSecret::new(client_secret.to_string())),
        )
        .set_redirect_uri(redirect);

        Ok(Self {
            client,
            pending_states: DashMap::new(),
        })
    }

    /// Generate an authorization URL with PKCE + nonce. Returns (url, state_key).
    pub fn generate_auth_url(&self) -> (String, String) {
        // Prune expired states on each call (cheap, bounded by concurrent logins)
        self.cleanup_expired_states();

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token, nonce) = self
            .client
            .authorize_url(
                AuthenticationFlow::<openidconnect::core::CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        let state_key = csrf_token.secret().clone();

        self.pending_states.insert(
            state_key.clone(),
            PendingState {
                pkce_verifier,
                nonce,
                created_at: Instant::now(),
            },
        );

        (auth_url.to_string(), state_key)
    }

    /// Exchange an authorization code for user info. Validates state and nonce.
    pub async fn exchange_code(
        &self,
        code: &str,
        state_key: &str,
    ) -> Result<OidcUserInfo, String> {
        // Remove and validate the pending state
        let pending = self
            .pending_states
            .remove(state_key)
            .ok_or_else(|| "invalid or expired OIDC state".to_string())?
            .1;

        // Check TTL
        if pending.created_at.elapsed() > STATE_TTL {
            return Err("OIDC state expired".to_string());
        }

        // Exchange code for tokens
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(pending.pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|e| format!("OIDC code exchange failed: {e}"))?;

        // Extract and verify the ID token
        let id_token = token_response
            .id_token()
            .ok_or_else(|| "no ID token in OIDC response".to_string())?;

        let verifier = self.client.id_token_verifier();
        let claims = id_token
            .claims(&verifier, &pending.nonce)
            .map_err(|e| format!("OIDC ID token verification failed: {e}"))?;

        let subject = claims.subject().to_string();
        let email = claims
            .email()
            .map(|e| e.to_string());
        let preferred_username = claims
            .preferred_username()
            .map(|u| u.to_string());
        let name = claims
            .name()
            .and_then(|n| n.get(None))
            .map(|n| n.to_string());

        Ok(OidcUserInfo {
            subject,
            email,
            preferred_username,
            name,
        })
    }

    /// Remove expired pending states.
    fn cleanup_expired_states(&self) {
        self.pending_states
            .retain(|_, state| state.created_at.elapsed() < STATE_TTL);
    }
}
