mod common;

use common::{
    create_test_client, mock_list_scheduled_success, mock_schedule_success, setup_mock_server,
};
use common::{mock_cancel_scheduled_success, mock_get_scheduled_success};
use sendly::{Error, ListScheduledMessagesOptions, ScheduleMessageRequest, ScheduledMessageStatus};
use serde_json::json;
use wiremock::matchers::{method, path, path_regex, query_param};
use wiremock::{Mock, ResponseTemplate};

// ==================== schedule() Tests ====================

#[tokio::test]
async fn test_schedule_success() {
    let mock_server = setup_mock_server().await;
    mock_schedule_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .schedule(ScheduleMessageRequest {
            to: "+15551234567".to_string(),
            text: "Scheduled message".to_string(),
            scheduled_at: "2025-01-20T10:00:00Z".to_string(),
            from: None,
        })
        .await;

    assert!(result.is_ok());
    let scheduled = result.unwrap();
    assert_eq!(scheduled.id, "sched_abc123");
    assert_eq!(scheduled.to, "+15551234567");
    assert_eq!(scheduled.text, "Scheduled message");
    assert_eq!(scheduled.status, ScheduledMessageStatus::Scheduled);
    assert_eq!(scheduled.credits_reserved, 1);
}

#[tokio::test]
async fn test_schedule_invalid_phone() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .schedule(ScheduleMessageRequest {
            to: "invalid-phone".to_string(),
            text: "Test".to_string(),
            scheduled_at: "2025-01-20T10:00:00Z".to_string(),
            from: None,
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
async fn test_schedule_empty_text() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .schedule(ScheduleMessageRequest {
            to: "+15551234567".to_string(),
            text: "".to_string(),
            scheduled_at: "2025-01-20T10:00:00Z".to_string(),
            from: None,
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

#[tokio::test]
async fn test_schedule_text_too_long() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let long_text = "a".repeat(1601);

    let result = client
        .messages()
        .schedule(ScheduleMessageRequest {
            to: "+15551234567".to_string(),
            text: long_text,
            scheduled_at: "2025-01-20T10:00:00Z".to_string(),
            from: None,
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
            assert!(message.contains("exceeds maximum length"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_schedule_empty_scheduled_at() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .schedule(ScheduleMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
            scheduled_at: "".to_string(),
            from: None,
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
            assert!(message.contains("scheduled_at is required"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_schedule_authentication_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages/schedule"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .schedule(ScheduleMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
            scheduled_at: "2025-01-20T10:00:00Z".to_string(),
            from: None,
        })
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Authentication { .. }));
}

#[tokio::test]
async fn test_schedule_insufficient_credits() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages/schedule"))
        .respond_with(ResponseTemplate::new(402).set_body_json(json!({
            "error": "Insufficient credits"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .schedule(ScheduleMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
            scheduled_at: "2025-01-20T10:00:00Z".to_string(),
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
async fn test_schedule_rate_limit() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages/schedule"))
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
        .schedule(ScheduleMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
            scheduled_at: "2025-01-20T10:00:00Z".to_string(),
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
async fn test_schedule_server_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/messages/schedule"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .schedule(ScheduleMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
            scheduled_at: "2025-01-20T10:00:00Z".to_string(),
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

// ==================== list_scheduled() Tests ====================

#[tokio::test]
async fn test_list_scheduled_success() {
    let mock_server = setup_mock_server().await;
    mock_list_scheduled_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list_scheduled(None).await;

    assert!(result.is_ok());
    let list = result.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list.data[0].id, "sched_1");
    assert_eq!(list.data[0].status, ScheduledMessageStatus::Scheduled);
}

#[tokio::test]
async fn test_list_scheduled_with_options() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/scheduled"))
        .and(query_param("limit", "50"))
        .and(query_param("offset", "10"))
        .and(query_param("status", "scheduled"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "count": 0
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let options = ListScheduledMessagesOptions::new()
        .limit(50)
        .offset(10)
        .status(ScheduledMessageStatus::Scheduled);

    let result = client.messages().list_scheduled(Some(options)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_scheduled_authentication_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/scheduled"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list_scheduled(None).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Authentication { .. }));
}

#[tokio::test]
async fn test_list_scheduled_not_found() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/scheduled"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Resource not found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list_scheduled(None).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NotFound { .. }));
}

#[tokio::test]
async fn test_list_scheduled_rate_limit() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/scheduled"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({"error": "Rate limit exceeded"}))
                .insert_header("Retry-After", "30"),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list_scheduled(None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::RateLimit { retry_after, .. } => {
            assert_eq!(retry_after, Some(30));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[tokio::test]
async fn test_list_scheduled_server_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/scheduled"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().list_scheduled(None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api { status_code, .. } => {
            assert_eq!(status_code, 500);
        }
        _ => panic!("Expected Api error"),
    }
}

// ==================== get_scheduled() Tests ====================

#[tokio::test]
async fn test_get_scheduled_success() {
    let mock_server = setup_mock_server().await;
    mock_get_scheduled_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get_scheduled("sched_abc123").await;

    assert!(result.is_ok());
    let scheduled = result.unwrap();
    assert_eq!(scheduled.id, "sched_abc123");
    assert_eq!(scheduled.status, ScheduledMessageStatus::Scheduled);
}

#[tokio::test]
async fn test_get_scheduled_empty_id() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get_scheduled("").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
            assert!(message.contains("Scheduled message ID is required"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_get_scheduled_not_found() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path_regex(r"^/messages/scheduled/.*$"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Scheduled message not found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get_scheduled("sched_nonexistent").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::NotFound { message } => {
            assert!(message.contains("not found"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_get_scheduled_authentication_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/scheduled/sched_test"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get_scheduled("sched_test").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Authentication { .. }));
}

#[tokio::test]
async fn test_get_scheduled_rate_limit() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/scheduled/sched_test"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({"error": "Rate limit exceeded"}))
                .insert_header("Retry-After", "45"),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get_scheduled("sched_test").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::RateLimit { retry_after, .. } => {
            assert_eq!(retry_after, Some(45));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[tokio::test]
async fn test_get_scheduled_server_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/messages/scheduled/sched_test"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().get_scheduled("sched_test").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api { status_code, .. } => {
            assert_eq!(status_code, 500);
        }
        _ => panic!("Expected Api error"),
    }
}

// ==================== cancel_scheduled() Tests ====================

#[tokio::test]
async fn test_cancel_scheduled_success() {
    let mock_server = setup_mock_server().await;
    mock_cancel_scheduled_success().mount(&mock_server).await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().cancel_scheduled("sched_abc123").await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.id, "sched_abc123");
    assert_eq!(response.status, ScheduledMessageStatus::Cancelled);
    assert_eq!(response.credits_refunded, 1);
}

#[tokio::test]
async fn test_cancel_scheduled_empty_id() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client.messages().cancel_scheduled("").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
            assert!(message.contains("Scheduled message ID is required"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_cancel_scheduled_not_found() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("DELETE"))
        .and(path_regex(r"^/messages/scheduled/.*$"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Scheduled message not found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .cancel_scheduled("sched_nonexistent")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::NotFound { message } => {
            assert!(message.contains("not found"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_cancel_scheduled_authentication_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("DELETE"))
        .and(path("/messages/scheduled/sched_test"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().cancel_scheduled("sched_test").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Authentication { .. }));
}

#[tokio::test]
async fn test_cancel_scheduled_rate_limit() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("DELETE"))
        .and(path("/messages/scheduled/sched_test"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({"error": "Rate limit exceeded"}))
                .insert_header("Retry-After", "60"),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().cancel_scheduled("sched_test").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::RateLimit { retry_after, .. } => {
            assert_eq!(retry_after, Some(60));
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[tokio::test]
async fn test_cancel_scheduled_server_error() {
    let mock_server = setup_mock_server().await;

    Mock::given(method("DELETE"))
        .and(path("/messages/scheduled/sched_test"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client.messages().cancel_scheduled("sched_test").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api { status_code, .. } => {
            assert_eq!(status_code, 500);
        }
        _ => panic!("Expected Api error"),
    }
}
