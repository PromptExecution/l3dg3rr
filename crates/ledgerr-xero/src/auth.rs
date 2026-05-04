/// Xero OAuth2 PKCE authentication flow — manual implementation using blocking reqwest.
///
/// Avoids the `oauth2` crate to keep everything synchronous without adapter hacks.
use std::path::PathBuf;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Duration, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use crate::error::{XeroError, XeroResult};

const AUTH_URL: &str = "https://login.xero.com/identity/connect/authorize";
const TOKEN_URL: &str = "https://identity.xero.com/connect/token";
const CONNECTIONS_URL: &str = "https://api.xero.com/connections";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XeroConfig {
    pub client_id: String,
    pub client_secret: String,
    #[serde(default = "default_port")]
    pub redirect_port: u16,
    #[serde(default = "default_scopes", skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

fn default_port() -> u16 {
    8080
}

fn default_scopes() -> Vec<String> {
    vec![
        "accounting.contacts.read".into(),
        "accounting.settings.read".into(),
        "accounting.transactions.read".into(),
        "offline_access".into(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XeroTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub tenant_id: Option<String>,
    pub tenant_name: Option<String>,
}

impl XeroTokens {
    pub fn is_expired(&self) -> bool {
        Utc::now() + Duration::seconds(30) >= self.expires_at
    }
}

/// PKCE code verifier and CSRF state, held between get_auth_url() and exchange_code().
struct PkceState {
    code_verifier: String,
    csrf_state: String,
}

pub struct XeroAuth {
    pub config: XeroConfig,
    pub token_path: PathBuf,
    pending: Option<PkceState>,
}

impl XeroAuth {
    pub fn new(config: XeroConfig, token_path: PathBuf) -> Self {
        Self {
            config,
            token_path,
            pending: None,
        }
    }

    pub fn get_auth_url(&mut self) -> XeroResult<String> {
        let code_verifier = pkce_verifier();
        let code_challenge = pkce_challenge(&code_verifier);
        let csrf_state = random_state();

        let redirect_uri = format!(
            "http://localhost:{}/xero/callback",
            self.config.redirect_port
        );

        let scopes = if self.config.scopes.is_empty() {
            default_scopes().join(" ")
        } else {
            self.config.scopes.join(" ")
        };

        let mut url = Url::parse(AUTH_URL).map_err(|e| XeroError::Auth(e.to_string()))?;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("scope", &scopes)
            .append_pair("state", &csrf_state)
            .append_pair("code_challenge", &code_challenge)
            .append_pair("code_challenge_method", "S256");

        self.pending = Some(PkceState {
            code_verifier,
            csrf_state,
        });

        Ok(url.to_string())
    }

    pub fn exchange_code(&mut self, code: String, state: String) -> XeroResult<XeroTokens> {
        let pending = self
            .pending
            .take()
            .ok_or_else(|| XeroError::Auth("No pending auth flow".into()))?;

        if state != pending.csrf_state {
            return Err(XeroError::Auth("CSRF state mismatch".into()));
        }

        let redirect_uri = format!(
            "http://localhost:{}/xero/callback",
            self.config.redirect_port
        );

        let http = reqwest::blocking::Client::new();

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<u64>,
        }

        let resp: TokenResponse = http
            .post(TOKEN_URL)
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", &code),
                ("redirect_uri", &redirect_uri),
                ("code_verifier", &pending.code_verifier),
            ])
            .send()?
            .json()?;

        let expires_in = resp.expires_in.unwrap_or(1800);
        let expires_at = Utc::now() + Duration::seconds(expires_in as i64);

        let (tenant_id, tenant_name) = fetch_tenant_blocking(&resp.access_token, &http)?;

        let tokens = XeroTokens {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            expires_at,
            tenant_id: Some(tenant_id),
            tenant_name: Some(tenant_name),
        };

        self.save_tokens(&tokens)?;
        Ok(tokens)
    }

    pub fn refresh_tokens(&self, tokens: &XeroTokens) -> XeroResult<XeroTokens> {
        let refresh = tokens
            .refresh_token
            .as_deref()
            .ok_or_else(|| XeroError::Auth("No refresh token".into()))?;

        let http = reqwest::blocking::Client::new();

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<u64>,
        }

        let resp: TokenResponse = http
            .post(TOKEN_URL)
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .form(&[("grant_type", "refresh_token"), ("refresh_token", refresh)])
            .send()?
            .json()
            .map_err(|e| XeroError::TokenExpired(e.to_string()))?;

        let expires_in = resp.expires_in.unwrap_or(1800);
        let expires_at = Utc::now() + Duration::seconds(expires_in as i64);

        let new_tokens = XeroTokens {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token.or_else(|| tokens.refresh_token.clone()),
            expires_at,
            tenant_id: tokens.tenant_id.clone(),
            tenant_name: tokens.tenant_name.clone(),
        };

        self.save_tokens(&new_tokens)?;
        Ok(new_tokens)
    }

    pub fn load_tokens(&self) -> XeroResult<Option<XeroTokens>> {
        if !self.token_path.exists() {
            return Ok(None);
        }
        let raw = std::fs::read_to_string(&self.token_path)?;
        Ok(Some(serde_json::from_str(&raw)?))
    }

    pub fn save_tokens(&self, tokens: &XeroTokens) -> XeroResult<()> {
        if let Some(parent) = self.token_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.token_path, serde_json::to_string_pretty(tokens)?)?;
        Ok(())
    }
}

// ── PKCE helpers ──────────────────────────────────────────────────────────────

fn pkce_verifier() -> String {
    let mut buf = [0u8; 32];
    rand::rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

fn pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn random_state() -> String {
    let mut buf = [0u8; 16];
    rand::rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

// ── Tenant fetch ──────────────────────────────────────────────────────────────

fn fetch_tenant_blocking(
    access_token: &str,
    http: &reqwest::blocking::Client,
) -> XeroResult<(String, String)> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Connection {
        tenant_id: String,
        tenant_name: String,
    }

    let resp: Vec<Connection> = http
        .get(CONNECTIONS_URL)
        .bearer_auth(access_token)
        .send()?
        .json()?;

    resp.into_iter()
        .next()
        .map(|c| (c.tenant_id, c.tenant_name))
        .ok_or(XeroError::NoTenant)
}
