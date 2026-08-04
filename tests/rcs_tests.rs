mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use sendly::Error;
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

// ==================== agents().list() Tests ====================

#[tokio::test]
async fn test_agents_list_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/agents"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agents": [
                {
                    "id": "rcsa_abc123",
                    "name": "Acme Inc",
                    "status": "approved",
                    "useCase": "OTP",
                    "sendable": true,
                    "createdAt": "2026-07-28T10:00:00Z"
                },
                {
                    "id": "rcsa_def456",
                    "name": "Acme Promos",
                    "status": "draft",
                    "useCase": null,
                    "sendable": false,
                    "createdAt": "2026-07-30T10:00:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().agents().list().await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.agents.len(), 2);
    let approved = &response.agents[0];
    assert_eq!(approved.id, "rcsa_abc123");
    assert_eq!(approved.name, "Acme Inc");
    assert_eq!(approved.status, "approved");
    assert_eq!(approved.use_case.as_deref(), Some("OTP"));
    assert!(approved.sendable);
    assert_eq!(approved.created_at, "2026-07-28T10:00:00Z");
    let draft = &response.agents[1];
    assert!(draft.use_case.is_none());
    assert!(!draft.sendable);
}

#[tokio::test]
async fn test_agents_list_empty() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/agents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "agents": [] })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().agents().list().await;

    assert!(result.is_ok());
    assert!(result.unwrap().agents.is_empty());
}

#[tokio::test]
async fn test_agents_list_channel_not_enabled() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/agents"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "not_found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().agents().list().await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

// ==================== capability() Tests ====================

#[tokio::test]
async fn test_capability_with_agent_id() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/capability"))
        .and(query_param("to", "+15551234567"))
        .and(query_param("agentId", "rcsa_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "to": "+15551234567",
            "agentId": "rcsa_abc123",
            "capable": true,
            "features": ["RICHCARD_STANDALONE", "ACTION_OPEN_URL"]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .capability("+15551234567", Some("rcsa_abc123"))
        .await;

    assert!(result.is_ok());
    let capability = result.unwrap();
    assert_eq!(capability.to, "+15551234567");
    assert_eq!(capability.agent_id, "rcsa_abc123");
    assert!(capability.capable);
    assert_eq!(capability.features.len(), 2);
    assert_eq!(capability.features[0], "RICHCARD_STANDALONE");
}

#[tokio::test]
async fn test_capability_without_agent_id_omits_param() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/capability"))
        .and(query_param("to", "+15551234567"))
        .and(query_param_is_missing("agentId"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "to": "+15551234567",
            "agentId": "rcsa_abc123",
            "capable": false,
            "features": []
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().capability("+15551234567", None).await;

    assert!(result.is_ok());
    let capability = result.unwrap();
    assert!(!capability.capable);
    assert!(capability.features.is_empty());
}

#[tokio::test]
async fn test_capability_requires_live_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/capability"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": "rcs_requires_live_key",
            "message": "RCS capability checks require a live API key. Test keys cannot query RCS capability."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().capability("+15551234567", None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api {
            status_code,
            message,
            ..
        } => {
            assert_eq!(status_code, 403);
            assert!(message.contains("live API key"));
        }
        other => panic!("Expected Api error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_capability_agent_ambiguous() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/capability"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "rcs_agent_ambiguous",
            "message": "This workspace has more than one RCS agent. Pass agentId to pick one."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().capability("+15551234567", None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("more than one RCS agent")),
        other => panic!("Expected Validation error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_capability_agent_not_ready() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/capability"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": "rcs_agent_not_ready",
            "agentStatus": "pending",
            "message": "This workspace's RCS agent isn't approved for sending yet."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .capability("+15551234567", Some("rcsa_abc123"))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api {
            status_code,
            message,
            ..
        } => {
            assert_eq!(status_code, 403);
            assert!(message.contains("isn't approved for sending"));
        }
        other => panic!("Expected Api error, got {:?}", other),
    }
}
