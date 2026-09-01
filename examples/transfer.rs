//! P2P transfer: request-payment + process-payment.
//!
//! Requires a token with the `payment-p2p` scope:
//! `$env:TOKEN = "..."` then `cargo run --example transfer -- 4100123456789 10.00`

use yoomoney::{Client, MoneySource, P2pTransfer, ProcessPaymentQuery};

#[tokio::main]
async fn main() -> yoomoney::Result<()> {
    let mut args = std::env::args().skip(1);
    let to = args.next().expect("usage: transfer <recipient> <amount>");
    let amount: f64 = args
        .next()
        .expect("usage: transfer <recipient> <amount>")
        .parse()
        .unwrap();

    let token = std::env::var("TOKEN").expect("set the TOKEN env var");
    let client = Client::new(&token);

    let request = client
        .request_p2p(&P2pTransfer {
            to,
            amount,
            message: Some("Sent via yoomoney-rs example".into()),
            ..Default::default()
        })
        .await?;

    if !request.is_success() {
        return Err(yoomoney::Error::Api(yoomoney::ApiError {
            code: request
                .error
                .as_deref()
                .unwrap_or("unknown")
                .parse()
                .unwrap(),
        }));
    }

    let payment = client
        .process_payment(&ProcessPaymentQuery {
            request_id: request.request_id.clone().unwrap(),
            money_source: Some(MoneySource::Wallet),
            ..Default::default()
        })
        .await?;

    if payment.is_success() {
        println!("transfer complete, payment_id: {:?}", payment.payment_id);
    } else if payment.is_ext_auth_required() {
        println!("3-D Secure required, confirm at {:?}", payment.acs_uri);
    } else if let Some(e) = &payment.error {
        println!("refused: {e}");
    }
    Ok(())
}
