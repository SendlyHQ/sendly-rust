use sendly::{ListMessagesOptions, MessageStatus, Sendly};

#[tokio::main]
async fn main() -> sendly::Result<()> {
    let api_key =
        std::env::var("SENDLY_API_KEY").unwrap_or_else(|_| "sk_test_v1_example".to_string());

    let client = Sendly::new(api_key);

    // List recent messages
    println!("=== Recent Messages ===");
    let messages = client
        .messages()
        .list(Some(ListMessagesOptions::new().limit(10)))
        .await?;

    println!("Total: {}", messages.total());
    println!("Count in page: {}", messages.len());
    println!();

    for msg in messages.iter() {
        println!("{}: {} - {}", msg.id, msg.to, msg.status);
    }

    // List with filters
    println!("\n=== Delivered Messages ===");
    let delivered = client
        .messages()
        .list(Some(
            ListMessagesOptions::new()
                .status(MessageStatus::Delivered)
                .limit(5),
        ))
        .await?;

    for msg in delivered.iter() {
        println!("{}: Delivered at {:?}", msg.id, msg.delivered_at);
    }

    // Get a specific message (if we have any)
    if let Some(first) = messages.first() {
        println!("\n=== Message Details ===");
        let msg = client.messages().get(&first.id).await?;
        println!("ID: {}", msg.id);
        println!("To: {}", msg.to);
        println!("Text: {}", msg.text);
        println!("Status: {}", msg.status);
        println!("Created: {:?}", msg.created_at);
    }

    Ok(())
}
