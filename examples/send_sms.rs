use sendly::{Error, Sendly, SendMessageRequest};

#[tokio::main]
async fn main() {
    // Get API key from environment or use test key
    let api_key = std::env::var("SENDLY_API_KEY").unwrap_or_else(|_| "sk_test_v1_example".to_string());

    // Create client
    let client = Sendly::new(api_key);

    // Send an SMS
    match client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Hello from Sendly Rust SDK!".to_string(),
        })
        .await
    {
        Ok(message) => {
            println!("Message sent successfully!");
            println!("  ID: {}", message.id);
            println!("  To: {}", message.to);
            println!("  Status: {}", message.status);
            println!("  Credits used: {}", message.credits_used);
        }
        Err(e) => {
            handle_error(e);
        }
    }
}

fn handle_error(error: Error) {
    match error {
        Error::Authentication { message } => {
            eprintln!("Authentication failed: {}", message);
        }
        Error::InsufficientCredits { message } => {
            eprintln!("Insufficient credits: {}", message);
        }
        Error::RateLimit { message, retry_after } => {
            eprintln!("Rate limited: {}", message);
            if let Some(seconds) = retry_after {
                eprintln!("Retry after: {} seconds", seconds);
            }
        }
        Error::Validation { message } => {
            eprintln!("Validation error: {}", message);
        }
        Error::NotFound { message } => {
            eprintln!("Not found: {}", message);
        }
        Error::Network { message } => {
            eprintln!("Network error: {}", message);
        }
        _ => {
            eprintln!("Error: {}", error);
        }
    }
}
