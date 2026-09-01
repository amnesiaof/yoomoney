//! Operation history (3 pages) and details of the first operation.

use yoomoney::{Client, HistoryQuery};

#[tokio::main]
async fn main() -> yoomoney::Result<()> {
    let token = std::env::var("TOKEN").expect("TOKEN env var");
    let client = Client::new(token);

    let mut next_record = None;
    for _ in 0..3 {
        let query = HistoryQuery {
            start_record: next_record,
            records: Some(30),
            ..Default::default()
        };
        let history = client.operation_history(&query).await?;
        for op in &history.operations {
            println!(
                "{} | {:?} | {:?} | {:?}",
                op.datetime.map(|d| d.to_rfc3339()).unwrap_or_default(),
                op.direction,
                op.amount,
                op.title
            );
        }
        match history.next_record {
            Some(n) => next_record = Some(n),
            None => break,
        }
    }

    Ok(())
}
