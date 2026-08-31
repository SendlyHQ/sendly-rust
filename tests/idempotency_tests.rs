mod common;

use common::{
    create_test_client, mock_batch_send_success, mock_cancel_scheduled_success, mock_list_success,
    mock_schedule_success, mock_send_success, setup_mock_server, TEST_API_KEY,
};
use regex::Regex;
use sendly::{
    BatchMessageItem, Error, IdempotentRequestOptions, ScheduleMessageRequest, SendBatchRequest,
    SendGroupMessageRequest, SendMessageRequest, SendRcsMessageRequest,
    SendWhatsAppMessageRequest,
};
use serde_json::json;
use wiremock::http::HeaderName;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const AUTO_KEY_PATTERN: &str =
    r"^sendly-rust-retry-[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

/// Creates a client that retries once, with a timeout short enough to trip
/// on a delayed mock response.
fn create_retrying_test_client(base_url: &str) -> sendly::Sendly {
    let config = sendly::SendlyConfig::new()
        .base_url(base_url)
        .timeout(std::time::Duration::from_millis(300))
        .max_retries(1);

    sendly::Sendly::with_config(TEST_API_KEY, config)
}

/// Returns the Idempotency-Key header of the nth request the server received.
async fn key_of_request(mock_server: &MockServer, index: usize) -> Option<String> {
    let name = HeaderName::from_string("idempotency-key".to_string()).unwrap();
    let requests = mock_server
        .received_requests()
        .await
        .expect("request recording is enabled");
    requests
        .get(index)
        .expect("request at index")
        .headers
        .get(&name)
        .map(|values| values.last().as_str().to_string())
}

fn send_request() -> SendMessageRequest {
    SendMessageRequest::new("+15551234567", "Hello World")
}

fn batch_request() -> SendBatchRequest {
    SendBatchRequest {
        messages: vec![BatchMessageItem {
            to: "+15551234567".to_string(),
            text: "Hello Alice!".to_string(),
            metadata: None,
        }],
        from: None,
        message_type: None,
        metadata: None,
    }
}

/// Mock a delayed message send that outlasts the retrying client's timeout.
fn mock_send_delayed() -> Mock {
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(2)),
        )
        .up_to_n_times(1)
}

// ==================== Automatic Key Generation Tests ====================

#[tokio::test]
async fn test_auto_key_attached_to_post() {
    let mock_server = setup_mock_server().await;
    mock_send_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());
    let result = client.messages().send(send_request()).await;

    assert!(result.is_ok());
    let key = key_of_request(&mock_server, 0).await.expect("key present");
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
    assert!(key.len() <= 255);
}

#[tokio::test]
async fn test_no_key_on_get() {
    let mock_server = setup_mock_server().await;
    mock_list_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());
    let result = client.messages().list(None).await;

    assert!(result.is_ok());
    assert_eq!(key_of_request(&mock_server, 0).await, None);
}

#[tokio::test]
async fn test_no_key_on_delete() {
    let mock_server = setup_mock_server().await;
    mock_cancel_scheduled_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());
    let result = client.messages().cancel_scheduled("sched_abc123").await;

    assert!(result.is_ok());
    assert_eq!(key_of_request(&mock_server, 0).await, None);
}

#[tokio::test]
async fn test_no_auto_key_on_batch_send() {
    let mock_server = setup_mock_server().await;
    mock_batch_send_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());
    let result = client.messages().send_batch(batch_request()).await;

    assert!(result.is_ok());
    assert_eq!(key_of_request(&mock_server, 0).await, None);
}

#[tokio::test]
async fn test_auto_key_on_media_upload() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/media"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "med_abc123",
            "url": "https://cdn.example.com/med_abc123.jpg",
            "contentType": "image/jpeg",
            "sizeBytes": 16
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .media()
        .upload_bytes(b"fake-image-bytes".to_vec(), "x.jpg", "image/jpeg")
        .await;

    assert!(result.is_ok());
    let key = key_of_request(&mock_server, 0).await.expect("key present");
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_distinct_keys_per_logical_request() {
    let mock_server = setup_mock_server().await;
    mock_send_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());
    client.messages().send(send_request()).await.unwrap();
    client.messages().send(send_request()).await.unwrap();

    let first = key_of_request(&mock_server, 0).await.expect("key present");
    let second = key_of_request(&mock_server, 1).await.expect("key present");
    assert_ne!(first, second);
}

// ==================== Retry Behavior Tests ====================

#[tokio::test]
async fn test_auto_key_reused_across_timeout_retry() {
    let mock_server = setup_mock_server().await;
    mock_send_delayed().mount(&mock_server).await;
    mock_send_success().mount(&mock_server).await;

    let client = create_retrying_test_client(&mock_server.uri());
    let result = client.messages().send(send_request()).await;

    assert!(result.is_ok());
    let first = key_of_request(&mock_server, 0).await.expect("key present");
    let second = key_of_request(&mock_server, 1).await.expect("key present");
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&first));
    assert_eq!(first, second);
}

#[tokio::test]
async fn test_caller_key_reused_across_timeout_retry() {
    let mock_server = setup_mock_server().await;
    mock_send_delayed().mount(&mock_server).await;
    mock_send_success().mount(&mock_server).await;

    let client = create_retrying_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_with_options(
            send_request(),
            IdempotentRequestOptions::new().idempotency_key("signup-otp-user-99"),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(
        key_of_request(&mock_server, 0).await.as_deref(),
        Some("signup-otp-user-99")
    );
    assert_eq!(
        key_of_request(&mock_server, 1).await.as_deref(),
        Some("signup-otp-user-99")
    );
}

// ==================== Caller-Supplied Key Tests ====================

#[tokio::test]
async fn test_caller_key_sent_verbatim() {
    let mock_server = setup_mock_server().await;
    mock_send_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_with_options(
            send_request(),
            IdempotentRequestOptions::new().idempotency_key("order-4821-shipped"),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(
        key_of_request(&mock_server, 0).await.as_deref(),
        Some("order-4821-shipped")
    );
}

#[tokio::test]
async fn test_caller_key_trimmed() {
    let mock_server = setup_mock_server().await;
    mock_send_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_with_options(
            send_request(),
            IdempotentRequestOptions::new().idempotency_key("  order-4821-shipped  "),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(
        key_of_request(&mock_server, 0).await.as_deref(),
        Some("order-4821-shipped")
    );
}

#[tokio::test]
async fn test_caller_key_on_batch_send() {
    let mock_server = setup_mock_server().await;
    mock_batch_send_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_batch_with_options(
            batch_request(),
            IdempotentRequestOptions::new().idempotency_key("campaign-77-wave-1"),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(
        key_of_request(&mock_server, 0).await.as_deref(),
        Some("campaign-77-wave-1")
    );
}

#[tokio::test]
async fn test_caller_key_on_schedule() {
    let mock_server = setup_mock_server().await;
    mock_schedule_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .schedule_with_options(
            ScheduleMessageRequest {
                to: "+15551234567".to_string(),
                text: "Reminder!".to_string(),
                scheduled_at: "2025-01-20T10:00:00Z".to_string(),
                from: None,
                message_type: None,
                metadata: None,
            },
            IdempotentRequestOptions::new().idempotency_key("reminder-visit-31"),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(
        key_of_request(&mock_server, 0).await.as_deref(),
        Some("reminder-visit-31")
    );
}

#[tokio::test]
async fn test_caller_key_on_group_send() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages/group"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_grp123",
            "status": "sent",
            "to": ["+14155551234", "+14155555678"],
            "groupMessageId": "grp_abc123"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_group_with_options(
            SendGroupMessageRequest::new(vec![
                "+14155551234".to_string(),
                "+14155555678".to_string(),
            ])
            .with_text("Team sync at noon"),
            IdempotentRequestOptions::new().idempotency_key("standup-ping-0824"),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(
        key_of_request(&mock_server, 0).await.as_deref(),
        Some("standup-ping-0824")
    );
}

#[tokio::test]
async fn test_caller_key_on_whatsapp_send() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_wa_123",
            "channel": "whatsapp",
            "message_format": "whatsapp",
            "to": "+15551234567",
            "from": "+15559876543",
            "text": "Hello!",
            "status": "queued",
            "segments": 1,
            "creditsUsed": 3,
            "whatsapp": { "kind": "text", "messageId": null },
            "createdAt": "2026-08-24T10:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_whatsapp_with_options(
            SendWhatsAppMessageRequest::new("+15551234567", "+15559876543").with_text("Hello!"),
            IdempotentRequestOptions::new().idempotency_key("wa-hello-1"),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(
        key_of_request(&mock_server, 0).await.as_deref(),
        Some("wa-hello-1")
    );
}

#[tokio::test]
async fn test_caller_key_on_rcs_send() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "msg_rcs_123",
            "channel": "rcs",
            "message_format": "rcs",
            "to": "+15551234567",
            "from": "Acme Inc",
            "text": "Hello!",
            "status": "sent",
            "segments": 1,
            "creditsUsed": 2,
            "rcs": { "kind": "text", "agentId": "rcsa_abc123", "agentName": "Acme Inc" },
            "createdAt": "2026-08-24T10:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_rcs_with_options(
            SendRcsMessageRequest::new("+15551234567").with_text("Hello!"),
            IdempotentRequestOptions::new().idempotency_key("rcs-hello-1"),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(
        key_of_request(&mock_server, 0).await.as_deref(),
        Some("rcs-hello-1")
    );
}

#[tokio::test]
async fn test_empty_caller_key_falls_back_to_auto() {
    let mock_server = setup_mock_server().await;
    mock_send_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_with_options(
            send_request(),
            IdempotentRequestOptions::new().idempotency_key(""),
        )
        .await;

    assert!(result.is_ok());
    let key = key_of_request(&mock_server, 0).await.expect("key present");
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_whitespace_caller_key_falls_back_to_auto() {
    let mock_server = setup_mock_server().await;
    mock_send_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_with_options(
            send_request(),
            IdempotentRequestOptions::new().idempotency_key("   "),
        )
        .await;

    assert!(result.is_ok());
    let key = key_of_request(&mock_server, 0).await.expect("key present");
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_non_ascii_caller_key_rejected_without_network_call() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send_with_options(
            send_request(),
            IdempotentRequestOptions::new().idempotency_key("Заказ-42"),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
            assert!(message.contains("1-255 printable ASCII"));
        }
        _ => panic!("Expected Validation error"),
    }
    assert!(mock_server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn test_overlong_caller_key_rejected_without_network_call() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send_with_options(
            send_request(),
            IdempotentRequestOptions::new().idempotency_key("k".repeat(256)),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
            assert!(message.contains("1-255 printable ASCII"));
        }
        _ => panic!("Expected Validation error"),
    }
    assert!(mock_server.received_requests().await.unwrap().is_empty());
}
