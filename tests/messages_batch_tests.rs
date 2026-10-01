mod common;

use common::{create_test_client, mock_batch_send_success, setup_mock_server};
use common::{mock_get_batch_success, mock_list_batches_success};
use sendly::{BatchMessageItem, BatchStatus, Error, ListBatchesOptions, SendBatchRequest};
use serde_json::json;
use wiremock::matchers::{method, path, path_regex, query_param};
use wiremock::{Mock, ResponseTemplate};

// ==================== send_batch() Tests ====================

#[tokio::test]
async fn test_send_batch_success() {
    let mock_server = setup_mock_server().await;
    mock_batch_send_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send_batch(SendBatchRequest {
            messages: vec![
                BatchMessageItem {
                    to: "+15551111111".to_string(),
                    text: "Message 1".to_string(),
                    metadata: None,
                },
                BatchMessageItem {
                    to: "+15552222222".to_string(),
                    text: "Message 2".to_string(),
                    metadata: None,
                },
            ],
            from: None,
            message_type: None,
            metadata: None,
        })
        .await;

    assert!(result.is_ok());
    let batch = result.unwrap();
    assert_eq!(batch.batch_id, "batch_abc123");
    assert_eq!(batch.status, BatchStatus::Processing);
    assert_eq!(batch.total, 2);
    assert_eq!(batch.queued, 2);
}

#[tokio::test]
async fn test_send_batch_empty_messages() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send_batch(SendBatchRequest {
            messages: vec![],
            from: None,
            message_type: None,
            metadata: None,
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message, .. } => {
            assert!(message.contains("Messages array is required"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_send_batch_invalid_phone() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send_batch(SendBatchRequest {
            messages: vec![
                BatchMessageItem {
                    to: "+15551111111".to_string(),
                    text: "Valid".to_string(),
                    metadata: None,
                },
                BatchMessageItem {
                    to: "invalid-phone".to_string(),
                    text: "Invalid".to_string(),
                    metadata: None,
                },
            ],
            from: None,
            message_type: None,
            metadata: None,
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message, .. } => {
            assert!(message.contains("Invalid phone number at index"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_send_batch_invalid_text() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send_batch(SendBatchRequest {
            messages: vec![
                BatchMessageItem {
                    to: "+15551111111".to_string(),
                    text: "Valid".to_string(),
                    metadata: None,
                },
                BatchMessageItem {
                    to: "+15552222222".to_string(),
                    text: "".to_string(),
                    metadata: None,
                },
            ],
            from: None,
            message_type: None,
            metadata: None,
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message, .. } => {
            assert!(message.contains("Invalid message text at index"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_send_batch_text_too_long() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let long_text = "a".repeat(1601);

    let result = client
        .messages()
        .send_batch(SendBatchRequest {
            messages: vec![BatchMessageItem {
                to: "+15551111111".to_string(),
                text: long_text,
                metadata: None,
            }],
            from: None,
            message_type: None,
            metadata: None,
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message, .. } => {
            assert!(message.contains("Invalid message text at index"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_send_batch_authentication_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages/batch"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send_batch(SendBatchRequest {
            messages: vec![BatchMessageItem {
                to: "+15551111111".to_string(),
                text: "Test".to_string(),
                metadata: None,
            }],
            from: None,
            message_type: None,
            metadata: None,
        })
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Authentication { .. }));
}

#[tokio::test]
async fn test_send_batch_insufficient_credits() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages/batch"))
        .respond_with(ResponseTemplate::new(402).set_body_json(json!({
            "error": "Insufficient credits"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send_batch(SendBatchRequest {
            messages: vec![BatchMessageItem {
                to: "+15551111111".to_string(),
                text: "Test".to_string(),
                metadata: None,
            }],
            from: None,
            message_type: None,
            metadata: None,
        })
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Error::InsufficientCredits { .. }
    ));
}

#[tokio::test]
async fn test_send_batch_not_found() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages/batch"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Resource not found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send_batch(SendBatchRequest {
            messages: vec![BatchMessageItem {
                to: "+15551111111".to_string(),
                text: "Test".to_string(),
                metadata: None,
            }],
            from: None,
            message_type: None,
            metadata: None,
        })
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NotFound { .. }));
}

#[tokio::test]
async fn test_send_batch_rate_limit() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages/batch"))
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
        .send_batch(SendBatchRequest {
            messages: vec![BatchMessageItem {
                to: "+15551111111".to_string(),
                text: "Test".to_string(),
                metadata: None,
            }],
            from: None,
            message_type: None,
            metadata: None,
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::RateLimit { retry_after, .. } => {
            assert_eq!(retry_after, Some(60));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[tokio::test]
async fn test_send_batch_server_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages/batch"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send_batch(SendBatchRequest {
            messages: vec![BatchMessageItem {
                to: "+15551111111".to_string(),
                text: "Test".to_string(),
                metadata: None,
            }],
            from: None,
            message_type: None,
            metadata: None,
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api { status_code, .. } => {
            assert_eq!(status_code, 500);
        }
        _ => panic!("Expected Api error"),
    }
}

// ==================== get_batch() Tests ====================

#[tokio::test]
async fn test_get_batch_success() {
    let mock_server = setup_mock_server().await;
    mock_get_batch_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get_batch("batch_abc123").await;

    assert!(result.is_ok());
    let batch = result.unwrap();
    assert_eq!(batch.batch_id, "batch_abc123");
    assert_eq!(batch.status, BatchStatus::Completed);
    assert_eq!(batch.total, 2);
    assert_eq!(batch.sent, 2);
    assert_eq!(batch.messages.len(), 2);
}

#[tokio::test]
async fn test_get_batch_empty_id() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get_batch("").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message, .. } => {
            assert!(message.contains("Batch ID is required"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_get_batch_not_found() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path_regex(r"^/messages/batch/.*$"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Batch not found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get_batch("batch_nonexistent").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::NotFound { message, .. } => {
            assert!(message.contains("not found"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_get_batch_authentication_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/batch/batch_test"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get_batch("batch_test").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Authentication { .. }));
}

#[tokio::test]
async fn test_get_batch_rate_limit() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/batch/batch_test"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({"error": "Rate limit exceeded"}))
                .insert_header("Retry-After", "45"),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get_batch("batch_test").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::RateLimit { retry_after, .. } => {
            assert_eq!(retry_after, Some(45));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[tokio::test]
async fn test_get_batch_server_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/batch/batch_test"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get_batch("batch_test").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api { status_code, .. } => {
            assert_eq!(status_code, 500);
        }
        _ => panic!("Expected Api error"),
    }
}

// ==================== list_batches() Tests ====================

#[tokio::test]
async fn test_list_batches_success() {
    let mock_server = setup_mock_server().await;
    mock_list_batches_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list_batches(None).await;

    assert!(result.is_ok());
    let list = result.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list.data[0].batch_id, "batch_1");
    assert_eq!(list.data[0].status, BatchStatus::Completed);
}

#[tokio::test]
async fn test_list_batches_with_options() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/batches"))
        .and(query_param("limit", "50"))
        .and(query_param("offset", "10"))
        .and(query_param("status", "completed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "count": 0
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let options = ListBatchesOptions::new()
        .limit(50)
        .offset(10)
        .status(BatchStatus::Completed);

    let result = client.messages().list_batches(Some(options)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_batches_authentication_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/batches"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list_batches(None).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Authentication { .. }));
}

#[tokio::test]
async fn test_list_batches_not_found() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/batches"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Resource not found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list_batches(None).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NotFound { .. }));
}

#[tokio::test]
async fn test_list_batches_rate_limit() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/batches"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({"error": "Rate limit exceeded"}))
                .insert_header("Retry-After", "30"),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list_batches(None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::RateLimit { retry_after, .. } => {
            assert_eq!(retry_after, Some(30));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[tokio::test]
async fn test_list_batches_server_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/batches"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list_batches(None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api { status_code, .. } => {
            assert_eq!(status_code, 500);
        }
        _ => panic!("Expected Api error"),
    }
}

#[tokio::test]
async fn test_preview_batch_decodes_the_preview_the_api_sends() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages/batch/preview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total": 2,
            "sendable": 2,
            "blocked": 0,
            "duplicates": 0,
            "creditsNeeded": 4,
            "creditBalance": 100,
            "hasSufficientCredits": true,
            "pooled": false,
            "keyType": "live",
            "keyScopes": ["sms:send", "sms:read"],
            "hasWriteScope": true,
            "messagingProfile": {
                "id": "mp_1",
                "canSendDomestic": true,
                "canSendInternational": false,
                "verificationStatus": "verified",
                "verificationType": "toll_free"
            },
            "byCountry": {
                "US": { "count": 2, "credits": 4, "tier": "domestic", "allowed": true }
            },
            "blockedMessages": [],
            "compliance": {
                "messageType": "marketing",
                "optedOutBlocked": 0,
                "shaftBlocked": 0,
                "quietHoursBlocked": 0,
                "quietHoursRescheduled": 0,
                "shaftBlockedMessages": [],
                "quietHoursBlockedMessages": []
            },
            "warnings": ["Marketing messages are subject to quiet hours enforcement (8pm-8am recipient local time)"]
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let preview = client
        .messages()
        .preview_batch(SendBatchRequest {
            messages: vec![
                BatchMessageItem {
                    to: "+15555550100".to_string(),
                    text: "Hello Alice!".to_string(),
                    metadata: None,
                },
                BatchMessageItem {
                    to: "+15555550101".to_string(),
                    text: "Hello Bob!".to_string(),
                    metadata: None,
                },
            ],
            from: None,
            message_type: None,
            metadata: None,
        })
        .await
        .expect("preview should decode");

    assert!(preview.can_send);
    assert_eq!(preview.total_messages, 2);
    assert_eq!(preview.will_send, 2);
    assert_eq!(preview.credits_needed, 4);
    assert_eq!(preview.current_balance, 100);
    assert!(preview.has_enough_credits);
    assert_eq!(preview.duplicates, 0);
    assert_eq!(preview.key_type.as_deref(), Some("live"));
    assert!(preview.has_write_scope);
    assert_eq!(preview.warnings.len(), 1);
}

fn preview_body(
    key_type: &str,
    blocked: serde_json::Value,
    opted_out_blocked: i64,
    has_sufficient_credits: bool,
) -> serde_json::Value {
    let blocked_count = blocked.as_array().unwrap().len();
    json!({
        "total": 3,
        "sendable": 3 - blocked_count,
        "blocked": blocked_count,
        "duplicates": 0,
        "creditsNeeded": 4,
        "creditBalance": 0,
        "hasSufficientCredits": has_sufficient_credits,
        "pooled": false,
        "keyType": key_type,
        "keyScopes": ["sms:send"],
        "hasWriteScope": true,
        "messagingProfile": { "id": "mp_1", "canSendDomestic": true, "canSendInternational": false, "verificationStatus": "verified", "verificationType": "toll_free" },
        "byCountry": {},
        "blockedMessages": blocked,
        "compliance": {
            "messageType": "marketing",
            "optedOutBlocked": opted_out_blocked,
            "shaftBlocked": 0,
            "quietHoursBlocked": 0,
            "quietHoursRescheduled": 0,
            "shaftBlockedMessages": [],
            "quietHoursBlockedMessages": []
        },
        "warnings": []
    })
}

async fn preview_with(body: serde_json::Value) -> sendly::BatchPreviewResponse {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages/batch/preview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());
    client
        .messages()
        .preview_batch(SendBatchRequest {
            messages: vec![BatchMessageItem {
                to: "+15555550100".to_string(),
                text: "Hello!".to_string(),
                metadata: None,
            }],
            from: None,
            message_type: None,
            metadata: None,
        })
        .await
        .expect("preview should decode")
}

#[tokio::test]
async fn test_preview_batch_cannot_send_when_a_message_is_blocked_for_access() {
    let preview = preview_with(preview_body(
        "live",
        json!([{ "index": 2, "to": "+447700900123", "reason": "International messaging is not enabled" }]),
        0,
        true,
    ))
    .await;

    assert!(!preview.can_send);
    assert_eq!(preview.blocked_messages[0].index, 2);
    assert_eq!(
        preview.blocked_messages[0].reason,
        "International messaging is not enabled"
    );
}

#[tokio::test]
async fn test_preview_batch_can_send_past_opted_out_recipients() {
    let preview = preview_with(preview_body(
        "live",
        json!([{ "index": 0, "to": "+15555550100", "reason": "Contact has opted out (texted STOP)" }]),
        1,
        true,
    ))
    .await;

    assert!(preview.can_send);
}

#[tokio::test]
async fn test_preview_batch_needs_a_balance_only_for_a_live_key() {
    let live = preview_with(preview_body("live", json!([]), 0, false)).await;
    let test = preview_with(preview_body("test", json!([]), 0, false)).await;

    assert!(!live.can_send);
    assert!(test.can_send);
}

#[tokio::test]
async fn test_preview_batch_cannot_send_more_than_ten_thousand_messages() {
    let mut body = preview_body("live", json!([]), 0, true);
    body["total"] = json!(10_001);
    body["sendable"] = json!(10_001);
    body["warnings"] = json!([
        "Batch size exceeds 10,000 limit - sending it will be rejected, split it into batches of 10,000 or fewer"
    ]);
    let preview = preview_with(body).await;

    assert_eq!(preview.total_messages, 10_001);
    assert!(!preview.can_send);

    let mut body = preview_body("live", json!([]), 0, true);
    body["total"] = json!(10_000);
    body["sendable"] = json!(10_000);
    assert!(preview_with(body).await.can_send);
}

#[tokio::test]
async fn test_get_batch_reads_the_status_counts() {
    let mock_server = setup_mock_server().await;
    mock_get_batch_success().mount(&mock_server).await;
    let client = create_test_client(&mock_server.uri());

    let batch = client.messages().get_batch("batch_abc123").await.unwrap();

    assert_eq!(batch.delivered, 2);
    assert_eq!(batch.credits_reserved, 2);
    assert_eq!(batch.messages[0].id.as_deref(), Some("msg_1"));
    assert_eq!(
        batch.messages[0].delivered_at.as_deref(),
        Some("2025-01-15T10:00:30Z")
    );
}
