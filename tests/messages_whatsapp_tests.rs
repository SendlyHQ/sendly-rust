mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use sendly::{
    Error, MessageStatus, SendWhatsAppMessageRequest, WhatsAppMessageCategory, WhatsAppMessageKind,
    WhatsAppTemplateButtonVariables, WhatsAppTemplateSendParams,
};
use serde_json::json;
use std::collections::HashMap;
use wiremock::matchers::{body_json, body_partial_json, header, method, path};
use wiremock::{Mock, ResponseTemplate};

// ==================== send_whatsapp() Text Tests ====================

#[tokio::test]
async fn test_send_whatsapp_text_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .and(body_json(json!({
            "channel": "whatsapp",
            "to": "+15551234567",
            "from": "+15559876543",
            "text": "Your table is ready!"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_wa_123",
            "channel": "whatsapp",
            "message_format": "whatsapp",
            "to": "+15551234567",
            "from": "+15559876543",
            "text": "Your table is ready!",
            "status": "queued",
            "segments": 1,
            "creditsUsed": 3,
            "whatsapp": {
                "kind": "text",
                "messageId": null
            },
            "createdAt": "2026-07-30T10:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_whatsapp(
            SendWhatsAppMessageRequest::new("+15551234567", "+15559876543")
                .with_text("Your table is ready!"),
        )
        .await;

    assert!(result.is_ok());
    let message = result.unwrap();
    assert_eq!(message.id, "msg_wa_123");
    assert_eq!(message.channel, "whatsapp");
    assert_eq!(message.message_format, "whatsapp");
    assert_eq!(message.to, "+15551234567");
    assert_eq!(message.from, "+15559876543");
    assert_eq!(message.text.as_deref(), Some("Your table is ready!"));
    assert_eq!(message.status, MessageStatus::Queued);
    assert_eq!(message.segments, 1);
    assert_eq!(message.credits_used, 3);
    assert_eq!(message.whatsapp.kind, WhatsAppMessageKind::Text);
    assert!(message.whatsapp.template.is_none());
    assert!(message.whatsapp.message_id.is_none());
}

// ==================== send_whatsapp() Media Tests ====================

#[tokio::test]
async fn test_send_whatsapp_media_with_caption() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(body_json(json!({
            "channel": "whatsapp",
            "to": "+15551234567",
            "from": "+15559876543",
            "text": "Here is your receipt",
            "mediaUrls": ["https://example.com/receipt.pdf"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_wa_456",
            "channel": "whatsapp",
            "message_format": "whatsapp",
            "to": "+15551234567",
            "from": "+15559876543",
            "text": null,
            "status": "queued",
            "segments": 1,
            "creditsUsed": 3,
            "whatsapp": {
                "kind": "media",
                "messageId": null
            },
            "createdAt": "2026-07-30T10:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_whatsapp(
            SendWhatsAppMessageRequest::new("+15551234567", "+15559876543")
                .with_media_urls(vec!["https://example.com/receipt.pdf".to_string()])
                .with_text("Here is your receipt"),
        )
        .await;

    assert!(result.is_ok());
    let message = result.unwrap();
    assert_eq!(message.whatsapp.kind, WhatsAppMessageKind::Media);
    assert!(message.text.is_none());
}

// ==================== send_whatsapp() Template Tests ====================

#[tokio::test]
async fn test_send_whatsapp_template_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(body_partial_json(json!({
            "channel": "whatsapp",
            "to": "+15551234567",
            "from": "+15559876543",
            "template": {
                "name": "order_shipped",
                "language": "en_US",
                "variables": { "1": "Acme Inc", "2": "#4821" },
                "buttons": [
                    { "index": 0, "variables": { "1": "4821" } }
                ]
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_wa_789",
            "channel": "whatsapp",
            "message_format": "whatsapp",
            "to": "+15551234567",
            "from": "+15559876543",
            "text": null,
            "status": "queued",
            "segments": 1,
            "creditsUsed": 4,
            "whatsapp": {
                "kind": "template",
                "template": {
                    "name": "order_shipped",
                    "language": "en_US",
                    "category": "utility"
                },
                "messageId": null
            },
            "createdAt": "2026-07-30T10:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let mut variables = HashMap::new();
    variables.insert("1".to_string(), "Acme Inc".to_string());
    variables.insert("2".to_string(), "#4821".to_string());
    let mut button_variables = HashMap::new();
    button_variables.insert("1".to_string(), "4821".to_string());
    let result = client
        .messages()
        .send_whatsapp(
            SendWhatsAppMessageRequest::new("+15551234567", "+15559876543").with_template(
                WhatsAppTemplateSendParams::new("order_shipped", "en_US")
                    .with_variables(variables)
                    .with_buttons(vec![WhatsAppTemplateButtonVariables {
                        index: 0,
                        variables: button_variables,
                    }]),
            ),
        )
        .await;

    assert!(result.is_ok());
    let message = result.unwrap();
    assert_eq!(message.whatsapp.kind, WhatsAppMessageKind::Template);
    let template = message
        .whatsapp
        .template
        .expect("template details should be present");
    assert_eq!(template.name, "order_shipped");
    assert_eq!(template.language, "en_US");
    assert_eq!(template.category, WhatsAppMessageCategory::Utility);
    assert_eq!(message.credits_used, 4);
}

// ==================== send_whatsapp() Validation Tests ====================

#[tokio::test]
async fn test_send_whatsapp_invalid_to_phone() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send_whatsapp(
            SendWhatsAppMessageRequest::new("invalid-phone", "+15559876543").with_text("Hello"),
        )
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
async fn test_send_whatsapp_invalid_from_phone() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send_whatsapp(
            SendWhatsAppMessageRequest::new("+15551234567", "SENDLY").with_text("Hello"),
        )
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
async fn test_send_whatsapp_missing_content() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .messages()
        .send_whatsapp(SendWhatsAppMessageRequest::new(
            "+15551234567",
            "+15559876543",
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
            assert!(message.contains("Provide 'text', 'media_urls', or 'template'"));
        }
        _ => panic!("Expected Validation error"),
    }
}

// ==================== send_whatsapp() Error Tests ====================

#[tokio::test]
async fn test_send_whatsapp_window_closed() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "whatsapp_window_closed",
            "message": "The 24-hour window is closed. Send an approved template instead."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_whatsapp(
            SendWhatsAppMessageRequest::new("+15551234567", "+15559876543")
                .with_text("Your table is ready!"),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
            assert!(message.contains("24-hour window"));
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_send_whatsapp_sender_not_connected() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "whatsapp_sender_not_connected",
            "message": "The 'from' number is not connected to WhatsApp"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .messages()
        .send_whatsapp(
            SendWhatsAppMessageRequest::new("+15551234567", "+15559876543").with_text("Hello"),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => {
            assert!(message.contains("not connected to WhatsApp"));
        }
        _ => panic!("Expected Validation error"),
    }
}
