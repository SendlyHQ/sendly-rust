mod common;

use common::{create_test_client, setup_mock_server};
use sendly::{
    Error, MessageStatus, SendWhatsAppMessageRequest, WhatsAppSmsFallback,
    WhatsAppSmsFallbackReason, WhatsAppSmsFallbackStatus, WhatsAppTemplateSendParams,
};
use serde_json::{json, Value};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sent_body() -> Value {
    json!({
        "id": "msg_wa_1",
        "channel": "whatsapp",
        "message_format": "whatsapp",
        "messageFormat": "whatsapp",
        "to": "+15551234567",
        "from": "+15559876543",
        "text": "Your table is ready!",
        "status": "sent",
        "direction": "outbound",
        "segments": 1,
        "creditsUsed": 1,
        "whatsapp": { "kind": "text", "messageId": "wamid.1" },
        "createdAt": "2026-10-17T10:00:00Z",
        "metadata": {}
    })
}

async fn mount_send(mock_server: &MockServer, request_body: Value, response: ResponseTemplate) {
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(body_json(request_body))
        .respond_with(response)
        .expect(1)
        .mount(mock_server)
        .await;
}

fn text_request() -> SendWhatsAppMessageRequest {
    SendWhatsAppMessageRequest::new("+15551234567", "+15559876543")
        .with_text("Your table is ready!")
}

// ==================== fallbackToSms on the request ====================

#[tokio::test]
async fn test_fallback_to_sms_true_is_sent() {
    let mock_server = setup_mock_server().await;
    mount_send(
        &mock_server,
        json!({
            "channel": "whatsapp",
            "to": "+15551234567",
            "from": "+15559876543",
            "text": "Your table is ready!",
            "fallbackToSms": true
        }),
        ResponseTemplate::new(201).set_body_json(sent_body()),
    )
    .await;

    let client = create_test_client(&mock_server.uri());
    client
        .messages()
        .send_whatsapp(text_request().with_fallback_to_sms(true))
        .await
        .expect("send");
}

#[tokio::test]
async fn test_fallback_to_sms_false_is_sent() {
    let mock_server = setup_mock_server().await;
    mount_send(
        &mock_server,
        json!({
            "channel": "whatsapp",
            "to": "+15551234567",
            "from": "+15559876543",
            "text": "Your table is ready!",
            "fallbackToSms": false
        }),
        ResponseTemplate::new(201).set_body_json(sent_body()),
    )
    .await;

    let client = create_test_client(&mock_server.uri());
    client
        .messages()
        .send_whatsapp(text_request().with_fallback_to_sms(false))
        .await
        .expect("send");
}

#[tokio::test]
async fn test_fallback_to_sms_omitted_when_unset() {
    let mock_server = setup_mock_server().await;
    mount_send(
        &mock_server,
        json!({
            "channel": "whatsapp",
            "to": "+15551234567",
            "from": "+15559876543",
            "text": "Your table is ready!"
        }),
        ResponseTemplate::new(201).set_body_json(sent_body()),
    )
    .await;

    let client = create_test_client(&mock_server.uri());
    let message = client
        .messages()
        .send_whatsapp(text_request())
        .await
        .expect("send");

    assert!(message.fallback.is_none());
    assert!(!message.fell_back());
}

#[tokio::test]
async fn test_fallback_to_sms_passes_through_send_with_options() {
    let mock_server = setup_mock_server().await;
    mount_send(
        &mock_server,
        json!({
            "channel": "whatsapp",
            "to": "+15551234567",
            "from": "+15559876543",
            "template": { "name": "order_shipped", "language": "en_US" },
            "fallbackToSms": true
        }),
        ResponseTemplate::new(201).set_body_json(sent_body()),
    )
    .await;

    let client = create_test_client(&mock_server.uri());
    client
        .messages()
        .send_whatsapp_with_options(
            SendWhatsAppMessageRequest::new("+15551234567", "+15559876543")
                .with_template(WhatsAppTemplateSendParams::new("order_shipped", "en_US"))
                .with_fallback_to_sms(true),
            sendly::IdempotentRequestOptions::new().idempotency_key("table-ready-1"),
        )
        .await
        .expect("send");
}

// ==================== fallback on the response ====================

#[tokio::test]
async fn test_send_that_fell_back_to_sms() {
    let mock_server = setup_mock_server().await;
    let mut body = sent_body();
    body["status"] = json!("failed");
    body["creditsUsed"] = json!(0);
    body["whatsapp"]["messageId"] = Value::Null;
    body["fallback"] = json!({
        "channel": "sms",
        "status": "sent",
        "reason": "window_closed",
        "messageId": "msg_sms_1",
        "creditsUsed": 2
    });
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(201).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let message = client
        .messages()
        .send_whatsapp(text_request().with_fallback_to_sms(true))
        .await
        .expect("send");

    assert_eq!(message.status, MessageStatus::Failed);
    assert!(message.fell_back());
    let fallback = message.fallback.expect("fallback");
    assert_eq!(fallback.channel, "sms");
    assert_eq!(fallback.status, WhatsAppSmsFallbackStatus::Sent);
    assert!(fallback.is_sent());
    assert!(!fallback.is_pending());
    assert_eq!(
        fallback.reason,
        Some(WhatsAppSmsFallbackReason::WindowClosed)
    );
    assert_eq!(fallback.message_id.as_deref(), Some("msg_sms_1"));
    assert_eq!(fallback.credits_used, Some(2));
    assert!(fallback.error.is_none());
    assert!(fallback.content_dropped.is_none());
}

#[tokio::test]
async fn test_send_with_a_pending_fallback() {
    let mock_server = setup_mock_server().await;
    let mut body = sent_body();
    body["fallback"] = json!({
        "channel": "sms",
        "status": "pending",
        "contentDropped": ["quick_replies", "header_media"]
    });
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(201).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let message = client
        .messages()
        .send_whatsapp(text_request().with_fallback_to_sms(true))
        .await
        .expect("send");

    assert_eq!(message.status, MessageStatus::Sent);
    assert!(!message.fell_back());
    let fallback = message.fallback.expect("fallback");
    assert!(fallback.is_pending());
    assert!(fallback.reason.is_none());
    assert!(fallback.message_id.is_none());
    assert_eq!(
        fallback.content_dropped,
        Some(vec![
            "quick_replies".to_string(),
            "header_media".to_string()
        ])
    );
}

#[tokio::test]
async fn test_fallback_with_values_this_version_does_not_know_still_decodes() {
    let mock_server = setup_mock_server().await;
    let mut body = sent_body();
    body["fallback"] = json!({
        "channel": "sms",
        "status": "deferred",
        "reason": "whatsapp_paused"
    });
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(201).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let message = client
        .messages()
        .send_whatsapp(text_request().with_fallback_to_sms(true))
        .await
        .expect("an unknown fallback status must not fail a charged send");

    assert!(!message.fell_back());
    let fallback = message.fallback.expect("fallback");
    assert_eq!(fallback.status, WhatsAppSmsFallbackStatus::Unknown);
    assert_eq!(fallback.reason, Some(WhatsAppSmsFallbackReason::Unknown));
}

// ==================== messages().get() ====================

#[tokio::test]
async fn test_get_whatsapp_message_with_fallback_and_its_sms() {
    let mock_server = setup_mock_server().await;
    let message = |id: &str, format: &str, extra: Value| {
        let mut body = json!({
            "id": id,
            "to": "+15551234567",
            "from": "+15559876543",
            "text": "Your table is ready!",
            "status": if format == "whatsapp" { "failed" } else { "delivered" },
            "direction": "outbound",
            "error": null,
            "errorCode": null,
            "retryCount": 0,
            "segments": 1,
            "creditsUsed": if format == "whatsapp" { 0 } else { 2 },
            "isSandbox": false,
            "createdAt": "2026-10-17T10:00:00Z",
            "deliveredAt": null,
            "message_format": format,
            "messageFormat": format
        });
        body.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        body
    };
    Mock::given(method("GET"))
        .and(path("/messages/msg_wa_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message(
            "msg_wa_1",
            "whatsapp",
            json!({
                "fallback": {
                    "channel": "sms",
                    "status": "sent",
                    "reason": "whatsapp_failed",
                    "messageId": "msg_sms_1",
                    "creditsUsed": 2
                }
            }),
        )))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/messages/msg_sms_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message(
            "msg_sms_1",
            "sms",
            json!({ "fallbackOf": "msg_wa_1" }),
        )))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let whatsapp = client.messages().get("msg_wa_1").await.expect("get");
    let fallback = whatsapp.fallback.expect("fallback");
    assert_eq!(fallback.status, WhatsAppSmsFallbackStatus::Sent);
    assert_eq!(
        fallback.reason,
        Some(WhatsAppSmsFallbackReason::WhatsappFailed)
    );
    assert!(whatsapp.fallback_of.is_none());

    let sms = client
        .messages()
        .get(fallback.message_id.as_deref().unwrap())
        .await
        .expect("get");
    assert_eq!(sms.fallback_of.as_deref(), Some("msg_wa_1"));
    assert!(sms.fallback.is_none());
}

// ==================== Errors ====================

#[tokio::test]
async fn test_sms_fallback_unavailable() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": "sms_fallback_unavailable",
            "message": "SMS fallback reaches US and Canada numbers only.",
            "suggestion": "Send without fallbackToSms, or make sure this number can send SMS to this recipient first."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .messages()
        .send_whatsapp(text_request().with_fallback_to_sms(true))
        .await
        .unwrap_err();

    assert_eq!(err.code(), Some("sms_fallback_unavailable"));
    assert!(!err.is_retryable());
    match &err {
        Error::Api {
            status_code,
            message,
            ..
        } => {
            assert_eq!(*status_code, 403);
            assert!(message.contains("US and Canada"));
        }
        other => panic!("Expected Api error, got {:?}", other),
    }
    assert!(err.body().unwrap()["suggestion"]
        .as_str()
        .unwrap()
        .contains("fallbackToSms"));
}

#[tokio::test]
async fn test_sms_fallback_not_supported() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "sms_fallback_not_supported",
            "reason": "marketing_template",
            "message": "Marketing templates can't fall back to SMS: SMS marketing needs its own consent. Send without fallbackToSms."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .messages()
        .send_whatsapp(
            SendWhatsAppMessageRequest::new("+15551234567", "+15559876543")
                .with_template(WhatsAppTemplateSendParams::new("spring_sale", "en_US"))
                .with_fallback_to_sms(true),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), Some("sms_fallback_not_supported"));
    assert!(matches!(err, Error::Validation { .. }));
    assert_eq!(err.body().unwrap()["reason"], "marketing_template");
}

#[tokio::test]
async fn test_window_closed_carries_the_fallback_that_could_not_go() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "whatsapp_window_closed",
            "message": "The 24-hour window is closed. Send an approved template instead.",
            "suggestion": "Send an approved template.",
            "id": "msg_wa_2",
            "fallback": {
                "channel": "sms",
                "status": "failed",
                "reason": "window_closed",
                "error": "Not enough credits to send the SMS."
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .messages()
        .send_whatsapp(text_request().with_fallback_to_sms(true))
        .await
        .unwrap_err();

    assert_eq!(err.code(), Some("whatsapp_window_closed"));
    let body = err.body().unwrap();
    assert_eq!(body["id"], "msg_wa_2");
    let fallback: WhatsAppSmsFallback =
        serde_json::from_value(body["fallback"].clone()).expect("fallback decodes");
    assert_eq!(fallback.status, WhatsAppSmsFallbackStatus::Failed);
    assert_eq!(
        fallback.reason,
        Some(WhatsAppSmsFallbackReason::WindowClosed)
    );
    assert_eq!(
        fallback.error.as_deref(),
        Some("Not enough credits to send the SMS.")
    );
}
