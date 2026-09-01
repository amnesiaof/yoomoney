use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use std::str::FromStr;
use yoomoney::{ApiErrorCode, HistoryQuery, Quickpay, Scope};

#[test]
fn quickpay_url_contains_all_params() {
    let qp = Quickpay::new("4100123456789", "shop", 4568.25)
        .targets("Оплата заказа")
        .payment_type("AC")
        .label("order-123")
        .success_url("https://example.com/success")
        .need_fio(true)
        .need_email(false);

    let url = qp.base_url().unwrap();
    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();

    assert_eq!(
        url.as_str().split('?').next().unwrap(),
        "https://yoomoney.ru/quickpay/confirm"
    );
    assert!(pairs.contains(&("receiver".into(), "4100123456789".into())));
    assert!(pairs.contains(&("quickpay-form".into(), "shop".into())));
    assert!(pairs.contains(&("paymentType".into(), "AC".into())));
    assert!(pairs.contains(&("sum".into(), "4568.25".into())));
    assert!(pairs.contains(&("label".into(), "order-123".into())));
    assert!(pairs.contains(&("successURL".into(), "https://example.com/success".into())));
    assert!(pairs.contains(&("need-fio".into(), "true".into())));
    assert!(pairs.contains(&("need-email".into(), "false".into())));
    assert!(pairs.contains(&("targets".into(), "Оплата заказа".into())));
    // Not set — not present.
    assert!(
        !pairs
            .iter()
            .any(|(k, _)| k == "need-phone" || k == "comment")
    );
    // Space in targets must be encoded.
    let qp2 = Quickpay::new("41001", "button", 1.0).targets("a b");
    assert!(qp2.base_url().unwrap().as_str().contains("targets=a+b"));
}

#[test]
fn quickpay_required_only() {
    let qp = Quickpay::new("41001", "button", 100.0);
    let url = qp.base_url().unwrap();
    assert_eq!(
        url.as_str(),
        "https://yoomoney.ru/quickpay/confirm?receiver=41001&quickpay-form=button&sum=100"
    );
}

#[test]
fn scope_strings_match_api() {
    assert_eq!(Scope::AccountInfo.as_str(), "account-info");
    assert_eq!(Scope::OperationHistory.as_str(), "operation-history");
    assert_eq!(Scope::OperationDetails.as_str(), "operation-details");
    assert_eq!(Scope::IncomingTransfers.as_str(), "incoming-transfers");
    assert_eq!(Scope::PaymentP2p.as_str(), "payment-p2p");
    assert_eq!(Scope::PaymentShop.as_str(), "payment-shop");
}

#[test]
fn authorize_url_has_params() {
    let auth = yoomoney::Authorize::new("client-id", "https://example.com/cb", None);
    let url = auth
        .authorize_url(&[Scope::AccountInfo, Scope::OperationHistory])
        .unwrap();
    let s = url.as_str();
    assert!(s.starts_with("https://yoomoney.ru/oauth/authorize?"));
    assert!(s.contains("client_id=client-id"));
    assert!(s.contains("response_type=code"));
    assert!(s.contains("redirect_uri=https%3A%2F%2Fexample.com%2Fcb"));
    assert!(s.contains("scope=account-info+operation-history"));
}

#[test]
fn error_code_mapping() {
    assert_eq!(
        ApiErrorCode::from_str("illegal_param_from").unwrap(),
        ApiErrorCode::IllegalParamFrom
    );
    assert_eq!(
        ApiErrorCode::from_str("invalid_grant").unwrap(),
        ApiErrorCode::InvalidGrant
    );
    assert_eq!(
        ApiErrorCode::from_str("insufficient_scope").unwrap(),
        ApiErrorCode::InsufficientScope
    );
    assert_eq!(
        ApiErrorCode::from_str("something_new").unwrap(),
        ApiErrorCode::Other("something_new".to_string())
    );
    assert_eq!(
        ApiErrorCode::IllegalParamTill.to_string(),
        "illegal_param_till"
    );
}

#[test]
fn error_code_roundtrip() {
    for code in [
        "invalid_request",
        "unauthorized_client",
        "invalid_grant",
        "illegal_param_type",
        "illegal_param_start_record",
        "illegal_param_records",
        "illegal_param_label",
        "illegal_param_from",
        "illegal_param_till",
        "illegal_param_operation_id",
        "insufficient_scope",
        "invalid_scope",
        "access_denied",
    ] {
        assert_eq!(ApiErrorCode::from_str(code).unwrap().as_str(), code);
    }
}

#[test]
fn datetime_request_format_is_iso8601_with_offset() {
    let dt: DateTime<Utc> = Utc.with_ymd_and_hms(2026, 1, 1, 10, 30, 45).unwrap();
    let s = dt.to_rfc3339_opts(SecondsFormat::Millis, false);
    assert_eq!(s, "2026-01-01T10:30:45.000+00:00");
}

#[test]
fn deserialize_account_sample() {
    let data = serde_json::json!({
        "account": "4100123456789",
        "balance": 1000.00,
        "currency": "643",
        "account_status": "identified",
        "account_type": "personal",
        "balance_details": {"total": 1100.0, "available": 1000.0, "blocked": 100.0},
        "cards_linked": [{"pan_fragment": "1234", "type": "VISA"}]
    });
    let account: yoomoney::Account = serde_json::from_value(data).unwrap();
    assert_eq!(account.account, "4100123456789");
    assert_eq!(account.balance, 1000.0);
    assert_eq!(account.account_status.as_deref(), Some("identified"));
    assert_eq!(account.balance_details.unwrap().blocked, Some(100.0));
    assert_eq!(
        account.cards_linked[0].pan_fragment.as_deref(),
        Some("1234")
    );
}

#[test]
fn deserialize_history_sample_from_docs() {
    let data = serde_json::json!({
        "next_record": "100",
        "operations": [
            {"operation_id": "12345", "status": "success", "datetime": "2023-10-27T10:00:00Z",
             "title": "Payment", "direction": "in", "amount": 50.0, "type": "incoming-transfer"}
        ]
    });
    let history: yoomoney::History = serde_json::from_value(data).unwrap();
    assert_eq!(history.next_record.as_deref(), Some("100"));
    let op = &history.operations[0];
    assert_eq!(op.operation_id.as_deref(), Some("12345"));
    assert_eq!(op.type_.as_deref(), Some("incoming-transfer"));
    let dt = op.datetime.unwrap();
    assert_eq!(
        dt.to_rfc3339_opts(SecondsFormat::Millis, false),
        "2023-10-27T10:00:00.000+00:00"
    );
}

#[test]
fn deserialize_details_sample_with_offset() {
    let data = serde_json::json!({
        "operation_id": "1234567",
        "status": "success",
        "pattern_id": "p2p",
        "direction": "out",
        "amount": 50.25,
        "amount_due": 50.00,
        "fee": 0.25,
        "datetime": "2011-07-11T20:43:00.000+04:00",
        "title": "Transfer to account 4100123456789",
        "recipient": "4100123456789",
        "recipient_type": "account",
        "message": "Buy some bagels",
        "comment": "Transfer",
        "codepro": false,
        "details": "Recipient account:\n4100123456789",
        "type": "payment-shop",
        "digital_goods": {
            "article": [{"merchantArticleId": "abc", "serial": "1", "secret": "s"}],
            "bonus": [{"serial": "b1", "secret": "bs"}]
        }
    });
    let details: yoomoney::OperationDetails = serde_json::from_value(data).unwrap();
    assert_eq!(details.amount_due, Some(50.00));
    assert_eq!(details.codepro, Some(false));
    let dt = details.datetime.unwrap();
    assert_eq!(
        dt.format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string(),
        "2011-07-11T20:43:00.000+04:00"
    );
    let goods = details.digital_goods.unwrap();
    assert_eq!(
        goods.products[0].merchant_article_id.as_deref(),
        Some("abc")
    );
    assert_eq!(goods.bonuses[0].serial.as_deref(), Some("b1"));
}

#[test]
fn history_query_defaults_empty() {
    let q = HistoryQuery::default();
    assert!(q.type_.is_none() && q.from.is_none() && q.till.is_none());
}

#[tokio::test]
async fn authorize_redirect_error_is_typed() {
    let auth = yoomoney::Authorize::new("client-id", "https://example.com/cb", None);
    for (error, expected) in [
        ("access_denied", yoomoney::ApiErrorCode::AccessDenied),
        ("invalid_scope", yoomoney::ApiErrorCode::InvalidScope),
    ] {
        let err = auth
            .handle_redirect(&format!("https://example.com/cb?error={error}"))
            .await
            .unwrap_err();
        match err {
            yoomoney::Error::Api(e) => assert_eq!(e.code, expected),
            other => panic!("expected Api error, got {other:?}"),
        }
    }
}

#[test]
fn authorize_url_instance_name() {
    let auth = yoomoney::Authorize::new("client-id", "https://example.com/cb", None)
        .instance_name("user-42");
    let s = auth.authorize_url(&[Scope::AccountInfo]).unwrap();
    assert!(s.contains("instance_name=user-42"));
}

#[test]
fn money_source_params() {
    use yoomoney::MoneySource;
    assert_eq!(MoneySource::Wallet.as_str(), "wallet");
    assert_eq!(
        MoneySource::Card("card-385244400".into()).as_str(),
        "card-385244400"
    );
}

#[test]
fn deserialize_request_payment_sample() {
    let data = serde_json::json!({
        "status": "success",
        "wallet": {"allowed": true},
        "cards": {
            "allowed": true,
            "csc_required": true,
            "items": [
                {"id": "card-385244400", "pan_fragment": "5280****7918", "type": "MasterCard"},
                {"id": "card-385244401", "pan_fragment": "4008****7919", "type": "Visa"}
            ]
        },
        "request_id": "1234567",
        "contract": "Оплата услуг, сумма 300.00 руб.",
        "balance": 1000.00
    });
    let rp: yoomoney::RequestPayment = serde_json::from_value(data).unwrap();
    assert!(rp.is_success());
    assert_eq!(rp.request_id.as_deref(), Some("1234567"));
    let sources = rp.money_sources().unwrap();
    assert!(sources.wallet.as_ref().unwrap().allowed);
    let cards = sources.cards.as_ref().unwrap();
    assert!(cards.csc_required.unwrap());
    assert_eq!(cards.items[0].id, "card-385244400");
    assert_eq!(cards.items[1].type_.as_deref(), Some("Visa"));

    // The same data nested in a `money_source` object must also parse.
    let nested: yoomoney::RequestPayment = serde_json::from_value(serde_json::json!({
        "status": "success",
        "money_source": {"wallet": {"allowed": true}},
        "request_id": "1"
    }))
    .unwrap();
    assert!(
        nested
            .money_sources()
            .unwrap()
            .wallet
            .as_ref()
            .unwrap()
            .allowed
    );
}

#[test]
fn deserialize_process_payment_samples() {
    let ok: yoomoney::ProcessPayment = serde_json::from_value(serde_json::json!({
        "status": "success",
        "payment_id": "2ABCDE123456789",
        "balance": 1000.00,
        "payer": "4100111",
        "payee": "4100222",
        "credit_amount": 990.00
    }))
    .unwrap();
    assert!(ok.is_success());
    assert!(!ok.is_in_progress() && !ok.is_ext_auth_required());
    assert_eq!(ok.payment_id.as_deref(), Some("2ABCDE123456789"));
    assert_eq!(ok.credit_amount, Some(990.00));

    let in_progress: yoomoney::ProcessPayment =
        serde_json::from_value(serde_json::json!({"status": "in_progress", "next_retry": 5000}))
            .unwrap();
    assert!(in_progress.is_in_progress());
    assert_eq!(in_progress.next_retry, Some(5000));

    let refused: yoomoney::ProcessPayment = serde_json::from_value(serde_json::json!({
        "status": "refused",
        "error": "not_enough_funds"
    }))
    .unwrap();
    assert!(!refused.is_success());
    assert_eq!(refused.error.as_deref(), Some("not_enough_funds"));

    let acs: yoomoney::ProcessPayment = serde_json::from_value(serde_json::json!({
        "status": "ext_auth_required",
        "acs_uri": "https://acs.alfabank.ru/acs/PAReq",
        "acs_params": {"MD": "723613-7431F11492F4F2D0", "PaReq": "eJxVUl1T"}
    }))
    .unwrap();
    assert!(acs.is_ext_auth_required());
    assert_eq!(
        acs.acs_uri.as_deref(),
        Some("https://acs.alfabank.ru/acs/PAReq")
    );
    assert_eq!(
        acs.acs_params.unwrap().get("MD"),
        Some(&"723613-7431F11492F4F2D0".to_string())
    );
}

#[test]
fn notification_parse_and_verify() {
    // Values exactly as in the docs example (including the template
    // placeholders around Cyrillic values) — their HMAC is the doc's `sign`.
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
                &building=12&suite=10&flat=10&zip=125075\
                &sha1_hash=8693ddf402fe5dcc4c4744d466cabada2628148c\
                &sign=092318d12a1249b8ff5cb7b93e1b409a35bfe01eadee9525a147dc977b4eb056";
    let n = yoomoney::IncomingTransfer::parse(body).unwrap();
    assert_eq!(n.notification_type, "p2p-incoming");
    assert_eq!(n.operation_id, "904035776918098009");
    assert_eq!(n.amount, 0.99);
    assert_eq!(n.withdraw_amount, Some(1.00));
    assert_eq!(n.currency, "643");
    assert_eq!(
        n.datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "2014-04-28T16:31:28Z"
    );
    assert_eq!(n.label, "YM.label.12345");
    assert!(n.lastname.contains("Иванов"));
    assert_eq!(n.phone, "+79253332211");
    // Signature from the YooMoney docs (secret: secret123).
    assert!(n.verify("secret123"));
    assert!(!n.verify("wrong-secret"));
}
