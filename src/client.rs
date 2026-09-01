//! Authorized wallet API client.

use crate::error::{ApiError, ApiErrorCode, Error, Result};
use crate::models::{Account, History, OperationDetails};
use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::Value;
use url::Url;

const BASE_URL: &str = "https://yoomoney.ru/api/";

/// Push `key=value` onto the form if the value is present.
pub(crate) fn push_opt<T: ToString>(
    form: &mut Vec<(String, String)>,
    key: &str,
    value: &Option<T>,
) {
    if let Some(v) = value {
        form.push((key.to_string(), v.to_string()));
    }
}

/// Parameters for [`Client::operation_history`]. All fields optional.
#[derive(Debug, Clone, Default)]
pub struct HistoryQuery {
    /// Operation types to show, e.g. `payment`, `deposition` (space-separated for several).
    pub type_: Option<String>,
    /// Filter by payment label.
    pub label: Option<String>,
    /// Show operations from this moment (inclusive).
    pub from: Option<DateTime<Utc>>,
    /// Show operations up to this moment (exclusive).
    pub till: Option<DateTime<Utc>>,
    /// Record to start from (for pagination, take from [`History::next_record`]).
    pub start_record: Option<String>,
    /// Number of records per page, 1..=100, default 30.
    pub records: Option<u32>,
    /// Include detailed info (requires the `operation-details` scope).
    pub details: Option<bool>,
}

/// YooMoney wallet API client.
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    token: String,
    base_url: Url,
}

impl Client {
    /// Create a client with an access token obtained via [`crate::Authorize`].
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            token: token.into(),
            base_url: Url::parse(BASE_URL).expect("valid base url"),
        }
    }

    /// Account info and balance. Requires the `account-info` scope.
    pub async fn account_info(&self) -> Result<Account> {
        let value = self.post("account-info", Vec::new()).await?;
        if value.as_object().is_some_and(|o| o.is_empty()) {
            return Err(Error::InvalidToken);
        }
        Ok(serde_json::from_value(value)?)
    }

    /// Operation history page. Requires the `operation-history` scope.
    pub async fn operation_history(&self, query: &HistoryQuery) -> Result<History> {
        let mut form: Vec<(String, String)> = Vec::new();
        push_opt(&mut form, "type", &query.type_);
        push_opt(&mut form, "label", &query.label);
        push_opt(&mut form, "from", &query.from.map(format_rfc3339));
        push_opt(&mut form, "till", &query.till.map(format_rfc3339));
        push_opt(&mut form, "start_record", &query.start_record);
        push_opt(&mut form, "records", &query.records);
        push_opt(&mut form, "details", &query.details);
        let value = self.post("operation-history", form).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Detailed information about one operation. Requires the `operation-details` scope.
    pub async fn operation_details(&self, operation_id: &str) -> Result<OperationDetails> {
        let value = self
            .post(
                "operation-details",
                vec![("operation_id".to_string(), operation_id.to_string())],
            )
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Revoke this access token. All rights granted to it are cancelled.
    ///
    /// Returns `Err(Error::InvalidToken)` if the token does not exist or is already revoked.
    pub async fn revoke_token(&self) -> Result<()> {
        let url = self.base_url.join("revoke")?;
        let resp = self.http.post(url).bearer_auth(&self.token).send().await?;
        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::InvalidToken);
        }
        resp.error_for_status()?;
        Ok(())
    }

    pub(crate) async fn post(&self, method: &str, form: Vec<(String, String)>) -> Result<Value> {
        let url = self.base_url.join(method)?;
        let resp = self
            .http
            .post(url)
            .bearer_auth(&self.token)
            .form(&form)
            .send()
            .await?;
        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::InvalidToken);
        }
        if resp.status() == reqwest::StatusCode::FORBIDDEN {
            return Err(ApiError {
                code: ApiErrorCode::InsufficientScope,
            }
            .into());
        }
        let value: Value = resp.json().await?;
        if let Some(error) = value.get("error").and_then(|e| e.as_str()) {
            return Err(ApiError::from_code(error).into());
        }
        Ok(value)
    }
}

fn format_rfc3339(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Millis, false)
}
