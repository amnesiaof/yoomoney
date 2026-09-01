# yoomoney

Async Rust client for the [YooMoney wallet API](https://yoomoney.ru/docs/wallet).

- OAuth authorization (code exchange, callback-style)
- Account info and balance
- Operation history and details
- P2P transfers and shop payments (`request-payment` / `process-payment`)
- Token revoke
- Incoming-transfer HTTP notification parsing + HMAC-SHA256 verification
- Quickpay payment link builder (no token needed)

## Installation

```toml
[dependencies]
yoomoney = "0.1"
```

## Usage

### Authorization

```rust,no_run
use yoomoney::{Authorize, Scope};

#[tokio::main]
async fn main() -> yoomoney::Result<()> {
    let auth = Authorize::new("your-client-id", "localhost", None);

    // 1. Show this URL to the user, they confirm access.
    let url = auth.authorize_url(&[Scope::AccountInfo, Scope::OperationHistory])?;
    println!("Open and confirm: {url}");

    // 2. Get the URL the user was redirected to and exchange the code.
    let redirected = "http://localhost/?code=XXXXXXXX";
    let token = auth.handle_redirect(redirected).await?;
    println!("token: {token}");
    Ok(())
}
```

### Account info

```rust,no_run
use yoomoney::Client;

#[tokio::main]
async fn main() -> yoomoney::Result<()> {
    let client = Client::new("your-token");
    let account = client.account_info().await?;
    println!("{}: {} {}", account.account, account.balance, account.currency);
    Ok(())
}
```

### Operation history

```rust,no_run
use yoomoney::{Client, HistoryQuery};

#[tokio::main]
async fn main() -> yoomoney::Result<()> {
    let client = Client::new("your-token");
    let history = client
        .operation_history(&HistoryQuery { records: Some(30), ..Default::default() })
        .await?;
    for op in history.operations {
        println!("{:?} {:?} {:?}", op.datetime, op.amount, op.title);
    }
    Ok(())
}
```

### P2P transfer

```rust,no_run
use yoomoney::{Client, MoneySource, P2pTransfer, ProcessPaymentQuery};

#[tokio::main]
async fn main() -> yoomoney::Result<()> {
    let client = Client::new("your-token");

    // 1. Create the transfer (requires the payment-p2p scope).
    let request = client
        .request_p2p(&P2pTransfer {
            to: "4100123456789".into(),
            amount: 10.0,
            message: Some("For coffee".into()),
            ..Default::default()
        })
        .await?;

    if !request.is_success() {
        eprintln!("refused: {:?}", request.error);
        return Ok(());
    }

    // 2. Confirm it from the wallet.
    let payment = client
        .process_payment(&ProcessPaymentQuery {
            request_id: request.request_id.clone().unwrap(),
            money_source: Some(MoneySource::Wallet),
            ..Default::default()
        })
        .await?;

    println!("success: {:?}", payment.payment_id);
    Ok(())
}
```

### Incoming transfer notification

YooMoney POSTs a form-urlencoded notification when money arrives. Verify the
signature before trusting it (secret from the
[notification settings](https://yoomoney.ru/transfer/myservices/http-notification)):

```rust
use yoomoney::IncomingTransfer;

let transfer = IncomingTransfer::parse(
    "notification_type=p2p-incoming&operation_id=123&amount=0.99&currency=643\
     &datetime=2014-04-28T16:31:28Z&codepro=false&label=&sign=ABCD",
)
.unwrap();

if transfer.verify("my-notification-secret") {
    println!("got {} RUB from {}", transfer.amount, transfer.sender);
}
```

### Quickpay link

```rust,no_run
use yoomoney::Quickpay;

let url = Quickpay::new("4100123456789", "button", 100.0)
    .payment_type("AC")
    .label("order-123")
    .base_url()?;
println!("{url}");
```

## Error handling

API error codes (`illegal_param_from`, `invalid_grant`, ...) are typed in
[`ApiErrorCode`]; unknown codes fall back to a `Other(String)` variant.

## License

MIT