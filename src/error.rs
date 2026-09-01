//! Error types for the YooMoney API.

use std::fmt;

/// Error code returned by the YooMoney API in the `"error"` field of a JSON response.
///
/// Well-known codes have named variants; anything unknown falls back to [`ApiErrorCode::Other`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ApiErrorCode {
    /// `invalid_request` — malformed request.
    InvalidRequest,
    /// `unauthorized_client` — client is not authorized to use this grant type.
    UnauthorizedClient,
    /// `invalid_grant` — invalid or expired authorization code / token.
    InvalidGrant,
    /// `illegal_param_type` — invalid `type` parameter in operation-history.
    IllegalParamType,
    /// `illegal_param_start_record` — invalid `start_record` parameter.
    IllegalParamStartRecord,
    /// `illegal_param_records` — invalid `records` parameter.
    IllegalParamRecords,
    /// `illegal_param_label` — invalid `label` parameter.
    IllegalParamLabel,
    /// `illegal_param_from` — invalid `from` parameter (bad datetime).
    IllegalParamFrom,
    /// `illegal_param_till` — invalid `till` parameter (bad datetime).
    IllegalParamTill,
    /// `illegal_param_operation_id` — invalid `operation_id` parameter.
    IllegalParamOperationId,
    /// `insufficient_scope` — token lacks rights for this operation (HTTP 403).
    InsufficientScope,
    /// `invalid_scope` — missing or invalid `scope` in the authorize request.
    InvalidScope,
    /// `access_denied` — user rejected the authorization request.
    AccessDenied,
    /// Any other error code string.
    Other(String),
}

impl std::str::FromStr for ApiErrorCode {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "invalid_request" => Self::InvalidRequest,
            "unauthorized_client" => Self::UnauthorizedClient,
            "invalid_grant" => Self::InvalidGrant,
            "illegal_param_type" => Self::IllegalParamType,
            "illegal_param_start_record" => Self::IllegalParamStartRecord,
            "illegal_param_records" => Self::IllegalParamRecords,
            "illegal_param_label" => Self::IllegalParamLabel,
            "illegal_param_from" => Self::IllegalParamFrom,
            "illegal_param_till" => Self::IllegalParamTill,
            "illegal_param_operation_id" => Self::IllegalParamOperationId,
            "insufficient_scope" => Self::InsufficientScope,
            "invalid_scope" => Self::InvalidScope,
            "access_denied" => Self::AccessDenied,
            other => Self::Other(other.to_string()),
        })
    }
}

impl ApiErrorCode {
    /// The raw error code string as returned by the API.
    pub fn as_str(&self) -> &str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::UnauthorizedClient => "unauthorized_client",
            Self::InvalidGrant => "invalid_grant",
            Self::IllegalParamType => "illegal_param_type",
            Self::IllegalParamStartRecord => "illegal_param_start_record",
            Self::IllegalParamRecords => "illegal_param_records",
            Self::IllegalParamLabel => "illegal_param_label",
            Self::IllegalParamFrom => "illegal_param_from",
            Self::IllegalParamTill => "illegal_param_till",
            Self::IllegalParamOperationId => "illegal_param_operation_id",
            Self::InsufficientScope => "insufficient_scope",
            Self::InvalidScope => "invalid_scope",
            Self::AccessDenied => "access_denied",
            Self::Other(s) => s,
        }
    }
}

impl fmt::Display for ApiErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A YooMoney API error: the API responded with a JSON object containing an `"error"` key.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("yoomoney api error: {code}")]
pub struct ApiError {
    /// The error code returned by the API.
    pub code: ApiErrorCode,
}

impl ApiError {
    pub(crate) fn from_code(code: &str) -> Self {
        Self {
            code: code.parse().unwrap(),
        }
    }
}

/// All errors produced by this crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The API returned an error code in its JSON response.
    #[error(transparent)]
    Api(#[from] ApiError),
    /// `account-info` returned an empty response — almost always an invalid/expired token.
    #[error("invalid token: account-info returned an empty response")]
    InvalidToken,
    /// The token exchange response contained no `access_token`.
    #[error("empty token: authorization code exchange returned no access_token")]
    EmptyToken,
    /// The authorization callback URL contained no `code` parameter.
    #[error("no `code` parameter found in the authorization callback URL")]
    MissingCode,
    /// The notification body is malformed or has an invalid signature format.
    #[error("invalid notification body")]
    InvalidNotification,
    /// HTTP request failed.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    /// Failed to parse the JSON response.
    #[error("failed to parse response: {0}")]
    Json(#[from] serde_json::Error),
    /// Failed to build a URL.
    #[error("url error: {0}")]
    Url(#[from] url::ParseError),
}

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;
