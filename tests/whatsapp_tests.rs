mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use sendly::{
    CreateWhatsAppTemplateRequest, Error, UpdateWhatsAppSenderProfileRequest,
    UpdateWhatsAppTemplateRequest, WhatsAppSenderStatus, WhatsAppSignupStatus,
    WhatsAppTemplateButton, WhatsAppTemplateButtonType, WhatsAppTemplateCategory,
    WhatsAppTemplateStatus,
};
use serde_json::json;
use std::collections::HashMap;
use wiremock::matchers::{body_json, body_partial_json, header, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

// ==================== signup().create() Tests ====================

#[tokio::test]
async fn test_signup_create_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/signup"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .and(body_json(json!({ "phoneNumber": "+15559876543" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "was_abc123",
            "connectUrl": "https://sendly.live/whatsapp/connect/was_abc123",
            "status": "initiated"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.whatsapp().signup().create("+15559876543").await;

    assert!(result.is_ok());
    let signup = result.unwrap();
    assert_eq!(signup.id, "was_abc123");
    assert_eq!(
        signup.connect_url,
        "https://sendly.live/whatsapp/connect/was_abc123"
    );
    assert_eq!(signup.status, WhatsAppSignupStatus::Initiated);
}

#[tokio::test]
async fn test_signup_create_number_not_eligible() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/signup"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "number_not_eligible",
            "message": "The number must be an active number in your workspace"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.whatsapp().signup().create("+15559876543").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("active number")),
        _ => panic!("Expected Validation error"),
    }
}

// ==================== signup().get() Tests ====================

#[tokio::test]
async fn test_signup_get_active() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/whatsapp/signup/was_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "was_abc123",
            "status": "active",
            "phoneNumber": "+15559876543",
            "businessAccountId": "1234567890",
            "failureReasons": null,
            "updatedAt": "2026-07-30T10:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.whatsapp().signup().get("was_abc123").await;

    assert!(result.is_ok());
    let signup = result.unwrap();
    assert_eq!(signup.id, "was_abc123");
    assert_eq!(signup.status, WhatsAppSignupStatus::Active);
    assert_eq!(signup.phone_number, "+15559876543");
    assert_eq!(signup.business_account_id.as_deref(), Some("1234567890"));
    assert!(signup.failure_reasons.is_none());
    assert_eq!(signup.updated_at, "2026-07-30T10:00:00Z");
}

#[tokio::test]
async fn test_signup_get_failed_with_reasons() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/whatsapp/signup/was_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "was_abc123",
            "status": "failed",
            "phoneNumber": "+15559876543",
            "businessAccountId": null,
            "failureReasons": ["The business account could not be verified"],
            "updatedAt": "2026-07-30T10:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.whatsapp().signup().get("was_abc123").await;

    assert!(result.is_ok());
    let signup = result.unwrap();
    assert_eq!(signup.status, WhatsAppSignupStatus::Failed);
    assert!(signup.business_account_id.is_none());
    assert_eq!(
        signup.failure_reasons,
        Some(vec![
            "The business account could not be verified".to_string()
        ])
    );
}

#[tokio::test]
async fn test_signup_get_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/whatsapp/signup/was_missing"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "not_found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.whatsapp().signup().get("was_missing").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NotFound { .. }));
}

// ==================== senders().list() Tests ====================

#[tokio::test]
async fn test_senders_list_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/whatsapp/senders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "senders": [
                {
                    "phoneNumber": "+15559876543",
                    "displayName": "Acme Support",
                    "status": "active",
                    "qualityRating": "GREEN",
                    "createdAt": "2026-07-28T10:00:00Z"
                },
                {
                    "phoneNumber": "+15551112222",
                    "displayName": null,
                    "status": "pending",
                    "qualityRating": null,
                    "createdAt": "2026-07-30T10:00:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.whatsapp().senders().list().await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.senders.len(), 2);
    let s = &response.senders[0];
    assert_eq!(s.phone_number, "+15559876543");
    assert_eq!(s.display_name.as_deref(), Some("Acme Support"));
    assert_eq!(s.status, WhatsAppSenderStatus::Active);
    assert_eq!(s.quality_rating.as_deref(), Some("GREEN"));
    assert_eq!(s.created_at, "2026-07-28T10:00:00Z");
    let p = &response.senders[1];
    assert!(p.display_name.is_none());
    assert_eq!(p.status, WhatsAppSenderStatus::Pending);
    assert!(p.quality_rating.is_none());
}

#[tokio::test]
async fn test_senders_list_empty() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/whatsapp/senders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "senders": []
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.whatsapp().senders().list().await;

    assert!(result.is_ok());
    assert!(result.unwrap().senders.is_empty());
}

// ==================== senders().get_profile() Tests ====================

#[tokio::test]
async fn test_senders_get_profile_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/whatsapp/senders/%2B15559876543/profile"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "phoneNumber": "+15559876543",
            "displayName": "Acme Inc",
            "profilePhotoUrl": "https://example.com/logo.png",
            "category": "Retail",
            "about": "Family-run since 1998",
            "description": "Order updates and support over WhatsApp.",
            "email": "support@example.com",
            "website": "https://example.com",
            "address": null
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .whatsapp()
        .senders()
        .get_profile("+15559876543")
        .await;

    assert!(result.is_ok());
    let profile = result.unwrap();
    assert_eq!(profile.phone_number, "+15559876543");
    assert_eq!(profile.display_name.as_deref(), Some("Acme Inc"));
    assert_eq!(
        profile.profile_photo_url.as_deref(),
        Some("https://example.com/logo.png")
    );
    assert_eq!(profile.category.as_deref(), Some("Retail"));
    assert_eq!(profile.about.as_deref(), Some("Family-run since 1998"));
    assert_eq!(profile.email.as_deref(), Some("support@example.com"));
    assert!(profile.address.is_none());
}

#[tokio::test]
async fn test_senders_get_profile_not_connected() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/whatsapp/senders/%2B15559876543/profile"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "whatsapp_sender_not_connected",
            "message": "This number isn't connected to WhatsApp yet."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .whatsapp()
        .senders()
        .get_profile("+15559876543")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::NotFound { message } => assert!(message.contains("isn't connected")),
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

// ==================== senders().update_profile() Tests ====================

#[tokio::test]
async fn test_senders_update_profile_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/whatsapp/senders/%2B15559876543/profile"))
        .and(body_json(json!({
            "about": "Family-run since 1998",
            "website": "https://example.com"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "phoneNumber": "+15559876543",
            "displayName": "Acme Inc",
            "profilePhotoUrl": null,
            "category": null,
            "about": "Family-run since 1998",
            "description": null,
            "email": null,
            "website": "https://example.com",
            "address": null
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .whatsapp()
        .senders()
        .update_profile(
            "+15559876543",
            UpdateWhatsAppSenderProfileRequest::new()
                .about("Family-run since 1998")
                .website("https://example.com"),
        )
        .await;

    assert!(result.is_ok());
    let profile = result.unwrap();
    assert_eq!(profile.about.as_deref(), Some("Family-run since 1998"));
    assert_eq!(profile.website.as_deref(), Some("https://example.com"));
    assert!(profile.description.is_none());
}

#[tokio::test]
async fn test_senders_update_profile_field_too_long() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/whatsapp/senders/%2B15559876543/profile"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "invalid_request",
            "message": "Field 'about' must be at most 139 characters."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .whatsapp()
        .senders()
        .update_profile(
            "+15559876543",
            UpdateWhatsAppSenderProfileRequest::new().about("x".repeat(200)),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("139 characters")),
        other => panic!("Expected Validation error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_senders_update_profile_requires_live_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/whatsapp/senders/%2B15559876543/profile"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": "whatsapp_requires_live_key",
            "message": "WhatsApp requires a live API key. Test keys cannot update sender profiles."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .whatsapp()
        .senders()
        .update_profile(
            "+15559876543",
            UpdateWhatsAppSenderProfileRequest::new().display_name("Acme Inc"),
        )
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

// ==================== templates().list() Tests ====================

#[tokio::test]
async fn test_templates_list_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/whatsapp/templates"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "templates": [
                {
                    "id": "wat_abc123",
                    "name": "order_shipped",
                    "language": "en_US",
                    "category": "UTILITY",
                    "status": "APPROVED",
                    "qualityRating": "GREEN",
                    "rejectionReason": null,
                    "createdAt": "2026-07-28T10:00:00Z",
                    "updatedAt": "2026-07-29T10:00:00Z"
                },
                {
                    "id": "wat_def456",
                    "name": "spring_sale",
                    "language": "en_US",
                    "category": "MARKETING",
                    "status": "REJECTED",
                    "qualityRating": null,
                    "rejectionReason": "INVALID_FORMAT",
                    "createdAt": "2026-07-28T10:00:00Z",
                    "updatedAt": "2026-07-29T10:00:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.whatsapp().templates().list().await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.templates.len(), 2);
    let t = &response.templates[0];
    assert_eq!(t.id, "wat_abc123");
    assert_eq!(t.name, "order_shipped");
    assert_eq!(t.language, "en_US");
    assert_eq!(t.category, WhatsAppTemplateCategory::Utility);
    assert_eq!(t.status, WhatsAppTemplateStatus::Approved);
    assert_eq!(t.quality_rating.as_deref(), Some("GREEN"));
    assert!(t.rejection_reason.is_none());
    let r = &response.templates[1];
    assert_eq!(r.category, WhatsAppTemplateCategory::Marketing);
    assert_eq!(r.status, WhatsAppTemplateStatus::Rejected);
    assert_eq!(r.rejection_reason.as_deref(), Some("INVALID_FORMAT"));
}

// ==================== templates().create() Tests ====================

#[tokio::test]
async fn test_templates_create_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/templates"))
        .and(body_partial_json(json!({
            "sender": "+15559876543",
            "name": "order_shipped",
            "language": "en_US",
            "category": "UTILITY",
            "body": "Hi {{1}}, your order {{2}} has shipped!",
            "footer": "Reply STOP to opt out",
            "buttons": [
                {
                    "type": "url",
                    "text": "Track order",
                    "url": "https://acme.example/orders/{{1}}",
                    "example": ["4821"]
                },
                {
                    "type": "quick_reply",
                    "text": "Stop promotions"
                }
            ],
            "examples": { "1": "Sam", "2": "#4821" }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "wat_abc123",
            "name": "order_shipped",
            "language": "en_US",
            "category": "UTILITY",
            "status": "PENDING",
            "qualityRating": null,
            "rejectionReason": null,
            "createdAt": "2026-07-30T10:00:00Z",
            "updatedAt": "2026-07-30T10:00:00Z",
            "warnings": ["The sender's display name has not been approved yet"]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let mut examples = HashMap::new();
    examples.insert("1".to_string(), "Sam".to_string());
    examples.insert("2".to_string(), "#4821".to_string());
    let result = client
        .whatsapp()
        .templates()
        .create(
            CreateWhatsAppTemplateRequest::new(
                "+15559876543",
                "order_shipped",
                "en_US",
                WhatsAppTemplateCategory::Utility,
                "Hi {{1}}, your order {{2}} has shipped!",
            )
            .footer("Reply STOP to opt out")
            .buttons(vec![
                WhatsAppTemplateButton {
                    button_type: WhatsAppTemplateButtonType::Url,
                    text: "Track order".to_string(),
                    url: Some("https://acme.example/orders/{{1}}".to_string()),
                    example: Some(vec!["4821".to_string()]),
                },
                WhatsAppTemplateButton {
                    button_type: WhatsAppTemplateButtonType::QuickReply,
                    text: "Stop promotions".to_string(),
                    url: None,
                    example: None,
                },
            ])
            .examples(examples),
        )
        .await;

    assert!(result.is_ok());
    let template = result.unwrap();
    assert_eq!(template.id, "wat_abc123");
    assert_eq!(template.status, WhatsAppTemplateStatus::Pending);
    assert_eq!(
        template.warnings,
        Some(vec![
            "The sender's display name has not been approved yet".to_string()
        ])
    );
}

#[tokio::test]
async fn test_templates_create_minimal_body_omits_unset_fields() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/templates"))
        .and(body_json(json!({
            "sender": "+15559876543",
            "name": "table_ready",
            "language": "en_US",
            "category": "UTILITY",
            "body": "Your table is ready!"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "wat_abc123",
            "name": "table_ready",
            "language": "en_US",
            "category": "UTILITY",
            "status": "PENDING",
            "qualityRating": null,
            "rejectionReason": null,
            "createdAt": "2026-07-30T10:00:00Z",
            "updatedAt": "2026-07-30T10:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .whatsapp()
        .templates()
        .create(CreateWhatsAppTemplateRequest::new(
            "+15559876543",
            "table_ready",
            "en_US",
            WhatsAppTemplateCategory::Utility,
            "Your table is ready!",
        ))
        .await;

    assert!(result.is_ok());
    let template = result.unwrap();
    assert_eq!(template.status, WhatsAppTemplateStatus::Pending);
    assert!(template.warnings.is_none());
}

#[tokio::test]
async fn test_templates_create_missing_examples() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/templates"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "missing_examples",
            "message": "Every body placeholder needs an example value"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .whatsapp()
        .templates()
        .create(CreateWhatsAppTemplateRequest::new(
            "+15559876543",
            "order_shipped",
            "en_US",
            WhatsAppTemplateCategory::Utility,
            "Hi {{1}}, your order {{2}} has shipped!",
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("example value")),
        _ => panic!("Expected Validation error"),
    }
}

// ==================== templates().update() Tests ====================

#[tokio::test]
async fn test_templates_update_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/whatsapp/templates/wat_abc123"))
        .and(body_json(json!({
            "body": "Hi {{1}}, your order {{2}} is on its way!",
            "examples": { "1": "Sam", "2": "#4821" }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "wat_abc123",
            "name": "order_shipped",
            "language": "en_US",
            "category": "UTILITY",
            "status": "PENDING",
            "qualityRating": null,
            "rejectionReason": null,
            "createdAt": "2026-07-28T10:00:00Z",
            "updatedAt": "2026-07-30T10:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let mut examples = HashMap::new();
    examples.insert("1".to_string(), "Sam".to_string());
    examples.insert("2".to_string(), "#4821".to_string());
    let result = client
        .whatsapp()
        .templates()
        .update(
            "wat_abc123",
            UpdateWhatsAppTemplateRequest::new()
                .body("Hi {{1}}, your order {{2}} is on its way!")
                .examples(examples),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, WhatsAppTemplateStatus::Pending);
}

#[tokio::test]
async fn test_templates_update_not_editable() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/whatsapp/templates/wat_abc123"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "template_not_editable",
            "message": "Only APPROVED or REJECTED templates can be edited"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .whatsapp()
        .templates()
        .update(
            "wat_abc123",
            UpdateWhatsAppTemplateRequest::new().body("Updated body"),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("APPROVED or REJECTED")),
        _ => panic!("Expected Validation error"),
    }
}

// ==================== templates().delete() Tests ====================

#[tokio::test]
async fn test_templates_delete_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("DELETE"))
        .and(path("/whatsapp/templates/wat_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "wat_abc123",
            "deleted": true
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.whatsapp().templates().delete("wat_abc123").await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.id, "wat_abc123");
    assert!(response.deleted);
}

#[tokio::test]
async fn test_templates_delete_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("DELETE"))
        .and(path("/whatsapp/templates/wat_missing"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "not_found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.whatsapp().templates().delete("wat_missing").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NotFound { .. }));
}

// ==================== window() Tests ====================

#[tokio::test]
async fn test_window_open() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/whatsapp/window"))
        .and(query_param("from", "+15559876543"))
        .and(query_param("to", "+15551234567"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "open": true,
            "expiresAt": "2026-07-31T09:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .whatsapp()
        .window("+15559876543", "+15551234567")
        .await;

    assert!(result.is_ok());
    let window = result.unwrap();
    assert!(window.open);
    assert_eq!(window.expires_at.as_deref(), Some("2026-07-31T09:00:00Z"));
}

#[tokio::test]
async fn test_window_closed() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/whatsapp/window"))
        .and(query_param("from", "+15559876543"))
        .and(query_param("to", "+15551234567"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "open": false,
            "expiresAt": null
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .whatsapp()
        .window("+15559876543", "+15551234567")
        .await;

    assert!(result.is_ok());
    let window = result.unwrap();
    assert!(!window.open);
    assert!(window.expires_at.is_none());
}
