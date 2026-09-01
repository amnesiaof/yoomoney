//! Full OAuth authorization flow.
//!
//! 1. Register an app at https://yoomoney.ru/settings/app-obtain-token
//! 2. Run with CLIENT_ID and CLIENT_REDIRECT env vars: `cargo run --example authorize`
//! 3. Open the printed URL, confirm, copy the redirected URL back into the terminal.

use std::io::{BufRead, Write};
use yoomoney::{Authorize, Scope};

#[tokio::main]
async fn main() -> yoomoney::Result<()> {
    let client_id = std::env::var("CLIENT_ID").expect("CLIENT_ID env var");
    let redirect_uri = std::env::var("CLIENT_REDIRECT").unwrap_or_else(|_| "localhost".to_string());

    let auth = Authorize::new(client_id, redirect_uri, None);
    let url = auth.authorize_url(&[
        Scope::AccountInfo,
        Scope::OperationHistory,
        Scope::OperationDetails,
        Scope::IncomingTransfers,
        Scope::PaymentP2p,
        Scope::PaymentShop,
    ])?;

    println!("Open this URL and confirm access:\n{url}");

    print!("\nPaste the URL you were redirected to: ");
    std::io::stdout().flush().unwrap();
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).unwrap();

    let token = auth.handle_redirect(line.trim()).await?;
    println!("\nAccess token: {token}");
    Ok(())
}
