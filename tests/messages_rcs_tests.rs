mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use sendly::{
    Error, MessageStatus, RcsCard, RcsCardOrientation, RcsMessageKind, RcsSuggestion,
    SendRcsMessageRequest,
};
use serde_json::json;
use wiremock::matchers::{body_json, body_partial_json, header, method, path};
use wiremock::{Mock, ResponseTemplate};

// ==================== send_rcs() Text Tests ====================

#[tokio::test]
async fn test_send_rcs_text_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .and(body_json(json!({
            "channel": "rcs",
            "to": "+15551234567",
            "text": "Your order #4821 has shipped!"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "msg_rcs_123",
            "channel": "rcs",
            "message_format": "rcs",
            "to": "+15551234567",
            "from": "Acme Inc",
            "text": "Your order #4821 has shipped!",
            "status": "sent",
            "segments": 1,
            "creditsUsed": 2,
            "rcs": {
                "kind": "text",
                "agentId": "rcsa_abc123",
                "agentName": "Acme Inc"
            },
            "createdAt": "2026-08-01T10:00:00Z",
            "metadata": {}
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_rcs(
            SendRcsMessageRequest::new("+15551234567").with_text("Your order #4821 has shipped!"),
        )
        .await;

    assert!(result.is_ok());
    let message = result.unwrap();
    assert_eq!(message.id, "msg_rcs_123");
    assert_eq!(message.channel, "rcs");
    assert_eq!(message.message_format, "rcs");
    assert_eq!(message.to, "+15551234567");
    assert_eq!(message.from, "Acme Inc");
    assert_eq!(message.status, MessageStatus::Sent);
    assert_eq!(message.segments, 1);
    assert_eq!(message.credits_used, 2);
    assert_eq!(message.rcs.kind, Some(RcsMessageKind::Text));
    assert_eq!(message.rcs.agent_id, "rcsa_abc123");
    assert_eq!(message.rcs.agent_name.as_deref(), Some("Acme Inc"));
    assert!(!message.fell_back_to_sms());
    assert!(message.fell_back_to.is_none());
    assert!(!message.rcs.suggestions_dropped);
}

#[tokio::test]
async fn test_send_rcs_text_with_suggestions() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(body_json(json!({
            "channel": "rcs",
            "to": "+15551234567",
            "agentId": "rcsa_abc123",
            "text": "Your order #4821 has shipped!",
            "suggestions": [
                { "reply": { "text": "Track it", "postbackData": "track_4821" } },
                {
                    "action": {
                        "text": "View receipt",
                        "postbackData": "receipt_4821",
                        "url": "https://example.com/receipts/4821"
                    }
                }
            ]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "msg_rcs_456",
            "channel": "rcs",
            "message_format": "rcs",
            "to": "+15551234567",
            "from": "Acme Inc",
            "text": "Your order #4821 has shipped!",
            "status": "sent",
            "segments": 1,
            "creditsUsed": 2,
            "rcs": {
                "kind": "text",
                "agentId": "rcsa_abc123",
                "agentName": "Acme Inc"
            },
            "createdAt": "2026-08-01T10:00:00Z",
            "metadata": {}
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_rcs(
            SendRcsMessageRequest::new("+15551234567")
                .with_agent_id("rcsa_abc123")
                .with_text("Your order #4821 has shipped!")
                .with_suggestions(vec![
                    RcsSuggestion::reply("Track it", "track_4821"),
                    RcsSuggestion::action(
                        "View receipt",
                        "receipt_4821",
                        "https://example.com/receipts/4821",
                    ),
                ]),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().rcs.kind, Some(RcsMessageKind::Text));
}

// ==================== send_rcs() Card Tests ====================

#[tokio::test]
async fn test_send_rcs_card_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(body_json(json!({
            "channel": "rcs",
            "to": "+15551234567",
            "card": {
                "title": "Your table is ready",
                "description": "Head to the host stand — we'll hold it for 10 minutes.",
                "mediaUrl": "https://example.com/table.jpg",
                "orientation": "horizontal",
                "suggestions": [
                    { "reply": { "text": "On my way", "postbackData": "otw" } }
                ]
            }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "msg_rcs_789",
            "channel": "rcs",
            "message_format": "rcs",
            "to": "+15551234567",
            "from": "Acme Inc",
            "text": null,
            "status": "sent",
            "segments": 1,
            "creditsUsed": 2,
            "rcs": {
                "kind": "card",
                "agentId": "rcsa_abc123",
                "agentName": "Acme Inc"
            },
            "createdAt": "2026-08-01T10:00:00Z",
            "metadata": {}
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_rcs(
            SendRcsMessageRequest::new("+15551234567").with_card(
                RcsCard::new(
                    "Your table is ready",
                    "Head to the host stand — we'll hold it for 10 minutes.",
                )
                .with_media_url("https://example.com/table.jpg")
                .with_orientation(RcsCardOrientation::Horizontal)
                .with_suggestions(vec![RcsSuggestion::reply("On my way", "otw")]),
            ),
        )
        .await;

    assert!(result.is_ok());
    let message = result.unwrap();
    assert_eq!(message.rcs.kind, Some(RcsMessageKind::Card));
    assert!(message.text.is_none());
    assert!(!message.fell_back_to_sms());
}

#[tokio::test]
async fn test_send_rcs_card_not_supported_for_recipient() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "rcs_not_supported_for_recipient",
            "message": "This recipient's device or network doesn't support RCS, and a rich card has no SMS form."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_rcs(
            SendRcsMessageRequest::new("+15551234567").with_card(RcsCard::new(
                "Your table is ready",
                "We'll hold it for 10 minutes.",
            )),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("no SMS form")),
        other => panic!("Expected Validation error, got {:?}", other),
    }
}

// ==================== send_rcs() SMS Fallback Tests ====================

#[tokio::test]
async fn test_send_rcs_falls_back_to_sms() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(body_partial_json(json!({ "channel": "rcs" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "msg_rcs_fallback",
            "channel": "sms",
            "fellBackTo": "sms",
            "message_format": "sms",
            "to": "+15551234567",
            "from": "+18885550101",
            "text": "Your order #4821 has shipped!",
            "status": "sent",
            "segments": 1,
            "creditsUsed": 2,
            "rcs": {
                "requestedChannel": "rcs",
                "agentId": "rcsa_abc123",
                "suggestionsDropped": true
            },
            "createdAt": "2026-08-01T10:00:00Z",
            "metadata": {}
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_rcs(
            SendRcsMessageRequest::new("+15551234567")
                .with_text("Your order #4821 has shipped!")
                .with_suggestions(vec![RcsSuggestion::reply("Track it", "track_4821")]),
        )
        .await;

    assert!(result.is_ok());
    let message = result.unwrap();
    assert!(message.fell_back_to_sms());
    assert_eq!(message.channel, "sms");
    assert_eq!(message.fell_back_to.as_deref(), Some("sms"));
    assert_eq!(message.message_format, "sms");
    assert_eq!(message.from, "+18885550101");
    assert!(message.rcs.kind.is_none());
    assert_eq!(message.rcs.requested_channel.as_deref(), Some("rcs"));
    assert_eq!(message.rcs.agent_id, "rcsa_abc123");
    assert!(message.rcs.suggestions_dropped);
}

#[tokio::test]
async fn test_send_rcs_fallback_disabled() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(body_json(json!({
            "channel": "rcs",
            "to": "+15551234567",
            "text": "RCS only.",
            "fallbackToSms": false
        })))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "rcs_not_supported_for_recipient",
            "message": "This recipient's device or network doesn't support RCS."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_rcs(
            SendRcsMessageRequest::new("+15551234567")
                .with_text("RCS only.")
                .with_fallback_to_sms(false),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("doesn't support RCS")),
        other => panic!("Expected Validation error, got {:?}", other),
    }
}

// ==================== send_rcs() Validation Tests ====================

#[tokio::test]
async fn test_send_rcs_requires_text_or_card() {
    let client = create_test_client("http://localhost:1");
    let result = client
        .messages()
        .send_rcs(SendRcsMessageRequest::new("+15551234567"))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("exactly one")),
        other => panic!("Expected Validation error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_send_rcs_rejects_text_and_card_together() {
    let client = create_test_client("http://localhost:1");
    let result = client
        .messages()
        .send_rcs(
            SendRcsMessageRequest::new("+15551234567")
                .with_text("Hi")
                .with_card(RcsCard::new("Title", "Description")),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("exactly one")),
        other => panic!("Expected Validation error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_send_rcs_rejects_suggestions_on_a_card() {
    let client = create_test_client("http://localhost:1");
    let result = client
        .messages()
        .send_rcs(
            SendRcsMessageRequest::new("+15551234567")
                .with_card(RcsCard::new("Title", "Description"))
                .with_suggestions(vec![RcsSuggestion::reply("Track it", "track_4821")]),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("ride on text messages")),
        other => panic!("Expected Validation error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_send_rcs_rejects_invalid_phone() {
    let client = create_test_client("http://localhost:1");
    let result = client
        .messages()
        .send_rcs(SendRcsMessageRequest::new("5551234567").with_text("Hi"))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { .. } => {}
        other => panic!("Expected Validation error, got {:?}", other),
    }
}

// ==================== send_rcs() API Error Tests ====================

#[tokio::test]
async fn test_send_rcs_requires_live_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": "rcs_requires_live_key",
            "message": "RCS messages require a live API key. Test keys cannot simulate RCS delivery."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_rcs(SendRcsMessageRequest::new("+15551234567").with_text("Hi"))
        .await;

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
async fn test_send_rcs_not_enabled() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "rcs_not_enabled",
            "message": "RCS is not enabled for your account."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_rcs(SendRcsMessageRequest::new("+15551234567").with_text("Hi"))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::NotFound { message } => assert!(message.contains("not enabled")),
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_send_rcs_insufficient_credits() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(402).set_body_json(json!({
            "error": "insufficient_credits",
            "message": "Insufficient credits. Please add credits to your account.",
            "creditsNeeded": 2,
            "currentBalance": 0
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_rcs(SendRcsMessageRequest::new("+15551234567").with_text("Hi"))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::InsufficientCredits { message } => assert!(message.contains("Insufficient credits")),
        other => panic!("Expected InsufficientCredits error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_send_rcs_recipient_opted_out() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": "recipient_opted_out",
            "message": "Recipient +15551234567 has opted out of messages. Cannot send."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_rcs(SendRcsMessageRequest::new("+15551234567").with_text("Hi"))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api {
            status_code,
            message,
            ..
        } => {
            assert_eq!(status_code, 403);
            assert!(message.contains("opted out"));
        }
        other => panic!("Expected Api error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_send_rcs_send_failed() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(502).set_body_json(json!({
            "error": "rcs_send_failed",
            "errorCode": "E001",
            "message": "The message couldn't be delivered."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_rcs(SendRcsMessageRequest::new("+15551234567").with_text("Hi"))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api { status_code, .. } => assert_eq!(status_code, 502),
        other => panic!("Expected Api error, got {:?}", other),
    }
}
