//! Async client for the YooMoney wallet API.
//!
//! Covers OAuth authorization, account info, operation history and details,
//! payments (P2P transfers and shop payments), quickpay payment URL building,
//! and incoming-transfer HTTP notifications.
//!
//! ```
//! use yoomoney::{Client, Quickpay};
//!
//! // Quickpay URL needs no token.
//! let url = Quickpay::new("4100123456789", "button", 100.0)
//!     .payment_type("AC")
//!     .base_url()
//!     .unwrap();
//! assert!(url.as_str().starts_with("https://yoomoney.ru/quickpay/confirm?"));
//!
//! // API calls need a token from the authorization flow.
//! let client = Client::new("your-token");
//! // let account = client.account_info().await?;
//! ```

mod auth;
mod client;
mod error;
mod models;
mod notification;
mod payments;
mod quickpay;

pub use auth::{Authorize, Scope};
pub use client::{Client, HistoryQuery};
pub use error::{ApiError, ApiErrorCode, Error, Result};
pub use models::{
    Account, BalanceDetails, Card, DigitalBonus, DigitalGood, DigitalProduct, History, Operation,
    OperationDetails,
};
pub use notification::IncomingTransfer;
pub use payments::{
    CardsSource, LinkedCard, MoneySource, MoneySources, P2pTransfer, ProcessPayment,
    ProcessPaymentQuery, RequestPayment, WalletSource,
};
pub use quickpay::Quickpay;
