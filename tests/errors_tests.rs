mod common;

use common::{create_test_client, setup_mock_server};
use sendly::{Error, SendMessageRequest};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

// ==================== Error::Authentication Tests ====================

#[tokio::test]
async fn test_error_authentication() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        Error::Authentication { message } => {
            assert_eq!(message, "Invalid API key");
            assert!(!error.is_retryable());
            assert_eq!(error.retry_after(), None);
            assert_eq!(error.to_string(), "Authentication failed: Invalid API key");
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[tokio::test]
async fn test_error_authentication_with_message_field() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "message": "Authentication required"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Authentication { message } => {
            assert_eq!(message, "Authentication required");
        }
        _ => panic!("Expected Authentication error"),
    }
}

// ==================== Error::RateLimit Tests ====================

#[tokio::test]
async fn test_error_rate_limit_with_retry_after() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({"error": "Rate limit exceeded"}))
                .insert_header("Retry-After", "60"),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        Error::RateLimit {
            message,
            retry_after,
        } => {
            assert_eq!(message, "Rate limit exceeded");
            assert_eq!(*retry_after, Some(60));
            assert!(error.is_retryable());
            assert_eq!(error.retry_after(), Some(60));
            assert_eq!(
                error.to_string(),
                "Rate limit exceeded: Rate limit exceeded"
            );
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[tokio::test]
async fn test_error_rate_limit_without_retry_after() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(429).set_body_json(json!({"error": "Too many requests"})),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::RateLimit {
            message,
            retry_after,
        } => {
            assert_eq!(message, "Too many requests");
            assert_eq!(retry_after, None);
        }
        _ => panic!("Expected RateLimit error"),
    }
}

// ==================== Error::InsufficientCredits Tests ====================

#[tokio::test]
async fn test_error_insufficient_credits() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(402).set_body_json(json!({
            "error": "Insufficient credits. Please add credits to your account."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        Error::InsufficientCredits { message } => {
            assert!(message.contains("Insufficient credits"));
            assert!(!error.is_retryable());
            assert_eq!(error.retry_after(), None);
            assert!(error.to_string().contains("Insufficient credits"));
        }
        _ => panic!("Expected InsufficientCredits error"),
    }
}

// ==================== Error::Validation Tests ====================

#[tokio::test]
async fn test_error_validation_bad_request() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "Invalid request parameters"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        Error::Validation { message } => {
            assert_eq!(message, "Invalid request parameters");
            assert!(!error.is_retryable());
            assert_eq!(error.retry_after(), None);
            assert_eq!(
                error.to_string(),
                "Validation error: Invalid request parameters"
            );
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_error_validation_unprocessable_entity() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "Invalid phone number format"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
            assert_eq!(message, "Invalid phone number format");
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_error_validation_client_side_phone() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "invalid-phone".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
            assert!(message.contains("Invalid phone number format"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_error_validation_client_side_text() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "".to_string(),
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
            assert!(message.contains("Message text is required"));
        }
        _ => panic!("Expected Validation error"),
    }
}

// ==================== Error::NotFound Tests ====================

#[tokio::test]
async fn test_error_not_found() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/msg_nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Message not found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get("msg_nonexistent").await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        Error::NotFound { message } => {
            assert_eq!(message, "Message not found");
            assert!(!error.is_retryable());
            assert_eq!(error.retry_after(), None);
            assert_eq!(error.to_string(), "Not found: Message not found");
        }
        _ => panic!("Expected NotFound error"),
    }
}

// ==================== Error::Network Tests ====================

#[tokio::test]
async fn test_error_network() {
    // Use invalid domain to trigger network error
    let config = sendly::SendlyConfig::new()
        .base_url("http://invalid-domain-that-does-not-exist-xyz123.com")
        .timeout(std::time::Duration::from_secs(1))
        .max_retries(0);

    let client = sendly::Sendly::with_config("test_key", config);

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    // Should be either Network or Http error
    match &error {
        Error::Network { .. } => {
            assert!(error.is_retryable());
            assert_eq!(error.retry_after(), None);
            assert!(error.to_string().contains("Network error"));
        }
        Error::Http(_) => {
            // Also acceptable
        }
        _ => panic!("Expected Network or Http error, got: {:?}", error),
    }
}

// ==================== Error::Timeout Tests ====================

#[tokio::test]
async fn test_error_timeout() {
    let mock_server = setup_mock_server().await;

    // Mock a slow endpoint that exceeds timeout
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(5)))
        .mount(&mock_server)
        .await;

    let config = sendly::SendlyConfig::new()
        .base_url(&mock_server.uri())
        .timeout(std::time::Duration::from_millis(100))
        .max_retries(0);

    let client = sendly::Sendly::with_config("test_key", config);

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        Error::Timeout => {
            assert!(error.is_retryable());
            assert_eq!(error.retry_after(), None);
            assert_eq!(error.to_string(), "Request timed out");
        }
        _ => panic!("Expected Timeout error, got: {:?}", error),
    }
}

// ==================== Error::Api Tests ====================

#[tokio::test]
async fn test_error_api_500() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        Error::Api {
            message,
            status_code,
            code,
        } => {
            assert_eq!(message, "Internal server error");
            assert_eq!(*status_code, 500);
            assert_eq!(code, &None);
            assert!(!error.is_retryable());
            assert_eq!(error.retry_after(), None);
            assert_eq!(error.to_string(), "API error (500): Internal server error");
        }
        _ => panic!("Expected Api error"),
    }
}

#[tokio::test]
async fn test_error_api_with_code() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(503).set_body_json(json!({
            "error": "Service temporarily unavailable",
            "code": "SERVICE_UNAVAILABLE"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api {
            message,
            status_code,
            code,
        } => {
            assert_eq!(message, "Service temporarily unavailable");
            assert_eq!(status_code, 503);
            assert_eq!(code, Some("SERVICE_UNAVAILABLE".to_string()));
        }
        _ => panic!("Expected Api error"),
    }
}

#[tokio::test]
async fn test_error_api_fallback_message() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(502).set_body_json(json!({})))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api {
            message,
            status_code,
            ..
        } => {
            assert_eq!(message, "Unknown error");
            assert_eq!(status_code, 502);
        }
        _ => panic!("Expected Api error"),
    }
}

// ==================== Error Utility Methods Tests ====================

#[tokio::test]
async fn test_error_is_retryable() {
    // Retryable errors
    assert!(Error::RateLimit {
        message: "test".to_string(),
        retry_after: None
    }
    .is_retryable());
    assert!(Error::Network {
        message: "test".to_string()
    }
    .is_retryable());
    assert!(Error::Timeout.is_retryable());

    // Non-retryable errors
    assert!(!Error::Authentication {
        message: "test".to_string()
    }
    .is_retryable());
    assert!(!Error::InsufficientCredits {
        message: "test".to_string()
    }
    .is_retryable());
    assert!(!Error::Validation {
        message: "test".to_string()
    }
    .is_retryable());
    assert!(!Error::NotFound {
        message: "test".to_string()
    }
    .is_retryable());
    assert!(!Error::Api {
        message: "test".to_string(),
        status_code: 500,
        code: None
    }
    .is_retryable());
}

#[tokio::test]
async fn test_error_retry_after() {
    let rate_limit_with_retry = Error::RateLimit {
        message: "test".to_string(),
        retry_after: Some(60),
    };
    assert_eq!(rate_limit_with_retry.retry_after(), Some(60));

    let rate_limit_without_retry = Error::RateLimit {
        message: "test".to_string(),
        retry_after: None,
    };
    assert_eq!(rate_limit_without_retry.retry_after(), None);

    // Other errors should return None
    assert_eq!(
        Error::Authentication {
            message: "test".to_string()
        }
        .retry_after(),
        None
    );
    assert_eq!(
        Error::Network {
            message: "test".to_string()
        }
        .retry_after(),
        None
    );
    assert_eq!(Error::Timeout.retry_after(), None);
}

// ==================== Error Display Tests ====================

#[tokio::test]
async fn test_error_display_formats() {
    let auth_error = Error::Authentication {
        message: "Invalid key".to_string(),
    };
    assert_eq!(
        format!("{}", auth_error),
        "Authentication failed: Invalid key"
    );

    let rate_limit_error = Error::RateLimit {
        message: "Too many requests".to_string(),
        retry_after: Some(30),
    };
    assert_eq!(
        format!("{}", rate_limit_error),
        "Rate limit exceeded: Too many requests"
    );

    let credits_error = Error::InsufficientCredits {
        message: "No credits".to_string(),
    };
    assert_eq!(
        format!("{}", credits_error),
        "Insufficient credits: No credits"
    );

    let validation_error = Error::Validation {
        message: "Invalid input".to_string(),
    };
    assert_eq!(
        format!("{}", validation_error),
        "Validation error: Invalid input"
    );

    let not_found_error = Error::NotFound {
        message: "Not found".to_string(),
    };
    assert_eq!(format!("{}", not_found_error), "Not found: Not found");

    let network_error = Error::Network {
        message: "Connection failed".to_string(),
    };
    assert_eq!(
        format!("{}", network_error),
        "Network error: Connection failed"
    );

    let timeout_error = Error::Timeout;
    assert_eq!(format!("{}", timeout_error), "Request timed out");

    let api_error = Error::Api {
        message: "Server error".to_string(),
        status_code: 500,
        code: None,
    };
    assert_eq!(format!("{}", api_error), "API error (500): Server error");
}
