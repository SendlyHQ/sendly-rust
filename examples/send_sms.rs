use sendly::{Error, SendMessageRequest, Sendly};

#[tokio::main]
async fn main() {
    // Get API key from environment or use test key
    let api_key =
        std::env::var("SENDLY_API_KEY").unwrap_or_else(|_| "sk_test_v1_example".to_string());

    // Create client
    let client = Sendly::new(api_key);

    // Send an SMS
    match client
        .messages()
        .send(SendMessageRequest::new(
            "+15125550123",
            "Hello from Sendly Rust SDK!",
        ))
        .await
    {
        Ok(message) => {
            println!("Message sent successfully!");
            println!("  ID: {}", message.id);
            println!("  To: {}", message.to);
            println!("  Status: {}", message.status);
            println!("  Credits used: {}", message.credits_used);
            if message.simulated {
                println!("  Simulated: {:?}", message.simulated_reason);
            }
        }
        Err(e) => {
            handle_error(e);
        }
    }
}

fn handle_error(error: Error) {
    match error {
        Error::Authentication { message, .. } => {
            eprintln!("Authentication failed: {}", message);
        }
        Error::InsufficientCredits { message, .. } => {
            eprintln!("Insufficient credits: {}", message);
        }
        Error::RateLimit { ref code, .. }
            if code.as_deref() == Some("too_many_failed_key_attempts") =>
        {
            eprintln!("Too many requests with a wrong API key; fix SENDLY_API_KEY instead of retrying");
        }
        Error::RateLimit {
            message,
            retry_after,
            ..
        } => {
            eprintln!("Rate limited: {}", message);
            if let Some(seconds) = retry_after {
                eprintln!("Retry after: {} seconds", seconds);
            }
        }
        Error::Validation { message, code, .. } => {
            eprintln!("Validation error ({:?}): {}", code, message);
        }
        Error::NotFound { message, .. } => {
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
