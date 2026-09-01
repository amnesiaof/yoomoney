//! OAuth 2.0 authorization flow.

use crate::error::{ApiError, Error, Result};
use serde::Deserialize;
use url::Url;

const AUTHORIZE_URL: &str = "https://yoomoney.ru/oauth/authorize";
const TOKEN_URL: &str = "https://yoomoney.ru/oauth/token";

/// Access token scopes (permissions) that can be requested during authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Scope {
    /// View account info and balance.
    AccountInfo,
    /// View operation history.
    OperationHistory,
    /// View operation details.
    OperationDetails,
    /// View and accept incoming transfers.
    IncomingTransfers,
    /// Make P2P transfers.
    PaymentP2p,
    /// Pay to shops.
    PaymentShop,
}

impl Scope {
    /// Scope string as used by the API, e.g. `account-info`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AccountInfo => "account-info",
            Self::OperationHistory => "operation-history",
            Self::OperationDetails => "operation-details",
            Self::IncomingTransfers => "incoming-transfers",
            Self::PaymentP2p => "payment-p2p",
            Self::PaymentShop => "payment-shop",
        }
    }
}

/// OAuth application credentials.
#[derive(Debug, Clone)]
pub struct Authorize {
    client_id: String,
    redirect_uri: String,
    client_secret: Option<String>,
    instance_name: Option<String>,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct TokenResponse {
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

impl Authorize {
    /// Create an authorization helper for your application.
    ///
    /// `client_secret` is only required for applications registered with a secret.
    pub fn new(
        client_id: impl Into<String>,
        redirect_uri: impl Into<String>,
        client_secret: Option<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            redirect_uri: redirect_uri.into(),
            client_secret,
            instance_name: None,
            http: reqwest::Client::new(),
        }
    }

    /// Optional authorization instance name (e.g. the user's login in your app).
    ///
    /// Allows several independent authorizations of the same application for one user;
    /// without it a repeated authorization revokes the previous one.
    pub fn instance_name(mut self, name: impl Into<String>) -> Self {
        self.instance_name = Some(name.into());
        self
    }

    /// Build the authorization URL for the given scopes.
    ///
    /// Show this URL to the user (open in a browser); after confirming,
    /// YooMoney redirects to `redirect_uri` with a `code` query parameter.
    pub fn authorize_url(&self, scopes: &[Scope]) -> Result<String> {
        let mut params = vec![
            ("client_id", self.client_id.as_str()),
            ("response_type", "code"),
            ("redirect_uri", self.redirect_uri.as_str()),
        ];
        let scope = scopes
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        if !scope.is_empty() {
            params.push(("scope", &scope));
        }
        if let Some(name) = &self.instance_name {
            params.push(("instance_name", name));
        }
        let url = Url::parse_with_params(AUTHORIZE_URL, &params)?;
        Ok(url.into())
    }

    /// Extract the authorization `code` from a redirect URL and exchange it for an access token.
    pub async fn handle_redirect(&self, redirect_url: &str) -> Result<String> {
        let url = Url::parse(redirect_url)?;
        if let Some((_, error)) = url.query_pairs().find(|(k, _)| k == "error") {
            return Err(ApiError::from_code(&error).into());
        }
        let code = url
            .query_pairs()
            .find(|(k, _)| k == "code")
            .map(|(_, v)| v.into_owned())
            .ok_or(Error::MissingCode)?;
        self.exchange_code(&code).await
    }

    /// Exchange a temporary authorization code for an access token.
    pub async fn exchange_code(&self, code: &str) -> Result<String> {
        let mut params = vec![
            ("code", code),
            ("client_id", self.client_id.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", self.redirect_uri.as_str()),
        ];
        if let Some(secret) = &self.client_secret {
            params.push(("client_secret", secret));
        }
        let resp = self.http.post(TOKEN_URL).form(&params).send().await?;
        let body: TokenResponse = resp.json().await?;
        if let Some(error) = body.error {
            return Err(ApiError::from_code(&error).into());
        }
        body.access_token.ok_or(Error::EmptyToken)
    }
}
