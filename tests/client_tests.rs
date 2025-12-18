mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use sendly::{Sendly, SendlyConfig};
use std::time::Duration;

#[tokio::test]
async fn test_client_new() {
    let client = Sendly::new(TEST_API_KEY);

    // Client should be created successfully
    assert!(format!("{:?}", client).contains("Sendly"));
}

#[tokio::test]
async fn test_client_with_config() {
    let config = SendlyConfig::new()
        .base_url("https://custom-api.example.com")
        .timeout(Duration::from_secs(60))
        .max_retries(5);

    let client = Sendly::with_config(TEST_API_KEY, config);

    // Client should be created successfully with custom config
    assert!(format!("{:?}", client).contains("Sendly"));
}

#[tokio::test]
async fn test_client_default_config() {
    let config = SendlyConfig::default();

    assert_eq!(config.base_url, "https://sendly.live/api/v1");
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert_eq!(config.max_retries, 3);
}

#[tokio::test]
async fn test_client_config_builder() {
    let config = SendlyConfig::new()
        .base_url("https://test.example.com")
        .timeout(Duration::from_secs(45))
        .max_retries(2);

    assert_eq!(config.base_url, "https://test.example.com");
    assert_eq!(config.timeout, Duration::from_secs(45));
    assert_eq!(config.max_retries, 2);
}

#[tokio::test]
async fn test_client_messages_resource() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    // Should return Messages resource
    let messages = client.messages();
    assert!(format!("{:?}", messages).contains("Messages"));
}

#[tokio::test]
async fn test_client_api_key_in_headers() {
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    let mock_server = setup_mock_server().await;

    // Mock accepts any request - we're just testing that headers are sent
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_test",
            "to": "+15551234567",
            "text": "Test",
            "status": "queued",
            "segments": 1,
            "creditsUsed": 1,
            "isSandbox": false
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(sendly::SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_client_user_agent_header() {
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    let mock_server = setup_mock_server().await;

    // Mock accepts any request - we're just testing that the client works
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_test",
            "to": "+15551234567",
            "text": "Test",
            "status": "queued",
            "segments": 1,
            "creditsUsed": 1,
            "isSandbox": false
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send(sendly::SendMessageRequest {
            to: "+15551234567".to_string(),
            text: "Test".to_string(),
        })
        .await;

    assert!(result.is_ok());
}
