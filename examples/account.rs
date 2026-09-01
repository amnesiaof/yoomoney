//! Account info and balance.

use yoomoney::Client;

#[tokio::main]
async fn main() -> yoomoney::Result<()> {
    let token = std::env::var("TOKEN").expect("TOKEN env var");
    let client = Client::new(token);

    let account = client.account_info().await?;
    println!("Account: {}", account.account);
    println!("Balance: {} {}", account.balance, account.currency);
    println!("Status:  {:?}", account.account_status);
    println!("Type:    {:?}", account.account_type);
    if let Some(bd) = account.balance_details {
        println!("Total: {:?}, available: {:?}", bd.total, bd.available);
    }
    for card in account.cards_linked {
        println!("Card: {:?} {:?}", card.pan_fragment, card.type_);
    }
    Ok(())
}
