//! Quickpay payment link (no token needed).

use yoomoney::Quickpay;

fn main() -> yoomoney::Result<()> {
    let url = Quickpay::new("4100123456789", "shop", 4568.25)
        .payment_type("AC")
        .targets("Оплата заказа 123")
        .label("order-123")
        .formcomment("Donation")
        .success_url("https://example.com/thanks")
        .base_url()?;

    println!("Payment URL: {url}");
    Ok(())
}
