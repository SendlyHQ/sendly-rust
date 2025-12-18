use serde_json::json;
use wiremock::matchers::{header, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub const TEST_API_KEY: &str = "sk_test_v1_abc123";

/// Creates a mock server and returns it.
pub async fn setup_mock_server() -> MockServer {
    MockServer::start().await
}

/// Creates a Sendly client configured to use the mock server.
pub fn create_test_client(base_url: &str) -> sendly::Sendly {
    let config = sendly::SendlyConfig::new()
        .base_url(base_url)
        .timeout(std::time::Duration::from_secs(5))
        .max_retries(0); // No retries for tests by default

    sendly::Sendly::with_config(TEST_API_KEY, config)
}

/// Mock a successful message send.
pub fn mock_send_success() -> Mock {
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_abc123",
            "to": "+15551234567",
            "from": "SENDLY",
            "text": "Hello World",
            "status": "queued",
            "segments": 1,
            "creditsUsed": 1,
            "isSandbox": false,
            "createdAt": "2025-01-15T10:00:00Z",
            "deliveredAt": null,
            "error": null
        })))
}

/// Mock a successful message list.
pub fn mock_list_success() -> Mock {
    Mock::given(method("GET"))
        .and(path("/messages"))
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
                    "status": "queued",
                    "segments": 1,
                    "creditsUsed": 1,
                    "isSandbox": false
                }
            ],
            "count": 2
        })))
}

/// Mock a successful get message by ID.
pub fn mock_get_success() -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(r"^/messages/msg_[a-z0-9]+$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_abc123",
            "to": "+15551234567",
            "from": "SENDLY",
            "text": "Hello World",
            "status": "delivered",
            "segments": 1,
            "creditsUsed": 1,
            "isSandbox": false,
            "createdAt": "2025-01-15T10:00:00Z",
            "deliveredAt": "2025-01-15T10:01:00Z",
            "error": null
        })))
}

/// Mock a 401 authentication error.
pub fn mock_auth_error() -> Mock {
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API key"
        })))
}

/// Mock a 402 insufficient credits error.
pub fn mock_insufficient_credits() -> Mock {
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(402).set_body_json(json!({
            "error": "Insufficient credits. Please add credits to your account."
        })))
}

/// Mock a 404 not found error.
pub fn mock_not_found() -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(r"^/messages/.*$"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Message not found"
        })))
}

/// Mock a 429 rate limit error with Retry-After.
pub fn mock_rate_limit(retry_after: u64) -> Mock {
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({
                    "error": "Rate limit exceeded. Please try again later."
                }))
                .insert_header("Retry-After", retry_after.to_string().as_str()),
        )
}

/// Mock a 500 server error.
pub fn mock_server_error() -> Mock {
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
}

/// Mock a successful schedule message.
pub fn mock_schedule_success() -> Mock {
    Mock::given(method("POST"))
        .and(path("/messages/schedule"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "sched_abc123",
            "to": "+15551234567",
            "from": "SENDLY",
            "text": "Scheduled message",
            "scheduledAt": "2025-01-20T10:00:00Z",
            "status": "scheduled",
            "creditsReserved": 1,
            "createdAt": "2025-01-15T10:00:00Z"
        })))
}

/// Mock a successful list scheduled messages.
pub fn mock_list_scheduled_success() -> Mock {
    Mock::given(method("GET"))
        .and(path("/messages/scheduled"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "sched_1",
                    "to": "+15551111111",
                    "text": "Scheduled 1",
                    "scheduledAt": "2025-01-20T10:00:00Z",
                    "status": "scheduled",
                    "creditsReserved": 1,
                    "createdAt": "2025-01-15T10:00:00Z"
                }
            ],
            "count": 1
        })))
}

/// Mock a successful get scheduled message by ID.
pub fn mock_get_scheduled_success() -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(r"^/messages/scheduled/sched_[a-z0-9]+$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "sched_abc123",
            "to": "+15551234567",
            "from": "SENDLY",
            "text": "Scheduled message",
            "scheduledAt": "2025-01-20T10:00:00Z",
            "status": "scheduled",
            "creditsReserved": 1,
            "createdAt": "2025-01-15T10:00:00Z"
        })))
}

/// Mock a successful cancel scheduled message.
pub fn mock_cancel_scheduled_success() -> Mock {
    Mock::given(method("DELETE"))
        .and(path_regex(r"^/messages/scheduled/sched_[a-z0-9]+$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "sched_abc123",
            "status": "cancelled",
            "creditsRefunded": 1
        })))
}

/// Mock a successful batch send.
pub fn mock_batch_send_success() -> Mock {
    Mock::given(method("POST"))
        .and(path("/messages/batch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "batchId": "batch_abc123",
            "status": "processing",
            "total": 2,
            "queued": 2,
            "sent": 0,
            "failed": 0,
            "creditsUsed": 0,
            "messages": [],
            "createdAt": "2025-01-15T10:00:00Z"
        })))
}

/// Mock a successful get batch.
pub fn mock_get_batch_success() -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(r"^/messages/batch/batch_[a-z0-9]+$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "batchId": "batch_abc123",
            "status": "completed",
            "total": 2,
            "queued": 0,
            "sent": 2,
            "failed": 0,
            "creditsUsed": 2,
            "messages": [
                {
                    "to": "+15551111111",
                    "messageId": "msg_1",
                    "status": "queued",
                    "error": null
                },
                {
                    "to": "+15552222222",
                    "messageId": "msg_2",
                    "status": "queued",
                    "error": null
                }
            ],
            "createdAt": "2025-01-15T10:00:00Z",
            "completedAt": "2025-01-15T10:01:00Z"
        })))
}

/// Mock a successful list batches.
pub fn mock_list_batches_success() -> Mock {
    Mock::given(method("GET"))
        .and(path("/messages/batches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "batchId": "batch_1",
                    "status": "completed",
                    "total": 2,
                    "queued": 0,
                    "sent": 2,
                    "failed": 0,
                    "creditsUsed": 2,
                    "messages": [],
                    "createdAt": "2025-01-15T10:00:00Z",
                    "completedAt": "2025-01-15T10:01:00Z"
                }
            ],
            "count": 1
        })))
}
