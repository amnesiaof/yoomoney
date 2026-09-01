//! Quickpay payment form URL builder.

use crate::client::push_opt;
use crate::error::Result;
use url::Url;

const QUICKPAY_URL: &str = "https://yoomoney.ru/quickpay/confirm";

/// Builder for a quickpay payment form URL.
///
/// The resulting URL ([`Quickpay::base_url`]) is a payment page link:
/// open it in a browser or embed it as a form action. No HTTP requests are made.
///
/// ```
/// use yoomoney::Quickpay;
///
/// let url = Quickpay::new("4100123456789", "button", 100.0)
///     .payment_type("AC")
///     .label("order-123")
///     .base_url()
///     .unwrap();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Quickpay {
    /// Wallet number to transfer to, e.g. `4100123456789`.
    pub receiver: String,
    /// Form type: `button`, `shop` or `donate`.
    pub quickpay_form: String,
    /// Transfer amount debited from the sender.
    pub sum: f64,
    /// Payment purpose, e.g. order number (for `shop` form).
    pub targets: Option<String>,
    /// Payment method: `PC` (YooMoney wallet), `AC` (bank card), `MC` (mobile).
    pub payment_type: Option<String>,
    /// Form name displayed on the payment page (up to 50 chars).
    pub formcomment: Option<String>,
    /// Short payment purpose (up to 50 chars).
    pub short_dest: Option<String>,
    /// Payment label set by the merchant (up to 64 chars).
    pub label: Option<String>,
    /// Comment for the transfer.
    pub comment: Option<String>,
    /// URL to redirect the payer to after a successful payment.
    pub success_url: Option<String>,
    /// Ask the payer for their full name.
    pub need_fio: Option<bool>,
    /// Ask the payer for their email.
    pub need_email: Option<bool>,
    /// Ask the payer for their phone number.
    pub need_phone: Option<bool>,
    /// Ask the payer for their postal address.
    pub need_address: Option<bool>,
}

impl Quickpay {
    /// Create a quickpay URL builder with the required parameters.
    ///
    /// - `receiver`: wallet number to transfer to.
    /// - `quickpay_form`: form type — `button`, `shop` or `donate`.
    /// - `sum`: transfer amount debited from the sender.
    pub fn new(receiver: impl Into<String>, quickpay_form: impl Into<String>, sum: f64) -> Self {
        Self {
            receiver: receiver.into(),
            quickpay_form: quickpay_form.into(),
            sum,
            targets: None,
            payment_type: None,
            formcomment: None,
            short_dest: None,
            label: None,
            comment: None,
            success_url: None,
            need_fio: None,
            need_email: None,
            need_phone: None,
            need_address: None,
        }
    }

    /// Payment purpose, e.g. order number (for `shop` form).
    pub fn targets(mut self, value: impl Into<String>) -> Self {
        self.targets = Some(value.into());
        self
    }

    /// Payment method: `PC` (YooMoney wallet), `AC` (bank card), `MC` (mobile).
    pub fn payment_type(mut self, value: impl Into<String>) -> Self {
        self.payment_type = Some(value.into());
        self
    }

    /// Form name displayed on the payment page (up to 50 chars).
    pub fn formcomment(mut self, value: impl Into<String>) -> Self {
        self.formcomment = Some(value.into());
        self
    }

    /// Short payment purpose (up to 50 chars).
    pub fn short_dest(mut self, value: impl Into<String>) -> Self {
        self.short_dest = Some(value.into());
        self
    }

    /// Payment label set by the merchant (up to 64 chars).
    pub fn label(mut self, value: impl Into<String>) -> Self {
        self.label = Some(value.into());
        self
    }

    /// Comment for the transfer.
    pub fn comment(mut self, value: impl Into<String>) -> Self {
        self.comment = Some(value.into());
        self
    }

    /// URL to redirect the payer to after a successful payment.
    pub fn success_url(mut self, value: impl Into<String>) -> Self {
        self.success_url = Some(value.into());
        self
    }

    /// Ask the payer for their full name.
    pub fn need_fio(mut self, value: bool) -> Self {
        self.need_fio = Some(value);
        self
    }

    /// Ask the payer for their email.
    pub fn need_email(mut self, value: bool) -> Self {
        self.need_email = Some(value);
        self
    }

    /// Ask the payer for their phone number.
    pub fn need_phone(mut self, value: bool) -> Self {
        self.need_phone = Some(value);
        self
    }

    /// Ask the payer for their postal address.
    pub fn need_address(mut self, value: bool) -> Self {
        self.need_address = Some(value);
        self
    }

    /// Build the payment URL with all set parameters.
    pub fn base_url(&self) -> Result<Url> {
        let mut params: Vec<(String, String)> = vec![
            ("receiver".into(), self.receiver.clone()),
            ("quickpay-form".into(), self.quickpay_form.clone()),
            ("sum".into(), self.sum.to_string()),
        ];
        push_opt(&mut params, "targets", &self.targets);
        push_opt(&mut params, "paymentType", &self.payment_type);
        push_opt(&mut params, "formcomment", &self.formcomment);
        push_opt(&mut params, "short-dest", &self.short_dest);
        push_opt(&mut params, "label", &self.label);
        push_opt(&mut params, "comment", &self.comment);
        push_opt(&mut params, "successURL", &self.success_url);
        push_opt(&mut params, "need-fio", &self.need_fio);
        push_opt(&mut params, "need-email", &self.need_email);
        push_opt(&mut params, "need-phone", &self.need_phone);
        push_opt(&mut params, "need-address", &self.need_address);
        Ok(Url::parse_with_params(QUICKPAY_URL, &params)?)
    }
}
