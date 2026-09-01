//! Data models for the YooMoney wallet API responses.

use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

/// Account information. Response of `POST /api/account-info`.
#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    /// Account number, e.g. `4100123456789`.
    pub account: String,
    /// Account balance.
    pub balance: f64,
    /// Currency code, e.g. `643` (RUB).
    pub currency: String,
    /// Account identification status (`anonymous`, `identified`, `semi-identified`).
    #[serde(default)]
    pub account_status: Option<String>,
    /// Account type (`personal`, `professional`).
    #[serde(default)]
    pub account_type: Option<String>,
    /// Detailed balance breakdown.
    #[serde(default)]
    pub balance_details: Option<BalanceDetails>,
    /// Cards linked to the wallet.
    #[serde(default)]
    pub cards_linked: Vec<Card>,
}

/// Detailed balance breakdown (all fields optional, present depending on account).
#[derive(Debug, Clone, Deserialize)]
pub struct BalanceDetails {
    #[serde(default)]
    pub total: Option<f64>,
    #[serde(default)]
    pub available: Option<f64>,
    #[serde(default)]
    pub deposition_pending: Option<f64>,
    #[serde(default)]
    pub blocked: Option<f64>,
    #[serde(default)]
    pub debt: Option<f64>,
    #[serde(default)]
    pub hold: Option<f64>,
}

/// A bank card linked to the wallet.
#[derive(Debug, Clone, Deserialize)]
pub struct Card {
    /// Last digits of the card number.
    #[serde(default)]
    pub pan_fragment: Option<String>,
    /// Card type, e.g. `VISA`.
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
}

/// Operation history page. Response of `POST /api/operation-history`.
#[derive(Debug, Clone, Deserialize)]
pub struct History {
    /// Record to pass as [`HistoryQuery::start_record`] to fetch the next page.
    #[serde(default)]
    pub next_record: Option<String>,
    /// Operations, in reverse chronological order.
    #[serde(default)]
    pub operations: Vec<Operation>,
}

/// A single operation (payment or deposit).
#[derive(Debug, Clone, Deserialize)]
pub struct Operation {
    #[serde(default)]
    pub operation_id: Option<String>,
    /// `success`, `refused` or `in_progress`.
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub datetime: Option<DateTime<FixedOffset>>,
    /// Brief description (merchant name or top-up source).
    #[serde(default)]
    pub title: Option<String>,
    /// Payment template id, present for payments.
    #[serde(default)]
    pub pattern_id: Option<String>,
    /// `in` or `out`.
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub amount: Option<f64>,
    /// Payment label if it was set.
    #[serde(default)]
    pub label: Option<String>,
    /// Operation type, e.g. `payment`, `deposition`, `incoming-transfer`.
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
}

/// Detailed information about a single operation. Response of `POST /api/operation-details`.
#[derive(Debug, Clone, Deserialize)]
pub struct OperationDetails {
    #[serde(default)]
    pub operation_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub pattern_id: Option<String>,
    /// `in` or `out`.
    #[serde(default)]
    pub direction: Option<String>,
    /// Debit amount from the account.
    #[serde(default)]
    pub amount: Option<f64>,
    /// Amount to be received, present for outgoing transfers.
    #[serde(default)]
    pub amount_due: Option<f64>,
    /// Commission, present for outgoing transfers.
    #[serde(default)]
    pub fee: Option<f64>,
    #[serde(default)]
    pub datetime: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub title: Option<String>,
    /// Sender account, present for incoming transfers.
    #[serde(default)]
    pub sender: Option<String>,
    /// Recipient identifier, present for outgoing transfers.
    #[serde(default)]
    pub recipient: Option<String>,
    /// `account`, `phone` or `email`.
    #[serde(default)]
    pub recipient_type: Option<String>,
    /// Message to the recipient.
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub codepro: Option<bool>,
    /// Protection code, present for protected transfers.
    #[serde(default)]
    pub protection_code: Option<String>,
    #[serde(default)]
    pub expires: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub answer_datetime: Option<DateTime<FixedOffset>>,
    /// Detailed payment description.
    #[serde(default)]
    pub details: Option<String>,
    /// Digital goods data, present for digital goods stores.
    #[serde(default)]
    pub digital_goods: Option<DigitalGood>,
    /// Payment label if it was set.
    #[serde(default)]
    pub label: Option<String>,
    /// Operation type.
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
}

/// Digital goods data attached to an operation.
#[derive(Debug, Clone, Deserialize)]
pub struct DigitalGood {
    #[serde(rename = "article", default)]
    pub products: Vec<DigitalProduct>,
    #[serde(rename = "bonus", default)]
    pub bonuses: Vec<DigitalBonus>,
}

/// A purchased digital product.
#[derive(Debug, Clone, Deserialize)]
pub struct DigitalProduct {
    #[serde(rename = "merchantArticleId", default)]
    pub merchant_article_id: Option<String>,
    #[serde(default)]
    pub serial: Option<String>,
    #[serde(default)]
    pub secret: Option<String>,
}

/// A purchased digital bonus.
#[derive(Debug, Clone, Deserialize)]
pub struct DigitalBonus {
    #[serde(default)]
    pub serial: Option<String>,
    #[serde(default)]
    pub secret: Option<String>,
}
