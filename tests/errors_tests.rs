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
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Test".to_string(),
        ))
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        Error::Authentication { message, .. } => {
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
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Test".to_string(),
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Authentication { message, .. } => {
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
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Test".to_string(),
        ))
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        Error::RateLimit {
            message,
            retry_after,
            ..
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
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Test".to_string(),
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::RateLimit {
            message,
            retry_after,
            ..
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
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Test".to_string(),
        ))
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        Error::InsufficientCredits { message, .. } => {
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
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Test".to_string(),
        ))
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        Error::Validation { message, .. } => {
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
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Test".to_string(),
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message, .. } => {
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
        .send(SendMessageRequest::new(
            "invalid-phone".to_string(),
            "Test".to_string(),
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message, .. } => {
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
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "".to_string(),
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message, .. } => {
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
        Error::NotFound { message, .. } => {
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
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Test".to_string(),
        ))
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    // Should be either Network, Http, or Timeout error (DNS can timeout)
    match &error {
        Error::Network { .. } => {
            assert!(error.is_retryable());
            assert_eq!(error.retry_after(), None);
            assert!(error.to_string().contains("Network error"));
        }
        Error::Http(_) => {
            // Also acceptable
        }
        Error::Timeout => {
            // DNS resolution can timeout on invalid domains
            assert!(error.is_retryable());
        }
        _ => panic!("Expected Network, Http, or Timeout error, got: {:?}", error),
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
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Test".to_string(),
        ))
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
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Test".to_string(),
        ))
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        Error::Api {
            message,
            status_code,
            code,
            ..
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
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Test".to_string(),
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api {
            message,
            status_code,
            code,
            ..
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
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Test".to_string(),
        ))
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
        retry_after: None,
        code: None,
        body: None
    }
    .is_retryable());
    assert!(Error::Network {
        message: "test".to_string()
    }
    .is_retryable());
    assert!(Error::Timeout.is_retryable());

    // Non-retryable errors
    assert!(!Error::Authentication {
        message: "test".to_string(),
        code: None,
        body: None
    }
    .is_retryable());
    assert!(!Error::InsufficientCredits {
        message: "test".to_string(),
        code: None,
        body: None
    }
    .is_retryable());
    assert!(!Error::Validation {
        message: "test".to_string(),
        code: None,
        body: None
    }
    .is_retryable());
    assert!(!Error::NotFound {
        message: "test".to_string(),
        code: None,
        body: None
    }
    .is_retryable());
    assert!(!Error::Api {
        message: "test".to_string(),
        status_code: 500,
        code: None,
        body: None
    }
    .is_retryable());
}

#[tokio::test]
async fn test_error_retry_after() {
    let rate_limit_with_retry = Error::RateLimit {
        message: "test".to_string(),
        retry_after: Some(60),
        code: None,
        body: None,
    };
    assert_eq!(rate_limit_with_retry.retry_after(), Some(60));

    let rate_limit_without_retry = Error::RateLimit {
        message: "test".to_string(),
        retry_after: None,
        code: None,
        body: None,
    };
    assert_eq!(rate_limit_without_retry.retry_after(), None);

    // Other errors should return None
    assert_eq!(
        Error::Authentication {
            message: "test".to_string(),
            code: None,
            body: None
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
        code: None,
        body: None,
    };
    assert_eq!(
        format!("{}", auth_error),
        "Authentication failed: Invalid key"
    );

    let rate_limit_error = Error::RateLimit {
        message: "Too many requests".to_string(),
        retry_after: Some(30),
        code: None,
        body: None,
    };
    assert_eq!(
        format!("{}", rate_limit_error),
        "Rate limit exceeded: Too many requests"
    );

    let credits_error = Error::InsufficientCredits {
        message: "No credits".to_string(),
        code: None,
        body: None,
    };
    assert_eq!(
        format!("{}", credits_error),
        "Insufficient credits: No credits"
    );

    let validation_error = Error::Validation {
        message: "Invalid input".to_string(),
        code: None,
        body: None,
    };
    assert_eq!(
        format!("{}", validation_error),
        "Validation error: Invalid input"
    );

    let not_found_error = Error::NotFound {
        message: "Not found".to_string(),
        code: None,
        body: None,
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
        body: None,
    };
    assert_eq!(format!("{}", api_error), "API error (500): Server error");
}

async fn check_error(status: u16, body: serde_json::Value) -> Error {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/verify/v_1/check"))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());
    client.verify().check("v_1", "000000").await.unwrap_err()
}

#[tokio::test]
async fn test_verify_check_keeps_the_code_and_remaining_attempts() {
    let error = check_error(
        400,
        json!({ "error": "invalid_code", "message": "Invalid verification code", "remaining_attempts": 2 }),
    )
    .await;

    assert!(matches!(error, Error::Validation { .. }), "{error:?}");
    assert_eq!(error.code(), Some("invalid_code"));
    assert_eq!(error.remaining_attempts(), Some(2));
    assert_eq!(
        error.body().unwrap()["message"],
        "Invalid verification code"
    );
    assert_eq!(
        error.to_string(),
        "Validation error: Invalid verification code"
    );
}

#[tokio::test]
async fn test_rate_limit_errors_keep_their_codes_apart() {
    let max_attempts = check_error(
        429,
        json!({ "error": "max_attempts_exceeded", "message": "Maximum verification attempts exceeded" }),
    )
    .await;
    let rate_limited = check_error(
        429,
        json!({ "error": "rate_limit_exceeded", "message": "Rate limit exceeded. Limit: 600 requests per minute.", "retryAfter": 60 }),
    )
    .await;

    assert_eq!(max_attempts.code(), Some("max_attempts_exceeded"));
    assert_eq!(rate_limited.code(), Some("rate_limit_exceeded"));
    assert!(!max_attempts.is_retryable());
    assert!(rate_limited.is_retryable());
    assert_eq!(rate_limited.retry_after(), Some(60));
    assert_eq!(max_attempts.remaining_attempts(), None);
}

#[tokio::test]
async fn test_failed_key_lockout_is_not_retryable() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "300")
                .set_body_json(json!({
                    "error": "too_many_failed_key_attempts",
                    "message": "Too many failed API key attempts. Try again in 300 seconds.",
                    "retryAfter": 300
                })),
        )
        .mount(&mock_server)
        .await;
    let config = sendly::SendlyConfig::new()
        .base_url(mock_server.uri())
        .max_retries(3);
    let client = sendly::Sendly::with_config("sk_live_v1_wrong", config);

    let started = std::time::Instant::now();
    let error = client
        .messages()
        .send(SendMessageRequest::new("+15551234567", "Test"))
        .await
        .unwrap_err();

    assert!(matches!(error, Error::RateLimit { .. }), "{error:?}");
    assert_eq!(error.code(), Some("too_many_failed_key_attempts"));
    assert_eq!(
        error.to_string(),
        "Rate limit exceeded: Too many failed API key attempts. Try again in 300 seconds."
    );
    assert_eq!(error.retry_after(), Some(300));
    assert!(!error.is_retryable());
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
    assert_eq!(mock_server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn test_concurrent_key_check_refusal_is_retried_with_the_same_idempotency_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "1")
                .set_body_json(json!({
                    "error": "too_many_concurrent_verifications",
                    "message": "Too many API key checks are already running for this account from this address. Try again in 1 second.",
                    "retryAfter": 1
                })),
        )
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "msg_1",
            "to": "+15551234567",
            "from": "+18005550199",
            "text": "Test",
            "status": "queued",
            "direction": "outbound",
            "error": null,
            "segments": 1,
            "creditsUsed": 2,
            "createdAt": "2026-09-25T10:00:00.000Z",
            "metadata": {}
        })))
        .mount(&mock_server)
        .await;
    let config = sendly::SendlyConfig::new()
        .base_url(mock_server.uri())
        .max_retries(2);
    let client = sendly::Sendly::with_config("sk_live_v1_abc", config);

    let message = client
        .messages()
        .send(SendMessageRequest::new("+15551234567", "Test"))
        .await
        .expect("a transient key-check refusal should be retried");

    assert_eq!(message.id, "msg_1");
    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 2);
    let name = wiremock::http::HeaderName::from_string("idempotency-key".to_string()).unwrap();
    let keys: Vec<_> = requests
        .iter()
        .map(|r| r.headers.get(&name).unwrap().last().as_str().to_string())
        .collect();
    assert_eq!(keys[0], keys[1]);
    assert_eq!(requests[0].body, requests[1].body);
    let body: serde_json::Value = serde_json::from_slice(&requests[1].body).unwrap();
    assert_eq!(body, json!({ "to": "+15551234567", "text": "Test" }));
}

#[tokio::test]
async fn test_otp_rate_limit_is_raised_at_once_with_the_body_retry_after() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/verify"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": "rate_limit_exceeded",
            "message": "Too many OTPs sent to this phone number. Max 5 per 10 minutes.",
            "retryAfter": 600
        })))
        .mount(&mock_server)
        .await;
    let config = sendly::SendlyConfig::new()
        .base_url(mock_server.uri())
        .max_retries(3);
    let client = sendly::Sendly::with_config("sk_live_v1_abc", config);

    let started = std::time::Instant::now();
    let error = client
        .verify()
        .send(sendly::SendVerificationRequest::new("+15551234567"))
        .await
        .unwrap_err();

    assert!(started.elapsed() < std::time::Duration::from_secs(5));
    assert_eq!(mock_server.received_requests().await.unwrap().len(), 1);
    assert_eq!(error.retry_after(), Some(600));
}

#[tokio::test]
async fn test_ordinary_rate_limit_is_raised_with_its_retry_after() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "42")
                .set_body_json(json!({
                    "error": "rate_limit_exceeded",
                    "message": "Rate limit exceeded. Limit: 600 requests per minute.",
                    "retryAfter": 42
                })),
        )
        .mount(&mock_server)
        .await;
    let config = sendly::SendlyConfig::new()
        .base_url(mock_server.uri())
        .max_retries(3);
    let client = sendly::Sendly::with_config("sk_live_v1_abc", config);

    let error = client
        .messages()
        .send(SendMessageRequest::new("+15551234567", "Test"))
        .await
        .unwrap_err();

    assert_eq!(mock_server.received_requests().await.unwrap().len(), 1);
    assert_eq!(error.retry_after(), Some(42));
    assert!(error.is_retryable());
}

#[tokio::test]
async fn test_call_from_a_number_outside_the_us_and_canada_carries_its_code() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/calls"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "from_number_not_supported",
            "message": "Calls can only be placed from numbers in the US or Canada right now."
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let error = client
        .calls()
        .create(
            sendly::CreateCallRequest::new("+15555550123", "3c4d5e6f-7081-4293-a4b5-c6d7e8f90a1b")
                .from_number("+447700900123"),
        )
        .await
        .unwrap_err();

    assert!(matches!(error, Error::Validation { .. }), "{error:?}");
    assert_eq!(error.code(), Some("from_number_not_supported"));
}

#[tokio::test]
async fn test_concurrent_key_check_refusal_is_raised_once_retries_run_out() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/account"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "1")
                .set_body_json(json!({
                    "error": "too_many_concurrent_verifications",
                    "message": "Too many API key checks are already running for this account from this address. Try again in 1 second.",
                    "retryAfter": 1
                })),
        )
        .mount(&mock_server)
        .await;
    let config = sendly::SendlyConfig::new()
        .base_url(mock_server.uri())
        .max_retries(1);
    let client = sendly::Sendly::with_config("sk_live_v1_abc", config);

    let error = client.account().get().await.unwrap_err();

    assert_eq!(error.code(), Some("too_many_concurrent_verifications"));
    assert!(error.is_retryable());
    assert_eq!(mock_server.received_requests().await.unwrap().len(), 2);
}

async fn refused_with_429(body: serde_json::Value) -> (wiremock::MockServer, sendly::Sendly) {
    let mock_server = setup_mock_server().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(ResponseTemplate::new(429).set_body_json(body))
        .mount(&mock_server)
        .await;
    let config = sendly::SendlyConfig::new()
        .base_url(mock_server.uri())
        .max_retries(3);
    let client = sendly::Sendly::with_config("sk_live_v1_abc", config);
    (mock_server, client)
}

#[tokio::test]
async fn test_monthly_quota_is_not_retryable() {
    let (mock_server, client) = refused_with_429(json!({
        "error": "quota_exceeded",
        "message": "Monthly message quota reached",
        "quota": 1000,
        "used": 1000
    }))
    .await;

    let error = client
        .messages()
        .schedule(sendly::ScheduleMessageRequest {
            to: "+15551234567".to_string(),
            text: "Reminder".to_string(),
            scheduled_at: "2026-12-01T10:00:00Z".to_string(),
            from: None,
            message_type: None,
            metadata: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(error, Error::RateLimit { .. }), "{error:?}");
    assert_eq!(error.code(), Some("quota_exceeded"));
    assert_eq!(error.retry_after(), None);
    assert!(!error.is_retryable());
    assert_eq!(error.body().unwrap()["used"], 1000);
    assert_eq!(mock_server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn test_daily_call_limit_is_not_retryable() {
    let (mock_server, client) = refused_with_429(json!({
        "error": "daily_call_limit",
        "message": "Today's calling limit has been reached. Try again tomorrow."
    }))
    .await;

    let error = client
        .calls()
        .create(sendly::CreateCallRequest::new(
            "+15555550123",
            "3c4d5e6f-7081-4293-a4b5-c6d7e8f90a1b",
        ))
        .await
        .unwrap_err();

    assert_eq!(error.code(), Some("daily_call_limit"));
    assert_eq!(error.retry_after(), None);
    assert!(!error.is_retryable());
    assert_eq!(mock_server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn test_limits_waiting_cannot_clear_are_not_retryable() {
    for (code, message) in [
        (
            "hold_limit_reached",
            "You have the maximum number of numbers on hold. Release one or buy it first.",
        ),
        (
            "whatsapp_signup_limit_reached",
            "Too many WhatsApp sign-ups have been started for this workspace.",
        ),
        (
            "too_many_failed_key_attempts",
            "Too many failed API key attempts. Try again in 300 seconds.",
        ),
        (
            "max_attempts_exceeded",
            "Maximum verification attempts exceeded",
        ),
    ] {
        let error = check_error(429, json!({ "error": code, "message": message })).await;
        assert_eq!(error.code(), Some(code));
        assert!(!error.is_retryable(), "{code} should not be retryable");
    }
}

#[tokio::test]
async fn test_provision_rate_limit_stays_retryable_with_the_body_retry_after() {
    let (mock_server, client) = refused_with_429(json!({
        "error": "provision_rate_limit",
        "message": "Max 10 provisions per minute.",
        "retryAfter": 60
    }))
    .await;

    let error = client
        .enterprise()
        .provision(sendly::ProvisionWorkspaceRequest::new("Acme"))
        .await
        .unwrap_err();

    assert_eq!(error.code(), Some("provision_rate_limit"));
    assert_eq!(error.retry_after(), Some(60));
    assert!(error.is_retryable());
    assert_eq!(mock_server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn test_concurrent_key_check_refusal_longer_than_a_minute_is_not_waited_out() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/account"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "120")
                .set_body_json(json!({
                    "error": "too_many_concurrent_verifications",
                    "message": "Too many API key checks are already running for this account from this address. Try again in 120 seconds.",
                    "retryAfter": 120
                })),
        )
        .mount(&mock_server)
        .await;
    let config = sendly::SendlyConfig::new()
        .base_url(mock_server.uri())
        .max_retries(3);
    let client = sendly::Sendly::with_config("sk_live_v1_abc", config);

    let started = std::time::Instant::now();
    let error = client.account().get().await.unwrap_err();

    assert!(started.elapsed() < std::time::Duration::from_secs(5));
    assert_eq!(error.retry_after(), Some(120));
    assert_eq!(mock_server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn test_waited_out_refusal_adds_no_backoff_and_no_wait_after_the_last_attempt() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/account"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "1")
                .set_body_json(json!({
                    "error": "too_many_concurrent_verifications",
                    "message": "Too many API key checks are already running for this account from this address. Try again in 1 second.",
                    "retryAfter": 1
                })),
        )
        .mount(&mock_server)
        .await;
    let config = sendly::SendlyConfig::new()
        .base_url(mock_server.uri())
        .max_retries(2);
    let client = sendly::Sendly::with_config("sk_live_v1_abc", config);

    let started = std::time::Instant::now();
    let error = client.account().get().await.unwrap_err();
    let elapsed = started.elapsed();

    assert_eq!(error.code(), Some("too_many_concurrent_verifications"));
    assert_eq!(mock_server.received_requests().await.unwrap().len(), 3);
    assert!(elapsed >= std::time::Duration::from_secs(2), "{elapsed:?}");
    assert!(
        elapsed < std::time::Duration::from_millis(2900),
        "{elapsed:?}"
    );
}

async fn upload_refused_once_then(
    route: &str,
    success: ResponseTemplate,
) -> (wiremock::MockServer, sendly::Sendly) {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(route))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "1")
                .set_body_json(json!({
                    "error": "too_many_concurrent_verifications",
                    "message": "Too many API key checks are already running for this account from this address. Try again in 1 second.",
                    "retryAfter": 1
                })),
        )
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(route))
        .respond_with(success)
        .mount(&mock_server)
        .await;
    let config = sendly::SendlyConfig::new()
        .base_url(mock_server.uri())
        .max_retries(2);
    let client = sendly::Sendly::with_config("sk_live_v1_abc", config);
    (mock_server, client)
}

async fn assert_upload_resent_whole(mock_server: &wiremock::MockServer, content: &[u8]) {
    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 2);
    let name = wiremock::http::HeaderName::from_string("idempotency-key".to_string()).unwrap();
    let keys: Vec<_> = requests
        .iter()
        .map(|r| r.headers.get(&name).unwrap().last().as_str().to_string())
        .collect();
    assert_eq!(keys[0], keys[1]);
    for request in &requests {
        assert!(request.body.windows(content.len()).any(|w| w == content));
    }
}

#[tokio::test]
async fn test_media_upload_retries_a_transient_key_check_refusal() {
    let (mock_server, client) = upload_refused_once_then(
        "/media",
        ResponseTemplate::new(200).set_body_json(json!({
            "id": "med_abc123",
            "url": "https://cdn.example.com/med_abc123.jpg",
            "contentType": "image/jpeg",
            "sizeBytes": 16
        })),
    )
    .await;

    let media = client
        .media()
        .upload_bytes(b"fake-image-bytes".to_vec(), "x.jpg", "image/jpeg")
        .await
        .expect("a transient key-check refusal should be retried");

    assert_eq!(media.id, "med_abc123");
    assert_upload_resent_whole(&mock_server, b"fake-image-bytes").await;
}

#[tokio::test]
async fn test_business_upgrade_start_retries_a_transient_key_check_refusal() {
    use sendly::business_upgrade::{BrnType, EinDocument, EntityType, StartUpgradeRequest};

    let (mock_server, client) = upload_refused_once_then(
        "/workspaces/ws_1/upgrade",
        ResponseTemplate::new(202).set_body_json(json!({
            "success": true,
            "pendingVerificationId": "bv_new",
            "status": "provisioning"
        })),
    )
    .await;

    let started = client
        .business_upgrade()
        .start(
            "ws_1",
            StartUpgradeRequest::new(
                "Acme Holdings LLC",
                "12-3456789",
                BrnType::Ein,
                "US",
                EntityType::PrivateProfit,
            ),
            Some(EinDocument::new(b"ein-letter-bytes".to_vec())),
        )
        .await
        .expect("a transient key-check refusal should be retried");

    assert_eq!(started.pending_verification_id, "bv_new");
    assert_upload_resent_whole(&mock_server, b"ein-letter-bytes").await;
}

#[tokio::test]
async fn test_business_upgrade_resubmit_retries_a_transient_key_check_refusal() {
    use sendly::business_upgrade::{EinDocument, ResubmitUpgradeRequest};

    let (mock_server, client) = upload_refused_once_then(
        "/workspaces/ws_1/upgrade/resubmit",
        ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "pendingVerificationId": "bv_new"
        })),
    )
    .await;

    let resubmitted = client
        .business_upgrade()
        .resubmit(
            "ws_1",
            ResubmitUpgradeRequest::default(),
            Some(EinDocument::new(b"ein-letter-bytes".to_vec())),
        )
        .await
        .expect("a transient key-check refusal should be retried");

    assert_eq!(resubmitted.pending_verification_id, "bv_new");
    assert_upload_resent_whole(&mock_server, b"ein-letter-bytes").await;
}

#[tokio::test]
async fn test_verification_document_upload_retries_a_transient_key_check_refusal() {
    let (mock_server, client) = upload_refused_once_then(
        "/enterprise/verification-document/upload",
        ResponseTemplate::new(200).set_body_json(json!({
            "url": "https://cdn.example.com/doc_1.pdf",
            "id": "doc_1"
        })),
    )
    .await;

    let uploaded = client
        .enterprise()
        .upload_verification_document(
            b"verification-pdf-bytes".to_vec(),
            "letter.pdf",
            Some("ws_1"),
            None,
        )
        .await
        .expect("a transient key-check refusal should be retried");

    assert_eq!(uploaded.id, "doc_1");
    assert_upload_resent_whole(&mock_server, b"verification-pdf-bytes").await;
}

#[tokio::test]
async fn test_upload_is_not_resent_after_a_timeout() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/media"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_millis(600))
                .set_body_json(json!({
                    "id": "med_abc123",
                    "url": "https://cdn.example.com/med_abc123.jpg",
                    "contentType": "image/jpeg",
                    "sizeBytes": 16
                })),
        )
        .mount(&mock_server)
        .await;
    let config = sendly::SendlyConfig::new()
        .base_url(mock_server.uri())
        .timeout(std::time::Duration::from_millis(200))
        .max_retries(2);
    let client = sendly::Sendly::with_config("sk_live_v1_abc", config);

    let error = client
        .media()
        .upload_bytes(b"fake-image-bytes".to_vec(), "x.jpg", "image/jpeg")
        .await
        .unwrap_err();

    assert!(matches!(error, Error::Timeout), "{error:?}");
    assert_eq!(mock_server.received_requests().await.unwrap().len(), 1);
}
