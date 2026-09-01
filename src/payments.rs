//! Payments: `request-payment` and `process-payment`.

use crate::client::{Client, push_opt};
use crate::error::Result;
use crate::models::DigitalGood;
use serde::Deserialize;
use std::collections::HashMap;

/// A P2P transfer request. See [`Client::request_p2p`].
#[derive(Debug, Clone, Default)]
pub struct P2pTransfer {
    /// Recipient: account number, linked phone or email.
    pub to: String,
    /// Amount the sender pays (including the service fee).
    pub amount: f64,
    /// Comment, shown in the sender's history.
    pub comment: Option<String>,
    /// Message, shown to the recipient.
    pub message: Option<String>,
    /// Payment label (up to 64 chars), searchable in history.
    pub label: Option<String>,
}

/// Payment method for [`ProcessPaymentQuery::money_source`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MoneySource {
    /// Pay from the wallet (default).
    Wallet,
    /// Pay with a linked card (its `id` from [`CardsSource::items`]); send `csc` too.
    Card(String),
}

impl MoneySource {
    /// Value sent as the `money_source` parameter.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Wallet => "wallet",
            Self::Card(id) => id,
        }
    }
}

/// Parameters for [`Client::process_payment`].
#[derive(Debug, Clone, Default)]
pub struct ProcessPaymentQuery {
    /// From [`RequestPayment::request_id`].
    pub request_id: String,
    /// Payment method, default wallet.
    pub money_source: Option<MoneySource>,
    /// CVV2/CVC2 code, required when paying with a linked card.
    pub csc: Option<String>,
    /// Return URL after successful 3-D Secure authentication.
    pub ext_auth_success_uri: Option<String>,
    /// Return URL after failed 3-D Secure authentication.
    pub ext_auth_fail_uri: Option<String>,
}

/// Response of `POST /api/request-payment`.
#[derive(Debug, Clone, Deserialize)]
pub struct RequestPayment {
    /// `success` or `refused` (see [`RequestPayment::error`]).
    pub status: String,
    /// Error code, present when `status` is `refused`
    /// (e.g. `not_enough_funds`, `payee_not_found`, `limit_exceeded`).
    #[serde(default)]
    pub error: Option<String>,
    /// Payment methods available for this payment.
    ///
    /// The API sends this either as a `money_source` object or as top-level
    /// `wallet`/`cards` fields (the docs are inconsistent); use
    /// [`RequestPayment::money_sources`] to read it in either shape.
    #[serde(default, rename = "money_source")]
    money_source_object: Option<MoneySources>,
    #[serde(flatten, default)]
    money_source_fields: Option<MoneySources>,
    /// Pass to [`ProcessPaymentQuery::request_id`].
    #[serde(default)]
    pub request_id: Option<String>,
    /// Human-readable payment description.
    #[serde(default)]
    pub contract: Option<String>,
    /// Amount to be debited (with the fee). Also present on `not_enough_funds`.
    #[serde(default)]
    pub contract_amount: Option<f64>,
    /// Current balance, if the token has the `account-info` scope.
    #[serde(default)]
    pub balance: Option<f64>,
    #[serde(default)]
    pub recipient_account_status: Option<String>,
    #[serde(default)]
    pub recipient_account_type: Option<String>,
    /// Unblock page, present on the `account_blocked` error.
    #[serde(default)]
    pub account_unblock_uri: Option<String>,
    /// Action page, present on the `ext_action_required` error.
    #[serde(default)]
    pub ext_action_uri: Option<String>,
}

/// Available payment methods (`money_source` response field).
#[derive(Debug, Clone, Deserialize)]
pub struct MoneySources {
    #[serde(default)]
    pub wallet: Option<WalletSource>,
    #[serde(default)]
    pub cards: Option<CardsSource>,
}

/// Wallet payment method availability.
#[derive(Debug, Clone, Deserialize)]
pub struct WalletSource {
    pub allowed: bool,
}

/// Linked-card payment method availability.
#[derive(Debug, Clone, Deserialize)]
pub struct CardsSource {
    pub allowed: bool,
    /// Whether CVV2/CVC2 is required for card payments.
    #[serde(default)]
    pub csc_required: Option<bool>,
    /// Linked cards available for payment.
    #[serde(default)]
    pub items: Vec<LinkedCard>,
}

/// A bank card linked to the wallet, available as a money source.
#[derive(Debug, Clone, Deserialize)]
pub struct LinkedCard {
    /// Pass as [`MoneySource::Card`] to [`Client::process_payment`].
    pub id: String,
    #[serde(default)]
    pub pan_fragment: Option<String>,
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
}

/// Response of `POST /api/process-payment`.
#[derive(Debug, Clone, Deserialize)]
pub struct ProcessPayment {
    /// `success`, `refused`, `in_progress` or `ext_auth_required`.
    pub status: String,
    /// Error code, present when `status` is `refused`.
    #[serde(default)]
    pub error: Option<String>,
    /// Payment id, present on success (equals `operation_id` in history).
    #[serde(default)]
    pub payment_id: Option<String>,
    /// Balance after the payment, if the token has the `account-info` scope.
    #[serde(default)]
    pub balance: Option<f64>,
    /// Store transaction number, present for shop payments.
    #[serde(default)]
    pub invoice_id: Option<String>,
    /// Sender account, present for P2P transfers.
    #[serde(default)]
    pub payer: Option<String>,
    /// Recipient account, present for P2P transfers.
    #[serde(default)]
    pub payee: Option<String>,
    /// Amount received, present for P2P transfers.
    #[serde(default)]
    pub credit_amount: Option<f64>,
    /// Unblock page, present on the `account_blocked` error.
    #[serde(default)]
    pub account_unblock_uri: Option<String>,
    /// 3-D Secure page of the issuing bank, present on `ext_auth_required`.
    #[serde(default)]
    pub acs_uri: Option<String>,
    /// 3-D Secure form parameters, POST them to `acs_uri` as form-urlencoded.
    #[serde(default)]
    pub acs_params: Option<HashMap<String, String>>,
    /// Retry delay in milliseconds, present on `in_progress`.
    #[serde(default)]
    pub next_retry: Option<u64>,
    /// Digital goods data, present for digital goods stores.
    #[serde(default)]
    pub digital_goods: Option<DigitalGood>,
}

impl RequestPayment {
    /// True if the payment was accepted for processing.
    pub fn is_success(&self) -> bool {
        self.status == "success"
    }

    /// Payment methods available for this payment.
    pub fn money_sources(&self) -> Option<&MoneySources> {
        self.money_source_object
            .as_ref()
            .or(self.money_source_fields.as_ref())
    }
}

impl ProcessPayment {
    /// True if the payment completed successfully.
    pub fn is_success(&self) -> bool {
        self.status == "success"
    }

    /// True if the call should be repeated (authorization not finished yet).
    pub fn is_in_progress(&self) -> bool {
        self.status == "in_progress"
    }

    /// True if 3-D Secure authentication is required: POST `acs_params` to
    /// `acs_uri`, then call [`Client::process_payment`] again with just `request_id`.
    pub fn is_ext_auth_required(&self) -> bool {
        self.status == "ext_auth_required"
    }
}

impl Client {
    /// Create a P2P transfer. Requires the `payment-p2p` or `payment.to-account` scope.
    ///
    /// The transfer is not executed yet: take [`RequestPayment::request_id`]
    /// and call [`Client::process_payment`].
    pub async fn request_p2p(&self, transfer: &P2pTransfer) -> Result<RequestPayment> {
        let mut form = vec![
            ("pattern_id".to_string(), "p2p".to_string()),
            ("to".to_string(), transfer.to.clone()),
            ("amount".to_string(), format!("{:.2}", transfer.amount)),
        ];
        push_opt(&mut form, "comment", &transfer.comment);
        push_opt(&mut form, "message", &transfer.message);
        push_opt(&mut form, "label", &transfer.label);
        self.request_payment_form(form).await
    }

    /// Generic `request-payment`: a shop payment by `pattern_id` (`scid`)
    /// with the store's template parameters, or any other payment pattern.
    pub async fn request_payment(
        &self,
        pattern_id: &str,
        params: Vec<(String, String)>,
    ) -> Result<RequestPayment> {
        let mut form: Vec<(String, String)> =
            vec![("pattern_id".to_string(), pattern_id.to_string())];
        form.extend(params);
        self.request_payment_form(form).await
    }

    async fn request_payment_form(&self, form: Vec<(String, String)>) -> Result<RequestPayment> {
        let value = self.post("request-payment", form).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Confirm a payment created by [`Client::request_p2p`] / [`Client::request_payment`].
    ///
    /// On `in_progress` repeat the call with the same parameters;
    /// on `ext_auth_required` run the 3-D Secure dance and repeat with just `request_id`.
    pub async fn process_payment(&self, query: &ProcessPaymentQuery) -> Result<ProcessPayment> {
        let mut form = vec![("request_id".to_string(), query.request_id.clone())];
        if let Some(source) = &query.money_source {
            form.push(("money_source".to_string(), source.as_str().to_string()));
        }
        push_opt(&mut form, "csc", &query.csc);
        push_opt(
            &mut form,
            "ext_auth_success_uri",
            &query.ext_auth_success_uri,
        );
        push_opt(&mut form, "ext_auth_fail_uri", &query.ext_auth_fail_uri);
        let value = self.post("process-payment", form).await?;
        Ok(serde_json::from_value(value)?)
    }
}
