mod common;

use common::{
    create_test_client, mock_get_success, mock_list_success, mock_send_success, setup_mock_server,
};
use common::{
    mock_auth_error, mock_insufficient_credits, mock_not_found, mock_rate_limit, mock_server_error,
};
use futures::StreamExt;
use sendly::{Error, ListMessagesOptions, MessageStatus, SendMessageRequest, SenderType};
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

// ==================== send() Tests ====================

#[tokio::test]
async fn test_send_success() {
    let mock_server = setup_mock_server().await;
    mock_send_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Hello World".to_string(),
        ))
        .await;

    assert!(result.is_ok());
    let message = result.unwrap();
    assert_eq!(message.id, "msg_abc123");
    assert_eq!(message.to, "+15551234567");
    assert_eq!(message.text, "Hello World");
    assert_eq!(message.status, MessageStatus::Queued);
    assert_eq!(message.segments, 1);
    assert_eq!(message.credits_used, 1);
}

#[tokio::test]
async fn test_send_invalid_phone_format() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest::new(
            "invalid-phone".to_string(),
            "Hello".to_string(),
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
async fn test_send_empty_text() {
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

#[tokio::test]
async fn test_send_text_too_long() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let long_text = "a".repeat(1601);

    let result = client
        .messages()
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            long_text,
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message, .. } => {
            assert!(message.contains("exceeds maximum length"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_send_counts_characters_not_bytes() {
    let mock_server = setup_mock_server().await;
    mock_send_success().mount(&mock_server).await;
    let client = create_test_client(&mock_server.uri());

    for text in ["é".repeat(1600), "日".repeat(1600), "😀".repeat(1600)] {
        client
            .messages()
            .send(SendMessageRequest::new("+15551234567".to_string(), text))
            .await
            .expect("1600 characters written outside ASCII should be sent");
    }

    let result = client
        .messages()
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "é".repeat(1601),
        ))
        .await;
    assert!(matches!(result, Err(Error::Validation { .. })));
    assert_eq!(mock_server.received_requests().await.unwrap().len(), 3);
}

#[tokio::test]
async fn test_send_authentication_error() {
    let mock_server = setup_mock_server().await;
    mock_auth_error().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Hello".to_string(),
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Authentication { message, .. } => {
            assert!(message.contains("Invalid API key"));
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[tokio::test]
async fn test_send_insufficient_credits() {
    let mock_server = setup_mock_server().await;
    mock_insufficient_credits().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Hello".to_string(),
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::InsufficientCredits { message, .. } => {
            assert!(message.contains("Insufficient credits"));
        }
        _ => panic!("Expected InsufficientCredits error"),
    }
}

#[tokio::test]
async fn test_send_rate_limit() {
    let mock_server = setup_mock_server().await;
    mock_rate_limit(60).mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Hello".to_string(),
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::RateLimit {
            message,
            retry_after,
            ..
        } => {
            assert!(message.contains("Rate limit exceeded"));
            assert_eq!(retry_after, Some(60));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[tokio::test]
async fn test_send_server_error() {
    let mock_server = setup_mock_server().await;
    mock_server_error().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Hello".to_string(),
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api { status_code, .. } => {
            assert_eq!(status_code, 500);
        }
        _ => panic!("Expected Api error"),
    }
}

#[tokio::test]
async fn test_send_network_error() {
    // Use invalid URL to trigger network error
    let config = sendly::SendlyConfig::new()
        .base_url("http://invalid-domain-that-does-not-exist-12345.com")
        .timeout(std::time::Duration::from_secs(1))
        .max_retries(0);

    let client = sendly::Sendly::with_config("test_key", config);

    let result = client
        .messages()
        .send(SendMessageRequest::new(
            "+15551234567".to_string(),
            "Hello".to_string(),
        ))
        .await;

    assert!(result.is_err());
    // Should be either Network or Http error
    assert!(matches!(
        result.unwrap_err(),
        Error::Network { .. } | Error::Http(_)
    ));
}

// ==================== send_to() Tests ====================

#[tokio::test]
async fn test_send_to_success() {
    let mock_server = setup_mock_server().await;
    mock_send_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().send_to("+15551234567", "Hello").await;

    assert!(result.is_ok());
    let message = result.unwrap();
    assert_eq!(message.id, "msg_abc123");
}

#[tokio::test]
async fn test_send_to_invalid_phone() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client.messages().send_to("invalid", "Hello").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Validation { .. }));
}

// ==================== list() Tests ====================

#[tokio::test]
async fn test_list_success() {
    let mock_server = setup_mock_server().await;
    mock_list_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list(None).await;

    assert!(result.is_ok());
    let list = result.unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.total(), 2);
    assert_eq!(list.data[0].id, "msg_1");
    assert_eq!(list.data[1].id, "msg_2");
}

#[tokio::test]
async fn test_list_with_options() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages"))
        .and(query_param("limit", "50"))
        .and(query_param("offset", "10"))
        .and(query_param("status", "delivered"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "count": 0
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let options = ListMessagesOptions::new()
        .limit(50)
        .offset(10)
        .status(MessageStatus::Delivered);

    let result = client.messages().list(Some(options)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_with_to_filter() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages"))
        .and(query_param("to", "+15551234567"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "count": 0
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let options = ListMessagesOptions::new().to("+15551234567");

    let result = client.messages().list(Some(options)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_authentication_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list(None).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Authentication { .. }));
}

#[tokio::test]
async fn test_list_not_found() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Resource not found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list(None).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NotFound { .. }));
}

#[tokio::test]
async fn test_list_rate_limit() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({"error": "Rate limit exceeded"}))
                .insert_header("Retry-After", "30"),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list(None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::RateLimit { retry_after, .. } => {
            assert_eq!(retry_after, Some(30));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[tokio::test]
async fn test_list_server_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list(None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api { status_code, .. } => {
            assert_eq!(status_code, 500);
        }
        _ => panic!("Expected Api error"),
    }
}

// ==================== get() Tests ====================

#[tokio::test]
async fn test_get_success() {
    let mock_server = setup_mock_server().await;
    mock_get_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get("msg_abc123").await;

    assert!(result.is_ok());
    let message = result.unwrap();
    assert_eq!(message.id, "msg_abc123");
    assert_eq!(message.status, MessageStatus::Delivered);
    assert!(message.delivered_at.is_some());
}

#[tokio::test]
async fn test_get_empty_id() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get("").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message, .. } => {
            assert!(message.contains("Message ID is required"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_get_not_found() {
    let mock_server = setup_mock_server().await;
    mock_not_found().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get("msg_nonexistent").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::NotFound { message, .. } => {
            assert!(message.contains("not found"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_get_authentication_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/msg_test"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get("msg_test").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Authentication { .. }));
}

#[tokio::test]
async fn test_get_rate_limit() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/msg_test"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({"error": "Rate limit exceeded"}))
                .insert_header("Retry-After", "45"),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get("msg_test").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::RateLimit { retry_after, .. } => {
            assert_eq!(retry_after, Some(45));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[tokio::test]
async fn test_get_server_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/msg_test"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get("msg_test").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api { status_code, .. } => {
            assert_eq!(status_code, 500);
        }
        _ => panic!("Expected Api error"),
    }
}

// ==================== iter() Tests ====================

#[tokio::test]
async fn test_iter_success() {
    let mock_server = setup_mock_server().await;

    // First page
    Mock::given(method("GET"))
        .and(path("/messages"))
        .and(query_param("limit", "100"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "msg_1",
                    "to": "+15551111111",
                    "text": "Message 1",
                    "status": "delivered",
                    "segments": 1,
                    "creditsUsed": 1,
                    "isSandbox": false
                },
                {
                    "id": "msg_2",
                    "to": "+15552222222",
                    "text": "Message 2",
                    "status": "delivered",
                    "segments": 1,
                    "creditsUsed": 1,
                    "isSandbox": false
                }
            ],
            "count": 2
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let messages_api = client.messages();
    let stream = messages_api.iter(None);
    futures::pin_mut!(stream);
    let mut messages = Vec::new();

    while let Some(result) = stream.next().await {
        messages.push(result.unwrap());
    }

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].id, "msg_1");
    assert_eq!(messages[1].id, "msg_2");
}

#[tokio::test]
async fn test_iter_pagination() {
    let mock_server = setup_mock_server().await;

    // First page
    Mock::given(method("GET"))
        .and(path("/messages"))
        .and(query_param("limit", "2"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "msg_1", "to": "+15551111111", "text": "1", "status": "delivered", "segments": 1, "creditsUsed": 1, "isSandbox": false},
                {"id": "msg_2", "to": "+15552222222", "text": "2", "status": "delivered", "segments": 1, "creditsUsed": 1, "isSandbox": false}
            ],
            "count": 3
        })))
        .mount(&mock_server)
        .await;

    // Second page
    Mock::given(method("GET"))
        .and(path("/messages"))
        .and(query_param("limit", "2"))
        .and(query_param("offset", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "msg_3", "to": "+15553333333", "text": "3", "status": "delivered", "segments": 1, "creditsUsed": 1, "isSandbox": false}
            ],
            "count": 3
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let options = ListMessagesOptions::new().limit(2);
    let messages_api = client.messages();
    let stream = messages_api.iter(Some(options));
    futures::pin_mut!(stream);
    let mut messages = Vec::new();

    while let Some(result) = stream.next().await {
        messages.push(result.unwrap());
    }

    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].id, "msg_1");
    assert_eq!(messages[1].id, "msg_2");
    assert_eq!(messages[2].id, "msg_3");
}

#[tokio::test]
async fn test_iter_with_filter() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages"))
        .and(query_param("status", "delivered"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "msg_1", "to": "+15551111111", "text": "1", "status": "delivered", "segments": 1, "creditsUsed": 1, "isSandbox": false}
            ],
            "count": 1
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let options = ListMessagesOptions::new().status(MessageStatus::Delivered);

    let messages_api = client.messages();
    let stream = messages_api.iter(Some(options));
    futures::pin_mut!(stream);
    let mut messages = Vec::new();

    while let Some(result) = stream.next().await {
        messages.push(result.unwrap());
    }

    assert_eq!(messages.len(), 1);
}

#[tokio::test]
async fn test_iter_error_handling() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let messages_api = client.messages();
    let stream = messages_api.iter(None);
    futures::pin_mut!(stream);

    if let Some(result) = stream.next().await {
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Authentication { .. }));
    } else {
        panic!("Expected error from stream");
    }
}

#[tokio::test]
async fn test_list_keeps_inbound_and_unknown_statuses() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                { "id": "msg_in", "to": "+15555550100", "text": "hi", "status": "received", "segments": 1, "creditsUsed": 0, "isSandbox": false },
                { "id": "msg_new", "to": "+15555550100", "text": "hi", "status": "scheduled", "segments": 1, "creditsUsed": 0, "isSandbox": false }
            ],
            "count": 2
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let list = client
        .messages()
        .list(None)
        .await
        .expect("inbound rows must deserialise");

    assert_eq!(list.data[0].status, MessageStatus::Received);
    assert_eq!(list.data[1].status, MessageStatus::Unknown);
}

fn live_send_response(sender_type: &str) -> serde_json::Value {
    json!({
        "id": "msg_live",
        "to": "+15555550100",
        "from": "+18005550199",
        "text": "Your order shipped",
        "status": "queued",
        "direction": "outbound",
        "error": null,
        "segments": 1,
        "creditsUsed": 2,
        "senderType": sender_type,
        "createdAt": "2026-09-25T10:00:00.000Z",
        "metadata": {}
    })
}

async fn send_with_sender_type(sender_type: &str) -> sendly::Result<sendly::Message> {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(201).set_body_json(live_send_response(sender_type)))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());
    client
        .messages()
        .send(SendMessageRequest::new(
            "+15555550100",
            "Your order shipped",
        ))
        .await
}

#[tokio::test]
async fn test_send_decodes_every_sender_type_the_api_sends() {
    for (wire, expected) in [
        ("number_pool", SenderType::NumberPool),
        ("explicit", SenderType::Explicit),
        ("alphanumeric", SenderType::Alphanumeric),
        ("sandbox", SenderType::Sandbox),
        ("brand_new", SenderType::Unknown),
    ] {
        let message = send_with_sender_type(wire)
            .await
            .unwrap_or_else(|e| panic!("senderType {wire} failed: {e}"));
        assert_eq!(message.sender_type, Some(expected));
        assert_eq!(message.credits_used, 2);
        assert!(!message.simulated);
    }
}

#[tokio::test]
async fn test_send_reports_a_simulated_send() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "msg_sim",
            "to": "+15555550100",
            "from": "+18005550199",
            "text": "hi",
            "status": "delivered",
            "direction": "outbound",
            "error": null,
            "segments": 1,
            "creditsUsed": 0,
            "createdAt": "2026-09-25T10:00:00.000Z",
            "metadata": {},
            "simulated": true,
            "simulatedReason": "This account isn't set up to send to this destination yet, so the message was simulated, not delivered to a handset.",
            "actionUrl": "/verify"
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let message = client
        .messages()
        .send(SendMessageRequest::new("+15555550100", "hi"))
        .await
        .expect("simulated send should decode");

    assert!(message.simulated);
    assert_eq!(message.action_url.as_deref(), Some("/verify"));
    assert!(message
        .simulated_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("simulated")));
    assert_eq!(message.sender_type, None);
}

#[tokio::test]
async fn test_list_total_is_the_pagination_total() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "msg_1",
                    "to": "+15555550100",
                    "from": "+18005550199",
                    "text": "hi",
                    "status": "delivered",
                    "direction": "outbound",
                    "error": null,
                    "errorCode": null,
                    "retryCount": 0,
                    "segments": 1,
                    "creditsUsed": 2,
                    "isSandbox": false,
                    "createdAt": "2026-09-25T10:00:00.000Z",
                    "deliveredAt": "2026-09-25T10:00:05.000Z",
                    "message_format": "sms",
                    "messageFormat": "sms"
                }
            ],
            "pagination": { "total": 57, "limit": 1, "offset": 0, "page": 1, "totalPages": 57, "hasMore": true },
            "count": 1
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let list = client
        .messages()
        .list(None)
        .await
        .expect("list should decode");

    assert_eq!(list.len(), 1);
    assert_eq!(list.count, 1);
    assert_eq!(list.total(), 57);
    let pagination = list.pagination.as_ref().unwrap();
    assert_eq!(pagination.total_pages, 57);
    assert!(pagination.has_more);
}

#[tokio::test]
async fn test_list_total_falls_back_to_the_page_size_without_pagination() {
    let mock_server = setup_mock_server().await;
    mock_list_success().mount(&mock_server).await;
    let client = create_test_client(&mock_server.uri());

    let list = client
        .messages()
        .list(None)
        .await
        .expect("list should decode");

    assert_eq!(list.total(), 2);
}

#[tokio::test]
async fn test_send_group_decodes_the_recipients_of_a_live_send() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages/group"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "msg_grp123",
            "status": "sent",
            "to": [
                { "phoneNumber": "+14155551234", "status": "queued" },
                { "phoneNumber": "+14155555678", "status": "queued" }
            ],
            "group_message_id": "grp_abc123"
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let sent = client
        .messages()
        .send_group(
            sendly::SendGroupMessageRequest::new(vec![
                "+14155551234".to_string(),
                "+14155555678".to_string(),
            ])
            .with_text("Team sync at noon"),
        )
        .await
        .expect("a live group send should decode");

    assert_eq!(sent.to, vec!["+14155551234", "+14155555678"]);
    assert_eq!(sent.recipients.len(), 2);
    assert_eq!(sent.recipients[1].phone_number, "+14155555678");
    assert_eq!(sent.recipients[1].status.as_deref(), Some("queued"));
    assert_eq!(sent.group_message_id.as_deref(), Some("grp_abc123"));
}

#[tokio::test]
async fn test_send_group_decodes_the_numbers_of_a_simulated_send() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages/group"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "msg_grp123",
            "status": "delivered",
            "to": ["+14155551234", "+14155555678"],
            "simulated": true,
            "message": "Group message simulated (test key or verification pending)."
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let sent = client
        .messages()
        .send_group(
            sendly::SendGroupMessageRequest::new(vec![
                "+14155551234".to_string(),
                "+14155555678".to_string(),
            ])
            .with_text("Team sync at noon"),
        )
        .await
        .expect("a simulated group send should decode");

    assert_eq!(sent.to, vec!["+14155551234", "+14155555678"]);
    assert!(sent.recipients.is_empty());
    assert_eq!(sent.simulated, Some(true));
}
