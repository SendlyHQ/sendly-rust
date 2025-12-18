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
                },
                BatchMessageItem {
                    to: "+15552222222".to_string(),
                    text: "Message 2".to_string(),
                },
            ],
            from: None,
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
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
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
                },
                BatchMessageItem {
                    to: "invalid-phone".to_string(),
                    text: "Invalid".to_string(),
                },
            ],
            from: None,
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
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
                },
                BatchMessageItem {
                    to: "+15552222222".to_string(),
                    text: "".to_string(),
                },
            ],
            from: None,
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
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
            }],
            from: None,
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
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
            }],
            from: None,
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
            }],
            from: None,
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
            }],
            from: None,
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
            }],
            from: None,
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
            }],
            from: None,
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
        Error::Validation { message } => {
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
        Error::NotFound { message } => {
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
