//! HTTP notifications about incoming transfers (`p2p-incoming`).
//!
//! YooMoney POSTs form-urlencoded bodies to the URL configured in
//! <https://yoomoney.ru/transfer/myservices/http-notification>.
//! Parse the body with [`IncomingTransfer::parse`] and verify its authenticity
//! with [`IncomingTransfer::verify`] before trusting it.

use crate::error::{Error, Result};
use chrono::{DateTime, FixedOffset};
use hmac::{Hmac, Mac};
use percent_encoding::{NON_ALPHANUMERIC, percent_encode};
use sha2::Sha256;
use url::form_urlencoded;

type HmacSha256 = Hmac<Sha256>;

/// An incoming transfer notification.
#[derive(Debug, Clone)]
pub struct IncomingTransfer {
    /// `p2p-incoming` (wallet transfer) or `card-incoming` (arbitrary card).
    pub notification_type: String,
    /// Operation id, pass to [`crate::Client::operation_details`].
    pub operation_id: String,
    /// Amount received.
    pub amount: f64,
    /// Amount debited from the sender.
    pub withdraw_amount: Option<f64>,
    /// Currency code, always `643` (RUB).
    pub currency: String,
    /// Transfer timestamp.
    pub datetime: DateTime<FixedOffset>,
    /// Sender account (empty for card transfers).
    pub sender: String,
    /// Protection code flag; always `false` nowadays.
    pub codepro: bool,
    /// Payment label if set.
    pub label: String,
    /// Test notification flag.
    pub test_notification: bool,
    /// Held flag; always `false` nowadays.
    pub unaccepted: bool,
    /// Sender contacts — only sent over HTTPS, empty if not requested.
    pub lastname: String,
    pub firstname: String,
    pub fathersname: String,
    pub email: String,
    pub phone: String,
    /// Delivery address fields — only sent over HTTPS, empty if not requested.
    pub city: String,
    pub street: String,
    pub building: String,
    pub suite: String,
    pub flat: String,
    pub zip: String,
    /// Hex signature.
    pub sign: String,
    params: Vec<(String, String)>,
}

impl IncomingTransfer {
    /// Parse a notification body (`application/x-www-form-urlencoded`).
    pub fn parse(body: &str) -> Result<Self> {
        let params: Vec<(String, String)> = form_urlencoded::parse(body.as_bytes())
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        let get = |name: &str| -> Result<&str> {
            params
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(Error::InvalidNotification)
        };
        let get_default = |name: &str| -> String {
            params
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        Ok(Self {
            notification_type: get("notification_type")?.to_string(),
            operation_id: get("operation_id")?.to_string(),
            amount: get("amount")?
                .parse()
                .map_err(|_| Error::InvalidNotification)?,
            withdraw_amount: get("withdraw_amount").ok().and_then(|v| v.parse().ok()),
            currency: get("currency")?.to_string(),
            datetime: DateTime::parse_from_rfc3339(get("datetime")?)
                .map_err(|_| Error::InvalidNotification)?,
            sender: get_default("sender"),
            codepro: get("codepro")? == "true",
            label: get_default("label"),
            test_notification: get_default("test_notification") == "true",
            unaccepted: get_default("unaccepted") == "true",
            lastname: get_default("lastname"),
            firstname: get_default("firstname"),
            fathersname: get_default("fathersname"),
            email: get_default("email"),
            phone: get_default("phone"),
            city: get_default("city"),
            street: get_default("street"),
            building: get_default("building"),
            suite: get_default("suite"),
            flat: get_default("flat"),
            zip: get_default("zip"),
            sign: get("sign")?.to_string(),
            params,
        })
    }

    /// Check the `sign` parameter against the shared secret from the
    /// notification settings (HMAC-SHA256, hex).
    pub fn verify(&self, secret: &str) -> bool {
        let Ok(expected) = unhex(&self.sign) else {
            return false;
        };
        let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
            return false;
        };
        mac.update(signed_string(&self.params).as_bytes());
        mac.verify_slice(&expected).is_ok()
    }
}

/// Build the string the notification signature is computed from:
/// params except `sign`, sorted by key, values RFC 3986 percent-encoded,
/// joined as `key=value` with `&`.
fn signed_string(params: &[(String, String)]) -> String {
    const SET: &percent_encoding::AsciiSet = &NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'.')
        .remove(b'_')
        .remove(b'~');
    let mut sorted: Vec<_> = params.iter().filter(|(k, _)| k != "sign").collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    sorted
        .iter()
        .map(|(k, v)| format!("{}={}", k, percent_encode(v.as_bytes(), SET)))
        .collect::<Vec<_>>()
        .join("&")
}

fn unhex(s: &str) -> Result<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return Err(Error::InvalidNotification);
    }
    (0..s.len() / 2)
        .map(|i| {
            u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|_| Error::InvalidNotification)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact signature string from the docs example.
    const DOCS_SIGNED_STRING: &str = "amount=0.99&building=12&city=%7B%25%20translate%20%25%7D\
%D0%9C%D0%BE%D1%81%D0%BA%D0%B2%D0%B0%7B%25%20%2Ftranslate%20%25%7D&codepro=false&currency=643\
&datetime=2014-04-28T16%3A31%3A28Z&email=address%40example.ru\
&fathersname=%7B%25%20translate%20%25%7D%D0%98%D0%B2%D0%B0%D0%BD%D0%BE%D0%B2%D0%B8%D1%87%7B%25%20%2Ftranslate%20%25%7D\
&firstname=%7B%25%20translate%20%25%7D%D0%98%D0%B2%D0%B0%D0%BD%7B%25%20%2Ftranslate%20%25%7D&flat=10\
&label=YM.label.12345&lastname=%7B%25%20translate%20%25%7D%D0%98%D0%B2%D0%B0%D0%BD%D0%BE%D0%B2%7B%25%20%2Ftranslate%20%25%7D\
&notification_type=p2p-incoming&operation_id=904035776918098009&phone=%2B79253332211\
&sender=41003188981230&sha1_hash=8693ddf402fe5dcc4c4744d466cabada2628148c\
&street=%7B%25%20translate%20%25%7D%D0%A2%D0%B2%D0%B5%D1%80%D1%81%D0%BA%D0%B0%D1%8F%7B%25%20%2Ftranslate%20%25%7D\
&suite=10&test_notification=false&unaccepted=false&withdraw_amount=1.00&zip=125075";

    #[test]
    fn signed_string_encoding_and_order() {
        // Values from the docs example (decoded), including an unknown
        // `sha1_hash` param that must still be part of the string.
        let params: Vec<(String, String)> = [
            ("notification_type", "p2p-incoming"),
            ("operation_id", "904035776918098009"),
            ("amount", "0.99"),
            ("withdraw_amount", "1.00"),
            ("currency", "643"),
            ("datetime", "2014-04-28T16:31:28Z"),
            ("sender", "41003188981230"),
            ("codepro", "false"),
            ("label", "YM.label.12345"),
            ("test_notification", "false"),
            ("unaccepted", "false"),
            ("lastname", "Иванов"),
            ("firstname", "Иван"),
            ("fathersname", "Иванович"),
            ("email", "address@example.ru"),
            ("phone", "+79253332211"),
            ("city", "Москва"),
            ("street", "Тверская"),
            ("building", "12"),
            ("suite", "10"),
            ("flat", "10"),
            ("zip", "125075"),
            ("sha1_hash", "8693ddf402fe5dcc4c4744d466cabada2628148c"),
            ("sign", "ignored"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        let mut sorted: Vec<_> = params.iter().filter(|(k, _)| k != "sign").collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        let built = signed_string(&params);
        // Compare token by token: the docs example has template placeholders
        // around Cyrillic values, so verify structure and encoding separately.
        assert_eq!(
            built.split('&').count(),
            DOCS_SIGNED_STRING.split('&').count()
        );
        let built_keys: Vec<_> = built
            .split('&')
            .map(|p| p.split('=').next().unwrap())
            .collect();
        let docs_keys: Vec<_> = DOCS_SIGNED_STRING
            .split('&')
            .map(|p| p.split('=').next().unwrap())
            .collect();
        assert_eq!(built_keys, docs_keys);
        assert_eq!(
            sorted.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
            docs_keys,
            "params must come out sorted by key"
        );
        // RFC 3986 encoding: `+` in phone, `@` in email, `:` in datetime.
        for token in built.split('&') {
            if let Some(v) = token.strip_prefix("phone=") {
                assert_eq!(v, "%2B79253332211");
            }
            if let Some(v) = token.strip_prefix("email=") {
                assert_eq!(v, "address%40example.ru");
            }
            if let Some(v) = token.strip_prefix("datetime=") {
                assert_eq!(v, "2014-04-28T16%3A31%3A28Z");
            }
        }
    }

    #[test]
    fn docs_signature_hmac_matches() {
        // Docs: HMAC-SHA256 of DOCS_SIGNED_STRING with key `secret123`
        // equals the example `sign` value.
        let mut mac = HmacSha256::new_from_slice(b"secret123").unwrap();
        mac.update(DOCS_SIGNED_STRING.as_bytes());
        let hex: String = mac
            .finalize()
            .into_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            hex,
            "092318d12a1249b8ff5cb7b93e1b409a35bfe01eadee9525a147dc977b4eb056"
        );
    }

    #[test]
    fn parse_missing_required_field() {
        let err = IncomingTransfer::parse("notification_type=p2p-incoming&sign=ab").unwrap_err();
        assert!(matches!(err, Error::InvalidNotification));
    }

    #[test]
    fn docs_body_roundtrip() {
        let body = "notification_type=p2p-incoming&operation_id=904035776918098009&amount=0.99\
&withdraw_amount=1.00&currency=643&datetime=2014-04-28T16%3A31%3A28Z\
&sender=41003188981230&codepro=false&label=YM.label.12345\
&test_notification=false&unaccepted=false\
&lastname=%7B%25%20translate%20%25%7D%D0%98%D0%B2%D0%B0%D0%BD%D0%BE%D0%B2%7B%25%20%2Ftranslate%20%25%7D\
&firstname=%7B%25%20translate%20%25%7D%D0%98%D0%B2%D0%B0%D0%BD%7B%25%20%2Ftranslate%20%25%7D\
&fathersname=%7B%25%20translate%20%25%7D%D0%98%D0%B2%D0%B0%D0%BD%D0%BE%D0%B2%D0%B8%D1%87%7B%25%20%2Ftranslate%20%25%7D\
&email=address%40example.ru&phone=%2B79253332211\
&city=%7B%25%20translate%20%25%7D%D0%9C%D0%BE%D1%81%D0%BA%D0%B2%D0%B0%7B%25%20%2Ftranslate%20%25%7D\
&street=%7B%25%20translate%20%25%7D%D0%A2%D0%B2%D0%B5%D1%80%D1%81%D0%BA%D0%B0%D1%8F%7B%25%20%2Ftranslate%20%25%7D\
&building=12&suite=10&flat=10&zip=125075&sha1_hash=8693ddf402fe5dcc4c4744d466cabada2628148c\
&sign=092318d12a1249b8ff5cb7b93e1b409a35bfe01eadee9525a147dc977b4eb056";
        let n = IncomingTransfer::parse(body).unwrap();
        let built = signed_string(&n.params);
        assert_eq!(
            built, DOCS_SIGNED_STRING,
            "\nbuilt:\n{built}\ndocs:\n{DOCS_SIGNED_STRING}"
        );
    }
}
