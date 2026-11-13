mod common;

use common::{create_test_client, setup_mock_server};
use sendly::{
    CreateWhatsAppSignupRequest, CreateWhatsAppTemplateRequest, Error, IdempotentRequestOptions,
    SendlyConfig, WhatsAppSignupStatus, WhatsAppTemplateCategory,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SENDER: &str = "+15125550142";
const WABA_ID: &str = "104729384756102";

fn retrying_client(base_url: &str) -> sendly::Sendly {
    let config = SendlyConfig::new()
        .base_url(base_url)
        .timeout(std::time::Duration::from_secs(5))
        .max_retries(2);
    sendly::Sendly::with_config(common::TEST_API_KEY, config)
}

async fn idempotency_keys(mock_server: &MockServer) -> Vec<Option<String>> {
    mock_server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .map(|r| {
            r.headers
                .get(&"idempotency-key".into())
                .map(|v| v.last().as_str().to_string())
        })
        .collect()
}

fn template_request() -> CreateWhatsAppTemplateRequest {
    CreateWhatsAppTemplateRequest::new(
        SENDER,
        "order_shipped",
        "en_US",
        WhatsAppTemplateCategory::Utility,
        "Hi {{1}}, your order {{2}} has shipped!",
    )
}

fn template_json() -> serde_json::Value {
    json!({
        "id": "wat_abc123",
        "name": "order_shipped",
        "language": "en_US",
        "category": "UTILITY",
        "status": "PENDING",
        "qualityRating": null,
        "rejectionReason": null,
        "createdAt": "2026-11-13T10:00:00Z",
        "updatedAt": "2026-11-13T10:00:00Z"
    })
}

#[tokio::test]
async fn test_signup_create_with_idempotency_sends_the_key_on_a_facebook_connection() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/signup"))
        .and(header("Idempotency-Key", "connect-15125550142"))
        .and(body_json(json!({ "phoneNumber": SENDER })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "was_abc123",
            "connectUrl": "https://sendly.live/whatsapp/connect/was_abc123",
            "status": "initiated"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let signup = client
        .whatsapp()
        .signup()
        .create_with_idempotency(
            CreateWhatsAppSignupRequest::new(SENDER),
            IdempotentRequestOptions::new().idempotency_key("connect-15125550142"),
        )
        .await
        .unwrap();

    assert_eq!(signup.id, "was_abc123");
    assert_eq!(signup.status, WhatsAppSignupStatus::Initiated);
}

#[tokio::test]
async fn test_signup_create_with_idempotency_by_code_sends_the_key_once_and_never_retries() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/signup"))
        .respond_with(ResponseTemplate::new(503).set_body_json(json!({
            "error": "whatsapp_verification_start_failed",
            "message": "WhatsApp couldn't be reached"
        })))
        .mount(&mock_server)
        .await;

    let client = retrying_client(&mock_server.uri());
    let result = client
        .whatsapp()
        .signup()
        .create_with_idempotency(
            CreateWhatsAppSignupRequest::new(SENDER).business_account_id(WABA_ID),
            IdempotentRequestOptions::new().idempotency_key("add-15125550142"),
        )
        .await;

    assert!(result.is_err());
    assert_eq!(
        idempotency_keys(&mock_server).await,
        vec![Some("add-15125550142".to_string())]
    );
}

#[tokio::test]
async fn test_signup_create_with_options_by_code_still_sends_no_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/signup"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "was_def456",
            "status": "verifying"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    client
        .whatsapp()
        .signup()
        .create_with_options(CreateWhatsAppSignupRequest::new(SENDER).business_account_id(WABA_ID))
        .await
        .unwrap();

    assert_eq!(idempotency_keys(&mock_server).await, vec![None]);
}

#[tokio::test]
async fn test_signup_create_with_idempotency_refuses_an_invalid_key_before_sending() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());
    let result = client
        .whatsapp()
        .signup()
        .create_with_idempotency(
            CreateWhatsAppSignupRequest::new(SENDER).business_account_id(WABA_ID),
            IdempotentRequestOptions::new().idempotency_key("bad\nkey"),
        )
        .await;

    assert!(matches!(result, Err(Error::Validation { .. })));
    assert!(mock_server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn test_templates_create_with_options_sends_the_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/templates"))
        .and(header("Idempotency-Key", "tpl-order-shipped"))
        .respond_with(ResponseTemplate::new(201).set_body_json(template_json()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let template = client
        .whatsapp()
        .templates()
        .create_with_options(
            template_request(),
            IdempotentRequestOptions::new().idempotency_key("tpl-order-shipped"),
        )
        .await
        .unwrap();

    assert_eq!(template.id, "wat_abc123");
}

#[tokio::test]
async fn test_templates_create_sends_an_automatic_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/templates"))
        .respond_with(ResponseTemplate::new(201).set_body_json(template_json()))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    client
        .whatsapp()
        .templates()
        .create(template_request())
        .await
        .unwrap();

    let keys = idempotency_keys(&mock_server).await;
    assert!(keys[0]
        .as_deref()
        .is_some_and(|k| k.starts_with("sendly-rust-retry-")));
}
