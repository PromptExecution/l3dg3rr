/// Xero API HTTP client with automatic token refresh (blocking).
use std::path::PathBuf;

use serde_json::Value;
use tracing::debug;

use crate::{
    auth::{XeroAuth, XeroConfig, XeroTokens},
    error::{XeroError, XeroResult},
    types::{
        AccountsResponse, ContactsResponse, InvoicesResponse, XeroAccount, XeroBankAccount,
        XeroContact, XeroInvoice, XeroTenant,
    },
};

const API_BASE: &str = "https://api.xero.com/api.xro/2.0";

pub struct XeroClient {
    auth: XeroAuth,
    http: reqwest::blocking::Client,
    tokens: Option<XeroTokens>,
}

impl XeroClient {
    pub fn new(config: XeroConfig, token_path: PathBuf) -> XeroResult<Self> {
        let auth = XeroAuth::new(config, token_path);
        let tokens = auth.load_tokens()?;
        let http = reqwest::blocking::Client::builder()
            .user_agent(concat!("ledgerr/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { auth, http, tokens })
    }

    pub fn is_authenticated(&self) -> bool {
        self.tokens
            .as_ref()
            .map(|t| !t.is_expired())
            .unwrap_or(false)
    }

    pub fn get_auth_url(&mut self) -> XeroResult<String> {
        self.auth.get_auth_url()
    }

    pub fn exchange_code(&mut self, code: String, state: String) -> XeroResult<XeroTenant> {
        let tokens = self.auth.exchange_code(code, state)?;
        let tenant = XeroTenant {
            tenant_id: tokens.tenant_id.clone().unwrap_or_default(),
            tenant_name: tokens.tenant_name.clone().unwrap_or_default(),
            tenant_type: "ORGANISATION".into(),
        };
        self.tokens = Some(tokens);
        Ok(tenant)
    }

    fn access_token(&mut self) -> XeroResult<String> {
        let tokens = self
            .tokens
            .as_ref()
            .ok_or(XeroError::NotAuthenticated)?
            .clone();

        if tokens.is_expired() {
            debug!("Xero token expired, refreshing");
            let refreshed = self.auth.refresh_tokens(&tokens)?;
            let tok = refreshed.access_token.clone();
            self.tokens = Some(refreshed);
            return Ok(tok);
        }

        Ok(tokens.access_token.clone())
    }

    fn tenant_id(&self) -> XeroResult<String> {
        self.tokens
            .as_ref()
            .and_then(|t| t.tenant_id.clone())
            .ok_or(XeroError::NoTenant)
    }

    fn get<T: serde::de::DeserializeOwned>(
        &mut self,
        path: &str,
        query: Option<&[(&str, &str)]>,
    ) -> XeroResult<T> {
        let token = self.access_token()?;
        let tenant = self.tenant_id()?;
        let url = format!("{API_BASE}/{path}");

        let mut req = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .header("Xero-tenant-id", &tenant)
            .header("Accept", "application/json");

        if let Some(params) = query {
            req = req.query(params);
        }

        let resp = req.send()?;
        let status = resp.status().as_u16();
        if status >= 400 {
            let body = resp.text().unwrap_or_default();
            return Err(XeroError::ApiError { status, message: body });
        }

        Ok(resp.json()?)
    }

    // ── Contacts ──────────────────────────────────────────────────────────────

    pub fn get_contacts(&mut self) -> XeroResult<Vec<XeroContact>> {
        let resp: ContactsResponse = self.get("Contacts", None)?;
        Ok(resp.contacts)
    }

    pub fn search_contacts(&mut self, query: &str) -> XeroResult<Vec<XeroContact>> {
        // Escape double-quotes in the query to prevent filter expression injection.
        let escaped = query.replace('"', "\\\"");
        let where_clause = format!("Name.Contains(\"{escaped}\")");
        let resp: ContactsResponse =
            self.get("Contacts", Some(&[("where", where_clause.as_str())]))?;
        Ok(resp.contacts)
    }

    pub fn get_contact(&mut self, contact_id: &str) -> XeroResult<Option<XeroContact>> {
        let resp: ContactsResponse = self.get(&format!("Contacts/{contact_id}"), None)?;
        Ok(resp.contacts.into_iter().next())
    }

    // ── Accounts ──────────────────────────────────────────────────────────────

    pub fn get_accounts(&mut self) -> XeroResult<Vec<XeroAccount>> {
        let resp: AccountsResponse = self.get("Accounts", None)?;
        Ok(resp.accounts)
    }

    pub fn get_bank_accounts(&mut self) -> XeroResult<Vec<XeroBankAccount>> {
        let resp: AccountsResponse =
            self.get("Accounts", Some(&[("where", "Type==\"BANK\"")]))?;
        Ok(resp
            .accounts
            .into_iter()
            .map(|a| XeroBankAccount {
                account_id: a.account_id,
                name: a.name,
                bank_account_number: None,
                bank_account_type: Some(a.account_type),
                currency_code: a.currency_code.unwrap_or_else(|| "USD".into()),
                status: a.status,
            })
            .collect())
    }

    // ── Invoices ──────────────────────────────────────────────────────────────

    pub fn get_invoices(&mut self, status: Option<&str>) -> XeroResult<Vec<XeroInvoice>> {
        if let Some(s) = status {
            let where_val = format!("Status==\"{s}\"");
            let resp: InvoicesResponse =
                self.get("Invoices", Some(&[("where", where_val.as_str())]))?;
            Ok(resp.invoices)
        } else {
            let resp: InvoicesResponse = self.get("Invoices", None)?;
            Ok(resp.invoices)
        }
    }

    pub fn raw_get(&mut self, path: &str) -> XeroResult<Value> {
        self.get(path, None)
    }
}
