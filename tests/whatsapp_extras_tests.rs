mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use sendly::webhooks::Webhooks;
use sendly::{
    CallChannel, CreateWhatsAppSignupRequest, Error, SendWhatsAppMessageRequest,
    UpdateWhatsAppConversationalComponentsRequest, WhatsAppCommand, WhatsAppSenderStatus,
    WhatsAppSignupStatus, WhatsAppVerificationMethod,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

const SENDER: &str = "+15125550142";
const SENDER_PATH: &str = "%2B15125550142";
const SIGNUP_ID: &str = "5b2c1d4e-7f80-4a91-b2c3-d4e5f6a7b8c9";
const WABA_ID: &str = "104729384756102";
const PNG: &[u8] = &[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x01];

fn retrying_client(base_url: &str) -> sendly::Sendly {
    let config = sendly::SendlyConfig::new()
        .base_url(base_url)
        .timeout(std::time::Duration::from_secs(5))
        .max_retries(3);
    sendly::Sendly::with_config(TEST_API_KEY, config)
}

async fn request_count(mock_server: &MockServer) -> usize {
    mock_server
        .received_requests()
        .await
        .expect("request recording is enabled")
        .len()
}

fn profile_json(photo: Option<&str>) -> serde_json::Value {
    json!({
        "phoneNumber": SENDER,
        "displayName": "Acme Bakery",
        "profilePhotoUrl": photo,
        "category": "Retail",
        "about": "Fresh bread, daily.",
        "description": null,
        "email": null,
        "website": null,
        "address": null
    })
}

fn verifying_signup_json() -> serde_json::Value {
    json!({
        "id": SIGNUP_ID,
        "status": "verifying",
        "phoneNumber": SENDER,
        "businessAccountId": WABA_ID,
        "failureReasons": null,
        "verificationMethod": "voice",
        "verificationAttemptsRemaining": 5,
        "updatedAt": "2026-10-01T09:00:00.000Z"
    })
}

struct MultipartFile {
    field: &'static str,
    bytes: &'static [u8],
}

impl Match for MultipartFile {
    fn matches(&self, request: &Request) -> bool {
        let is_multipart = request
            .headers
            .get(&wiremock::http::HeaderName::from_string("content-type".to_string()).unwrap())
            .map(|v| v.last().as_str().starts_with("multipart/form-data"))
            .unwrap_or(false);
        let field = format!("name=\"{}\"", self.field);
        let body = &request.body;
        is_multipart
            && body.windows(field.len()).any(|w| w == field.as_bytes())
            && body.windows(self.bytes.len()).any(|w| w == self.bytes)
    }
}

// ==================== senders().list() new fields ====================

#[tokio::test]
async fn test_senders_list_reads_account_and_calling_fields() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/whatsapp/senders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "senders": [
                {
                    "phoneNumber": "+447700900142",
                    "displayName": "Acme Bakery",
                    "status": "active",
                    "qualityRating": "GREEN",
                    "businessAccountId": WABA_ID,
                    "businessName": "Acme Bakery Ltd",
                    "callingEnabled": true,
                    "outboundCallingAllowed": true,
                    "createdAt": "2026-09-28T10:00:00.000Z"
                },
                {
                    "phoneNumber": SENDER,
                    "displayName": null,
                    "status": "pending",
                    "qualityRating": null,
                    "businessAccountId": null,
                    "businessName": null,
                    "callingEnabled": false,
                    "outboundCallingAllowed": false,
                    "createdAt": "2026-09-30T10:00:00.000Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let senders = client.whatsapp().senders().list().await.unwrap().senders;

    let active = &senders[0];
    assert_eq!(active.business_account_id.as_deref(), Some(WABA_ID));
    assert_eq!(active.business_name.as_deref(), Some("Acme Bakery Ltd"));
    assert_eq!(active.calling_enabled, Some(true));
    assert_eq!(active.outbound_calling_allowed, Some(true));
    let pending = &senders[1];
    assert_eq!(pending.status, WhatsAppSenderStatus::Pending);
    assert!(pending.business_account_id.is_none());
    assert!(pending.business_name.is_none());
    assert_eq!(pending.calling_enabled, Some(false));
    assert_eq!(pending.outbound_calling_allowed, Some(false));
}

#[tokio::test]
async fn test_senders_list_without_the_new_fields_still_decodes() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/whatsapp/senders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "senders": [{
                "phoneNumber": SENDER,
                "displayName": "Acme Bakery",
                "status": "active",
                "qualityRating": "GREEN",
                "createdAt": "2026-09-28T10:00:00.000Z"
            }]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let sender = &client.whatsapp().senders().list().await.unwrap().senders[0];

    assert!(sender.business_account_id.is_none());
    assert!(sender.calling_enabled.is_none());
    assert!(sender.outbound_calling_allowed.is_none());
}

// ==================== senders() profile photo ====================

#[tokio::test]
async fn test_upload_profile_photo_sends_the_file_field() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/whatsapp/senders/{SENDER_PATH}/profile/photo"
        )))
        .and(header(
            "Authorization",
            format!("Bearer {TEST_API_KEY}").as_str(),
        ))
        .and(MultipartFile {
            field: "file",
            bytes: PNG,
        })
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(profile_json(Some("https://pps.example.net/p/acme.png"))),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let profile = client
        .whatsapp()
        .senders()
        .upload_profile_photo_bytes(SENDER, PNG.to_vec(), "logo.png", "image/png")
        .await
        .unwrap();

    assert_eq!(profile.phone_number, SENDER);
    assert_eq!(
        profile.profile_photo_url.as_deref(),
        Some("https://pps.example.net/p/acme.png")
    );
}

#[tokio::test]
async fn test_upload_profile_photo_from_a_path() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/whatsapp/senders/{SENDER_PATH}/profile/photo"
        )))
        .and(MultipartFile {
            field: "file",
            bytes: PNG,
        })
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(profile_json(Some("https://pps.example.net/p/acme.png"))),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let dir = std::env::temp_dir().join(format!("sendly-wa-photo-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("logo.png");
    std::fs::write(&file, PNG).unwrap();

    let client = create_test_client(&mock_server.uri());
    let profile = client
        .whatsapp()
        .senders()
        .upload_profile_photo(SENDER, file.to_str().unwrap())
        .await;
    std::fs::remove_dir_all(&dir).ok();

    assert!(profile.unwrap().profile_photo_url.is_some());
}

#[tokio::test]
async fn test_upload_profile_photo_too_large_is_a_413() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/whatsapp/senders/{SENDER_PATH}/profile/photo"
        )))
        .respond_with(ResponseTemplate::new(413).set_body_json(json!({
            "error": "whatsapp_profile_photo_too_large",
            "message": "The photo must be 5 MB or smaller."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .whatsapp()
        .senders()
        .upload_profile_photo_bytes(SENDER, PNG.to_vec(), "logo.png", "image/png")
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        Error::Api {
            status_code: 413,
            ..
        }
    ));
    assert_eq!(err.code(), Some("whatsapp_profile_photo_too_large"));
}

#[tokio::test]
async fn test_upload_profile_photo_not_jpeg_or_png_is_a_validation_error() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/whatsapp/senders/{SENDER_PATH}/profile/photo"
        )))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "whatsapp_profile_photo_invalid",
            "message": "The photo must be a JPEG or PNG image."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .whatsapp()
        .senders()
        .upload_profile_photo_bytes(SENDER, b"GIF89a".to_vec(), "logo.gif", "image/gif")
        .await
        .unwrap_err();

    assert!(matches!(err, Error::Validation { .. }));
    assert_eq!(err.code(), Some("whatsapp_profile_photo_invalid"));
}

#[tokio::test]
async fn test_upload_profile_photo_is_sent_once_on_a_502() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/whatsapp/senders/{SENDER_PATH}/profile/photo"
        )))
        .respond_with(ResponseTemplate::new(502).set_body_json(json!({
            "error": "whatsapp_profile_update_failed",
            "message": "The photo couldn't be uploaded."
        })))
        .mount(&mock_server)
        .await;

    let client = retrying_client(&mock_server.uri());
    let err = client
        .whatsapp()
        .senders()
        .upload_profile_photo_bytes(SENDER, PNG.to_vec(), "logo.png", "image/png")
        .await
        .unwrap_err();

    assert_eq!(err.code(), Some("whatsapp_profile_update_failed"));
    assert_eq!(request_count(&mock_server).await, 1);
}

#[tokio::test]
async fn test_upload_profile_photo_is_sent_once_on_a_timeout() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/whatsapp/senders/{SENDER_PATH}/profile/photo"
        )))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(profile_json(Some("https://pps.example.net/p/acme.png")))
                .set_delay(std::time::Duration::from_millis(1500)),
        )
        .mount(&mock_server)
        .await;

    let config = sendly::SendlyConfig::new()
        .base_url(mock_server.uri())
        .timeout(std::time::Duration::from_millis(300))
        .max_retries(3);
    let client = sendly::Sendly::with_config(TEST_API_KEY, config);
    let err = client
        .whatsapp()
        .senders()
        .upload_profile_photo_bytes(SENDER, PNG.to_vec(), "logo.png", "image/png")
        .await
        .unwrap_err();

    assert!(matches!(err, Error::Timeout));
    assert_eq!(request_count(&mock_server).await, 1);
}

#[tokio::test]
async fn test_delete_profile_photo() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("DELETE"))
        .and(path(format!(
            "/whatsapp/senders/{SENDER_PATH}/profile/photo"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(profile_json(None)))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let profile = client
        .whatsapp()
        .senders()
        .delete_profile_photo(SENDER)
        .await
        .unwrap();

    assert_eq!(profile.phone_number, SENDER);
    assert!(profile.profile_photo_url.is_none());
}

// ==================== senders() conversational components ====================

#[tokio::test]
async fn test_get_conversational_components() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/whatsapp/senders/{SENDER_PATH}/conversational_components"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "phoneNumber": SENDER,
            "iceBreakers": ["What are your opening hours?", "Can I order a cake?"],
            "commands": [{ "command": "menu", "description": "See today's menu" }]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let components = client
        .whatsapp()
        .senders()
        .get_conversational_components(SENDER)
        .await
        .unwrap();

    assert_eq!(components.phone_number, SENDER);
    assert_eq!(
        components.ice_breakers,
        vec!["What are your opening hours?", "Can I order a cake?"]
    );
    assert_eq!(components.commands.len(), 1);
    assert_eq!(components.commands[0].command, "menu");
    assert_eq!(components.commands[0].description, "See today's menu");
}

#[tokio::test]
async fn test_update_conversational_components_sends_only_the_given_list() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path(format!(
            "/whatsapp/senders/{SENDER_PATH}/conversational_components"
        )))
        .and(body_json(json!({
            "iceBreakers": ["What are your opening hours?"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "phoneNumber": SENDER,
            "iceBreakers": ["What are your opening hours?"],
            "commands": [{ "command": "menu", "description": "See today's menu" }]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let components = client
        .whatsapp()
        .senders()
        .update_conversational_components(
            SENDER,
            UpdateWhatsAppConversationalComponentsRequest::new()
                .ice_breakers(vec!["What are your opening hours?"]),
        )
        .await
        .unwrap();

    assert_eq!(
        components.ice_breakers,
        vec!["What are your opening hours?"]
    );
    assert_eq!(components.commands[0].command, "menu");
}

#[tokio::test]
async fn test_update_conversational_components_empty_list_clears() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path(format!(
            "/whatsapp/senders/{SENDER_PATH}/conversational_components"
        )))
        .and(body_json(json!({
            "iceBreakers": [],
            "commands": [{ "command": "hours", "description": "Opening hours" }]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "phoneNumber": SENDER,
            "iceBreakers": [],
            "commands": [{ "command": "hours", "description": "Opening hours" }]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let components = client
        .whatsapp()
        .senders()
        .update_conversational_components(
            SENDER,
            UpdateWhatsAppConversationalComponentsRequest::new()
                .ice_breakers(Vec::<String>::new())
                .commands(vec![WhatsAppCommand::new("hours", "Opening hours")]),
        )
        .await
        .unwrap();

    assert!(components.ice_breakers.is_empty());
    assert_eq!(components.commands[0].description, "Opening hours");
}

#[tokio::test]
async fn test_update_conversational_components_refusal_is_a_validation_error() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path(format!(
            "/whatsapp/senders/{SENDER_PATH}/conversational_components"
        )))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "invalid_request",
            "message": "At most 4 ice breakers are allowed."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .whatsapp()
        .senders()
        .update_conversational_components(
            SENDER,
            UpdateWhatsAppConversationalComponentsRequest::new()
                .ice_breakers(vec!["a", "b", "c", "d", "e"]),
        )
        .await
        .unwrap_err();

    match err {
        Error::Validation { message, code, .. } => {
            assert_eq!(code.as_deref(), Some("invalid_request"));
            assert!(message.contains("At most 4"));
        }
        other => panic!("expected Error::Validation, got {other:?}"),
    }
}

// ==================== senders().set_calling() ====================

#[tokio::test]
async fn test_set_calling() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path(format!("/whatsapp/senders/{SENDER_PATH}/calling")))
        .and(body_json(json!({ "enabled": true })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "phoneNumber": SENDER,
            "callingEnabled": true,
            "outboundCallingAllowed": false
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let calling = client
        .whatsapp()
        .senders()
        .set_calling(SENDER, true)
        .await
        .unwrap();

    assert_eq!(calling.phone_number, SENDER);
    assert!(calling.calling_enabled);
    assert!(!calling.outbound_calling_allowed);
}

#[tokio::test]
async fn test_set_calling_without_voice_is_a_409() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path(format!("/whatsapp/senders/{SENDER_PATH}/calling")))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "voice_not_enabled",
            "message": "Turn on calls for this number first, in its voice settings, so WhatsApp calls have somewhere to ring."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .whatsapp()
        .senders()
        .set_calling(SENDER, true)
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        Error::Api {
            status_code: 409,
            ..
        }
    ));
    assert_eq!(err.code(), Some("voice_not_enabled"));
}

#[tokio::test]
async fn test_set_calling_refused_by_meta_is_a_validation_error() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path(format!("/whatsapp/senders/{SENDER_PATH}/calling")))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "whatsapp_calling_unavailable",
            "message": "WhatsApp didn't allow calling on this number."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .whatsapp()
        .senders()
        .set_calling(SENDER, true)
        .await
        .unwrap_err();

    assert!(matches!(err, Error::Validation { .. }));
    assert_eq!(err.code(), Some("whatsapp_calling_unavailable"));
}

#[tokio::test]
async fn test_set_calling_off_sends_false() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path(format!("/whatsapp/senders/{SENDER_PATH}/calling")))
        .and(body_json(json!({ "enabled": false })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "phoneNumber": SENDER,
            "callingEnabled": false,
            "outboundCallingAllowed": false
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let calling = client
        .whatsapp()
        .senders()
        .set_calling(SENDER, false)
        .await
        .unwrap();

    assert!(!calling.calling_enabled);
}

// ==================== signup() adding a number by code ====================

#[tokio::test]
async fn test_signup_create_with_options_adds_a_number_by_code() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/signup"))
        .and(body_json(json!({
            "phoneNumber": SENDER,
            "businessAccountId": WABA_ID,
            "verificationMethod": "voice",
            "displayName": "Acme Bakery"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(verifying_signup_json()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let signup = client
        .whatsapp()
        .signup()
        .create_with_options(
            CreateWhatsAppSignupRequest::new(SENDER)
                .business_account_id(WABA_ID)
                .verification_method(WhatsAppVerificationMethod::Voice)
                .display_name("Acme Bakery"),
        )
        .await
        .unwrap();

    assert_eq!(signup.id, SIGNUP_ID);
    assert_eq!(signup.status, WhatsAppSignupStatus::Verifying);
    assert!(signup.connect_url.is_empty());
    assert_eq!(signup.phone_number.as_deref(), Some(SENDER));
    assert_eq!(signup.business_account_id.as_deref(), Some(WABA_ID));
    assert_eq!(
        signup.verification_method,
        Some(WhatsAppVerificationMethod::Voice)
    );
    assert_eq!(signup.verification_attempts_remaining, Some(5));
    assert_eq!(
        signup.updated_at.as_deref(),
        Some("2026-10-01T09:00:00.000Z")
    );
}

#[tokio::test]
async fn test_signup_create_with_options_phone_only_matches_create() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/signup"))
        .and(body_json(json!({ "phoneNumber": SENDER })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": SIGNUP_ID,
            "connectUrl": "https://sendly.live/whatsapp/connect/tok_abc",
            "status": "initiated"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let signup = client
        .whatsapp()
        .signup()
        .create_with_options(CreateWhatsAppSignupRequest::new(SENDER))
        .await
        .unwrap();

    assert_eq!(signup.status, WhatsAppSignupStatus::Initiated);
    assert_eq!(
        signup.connect_url,
        "https://sendly.live/whatsapp/connect/tok_abc"
    );
    assert!(signup.phone_number.is_none());
    assert!(signup.verification_method.is_none());
}

#[tokio::test]
async fn test_signup_create_with_options_unknown_account_is_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/signup"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "whatsapp_business_account_not_found",
            "message": "There's no connected WhatsApp Business account with this id in your workspace."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .whatsapp()
        .signup()
        .create_with_options(CreateWhatsAppSignupRequest::new(SENDER).business_account_id(WABA_ID))
        .await
        .unwrap_err();

    assert!(matches!(err, Error::NotFound { .. }));
    assert_eq!(err.code(), Some("whatsapp_business_account_not_found"));
}

#[tokio::test]
async fn test_signup_create_facebook_flow_while_verifying_is_a_409_with_the_id() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/signup"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "whatsapp_verification_in_progress",
            "message": "This number is already being added to a connected WhatsApp Business account.",
            "id": SIGNUP_ID
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client.whatsapp().signup().create(SENDER).await.unwrap_err();

    assert!(matches!(
        err,
        Error::Api {
            status_code: 409,
            ..
        }
    ));
    assert_eq!(err.code(), Some("whatsapp_verification_in_progress"));
    assert_eq!(err.body().unwrap()["id"], SIGNUP_ID);
}

#[tokio::test]
async fn test_signup_get_while_verifying_reads_the_code() {
    let mock_server = setup_mock_server().await;
    let mut body = verifying_signup_json();
    body["verificationMethod"] = json!("sms");
    body["verificationAttemptsRemaining"] = json!(4);
    body["verificationCode"] = json!("482913");
    Mock::given(method("GET"))
        .and(path(format!("/whatsapp/signup/{SIGNUP_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let signup = client.whatsapp().signup().get(SIGNUP_ID).await.unwrap();

    assert_eq!(signup.status, WhatsAppSignupStatus::Verifying);
    assert_eq!(signup.business_account_id.as_deref(), Some(WABA_ID));
    assert_eq!(
        signup.verification_method,
        Some(WhatsAppVerificationMethod::Sms)
    );
    assert_eq!(signup.verification_attempts_remaining, Some(4));
    assert_eq!(signup.verification_code.as_deref(), Some("482913"));
}

#[tokio::test]
async fn test_signup_get_while_verifying_before_the_code_arrives() {
    let mock_server = setup_mock_server().await;
    let mut body = verifying_signup_json();
    body["verificationCode"] = json!(null);
    Mock::given(method("GET"))
        .and(path(format!("/whatsapp/signup/{SIGNUP_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let signup = client.whatsapp().signup().get(SIGNUP_ID).await.unwrap();

    assert!(signup.verification_code.is_none());
    assert_eq!(signup.verification_attempts_remaining, Some(5));
}

#[tokio::test]
async fn test_signup_get_failed_by_verification() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/whatsapp/signup/{SIGNUP_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": SIGNUP_ID,
            "status": "failed",
            "phoneNumber": SENDER,
            "businessAccountId": null,
            "failureReasons": ["verification_failed"],
            "updatedAt": "2026-10-01T09:10:00.000Z"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let signup = client.whatsapp().signup().get(SIGNUP_ID).await.unwrap();

    assert_eq!(signup.status, WhatsAppSignupStatus::Failed);
    assert_eq!(
        signup.failure_reasons,
        Some(vec!["verification_failed".to_string()])
    );
    assert!(signup.verification_method.is_none());
    assert!(signup.verification_attempts_remaining.is_none());
    assert!(signup.verification_code.is_none());
}

#[tokio::test]
async fn test_signup_verify() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/whatsapp/signup/{SIGNUP_ID}/verify")))
        .and(body_json(json!({ "code": "482913" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": SIGNUP_ID,
            "status": "active",
            "phoneNumber": SENDER,
            "businessAccountId": WABA_ID,
            "failureReasons": null,
            "updatedAt": "2026-10-01T09:02:00.000Z"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let signup = client
        .whatsapp()
        .signup()
        .verify(SIGNUP_ID, "482913")
        .await
        .unwrap();

    assert_eq!(signup.status, WhatsAppSignupStatus::Active);
    assert_eq!(signup.business_account_id.as_deref(), Some(WABA_ID));
}

#[tokio::test]
async fn test_signup_verify_wrong_code_reports_attempts_remaining() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/whatsapp/signup/{SIGNUP_ID}/verify")))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "whatsapp_verification_code_invalid",
            "message": "That code wasn't accepted. Check it, or request a new one.",
            "attemptsRemaining": 3
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .whatsapp()
        .signup()
        .verify(SIGNUP_ID, "111111")
        .await
        .unwrap_err();

    assert!(matches!(err, Error::Validation { .. }));
    assert_eq!(err.code(), Some("whatsapp_verification_code_invalid"));
    assert_eq!(err.remaining_attempts(), Some(3));
}

#[tokio::test]
async fn test_signup_verify_too_many_wrong_codes_is_a_409() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/whatsapp/signup/{SIGNUP_ID}/verify")))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "whatsapp_verification_failed",
            "message": "Too many wrong codes. Any setup fee is refunded automatically. Start again to retry."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .whatsapp()
        .signup()
        .verify(SIGNUP_ID, "111111")
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        Error::Api {
            status_code: 409,
            ..
        }
    ));
    assert_eq!(err.code(), Some("whatsapp_verification_failed"));
}

#[tokio::test]
async fn test_signup_verify_is_sent_once_on_a_502() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/whatsapp/signup/{SIGNUP_ID}/verify")))
        .respond_with(ResponseTemplate::new(502).set_body_json(json!({
            "error": "whatsapp_activation_pending",
            "message": "WhatsApp accepted the code, but we couldn't finish connecting the number. Our team has been alerted; check back shortly."
        })))
        .mount(&mock_server)
        .await;

    let client = retrying_client(&mock_server.uri());
    let err = client
        .whatsapp()
        .signup()
        .verify(SIGNUP_ID, "482913")
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        Error::Api {
            status_code: 502,
            ..
        }
    ));
    assert_eq!(err.code(), Some("whatsapp_activation_pending"));
    assert_eq!(request_count(&mock_server).await, 1);
}

#[tokio::test]
async fn test_signup_verify_is_sent_once_on_a_timeout() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/whatsapp/signup/{SIGNUP_ID}/verify")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(verifying_signup_json())
                .set_delay(std::time::Duration::from_millis(1500)),
        )
        .mount(&mock_server)
        .await;

    let config = sendly::SendlyConfig::new()
        .base_url(mock_server.uri())
        .timeout(std::time::Duration::from_millis(300))
        .max_retries(3);
    let client = sendly::Sendly::with_config(TEST_API_KEY, config);
    let err = client
        .whatsapp()
        .signup()
        .verify(SIGNUP_ID, "482913")
        .await
        .unwrap_err();

    assert!(matches!(err, Error::Timeout));
    assert_eq!(request_count(&mock_server).await, 1);
}

#[tokio::test]
async fn test_signup_create_with_options_is_sent_once_on_a_502() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/signup"))
        .respond_with(ResponseTemplate::new(502).set_body_json(json!({
            "error": "whatsapp_verification_start_failed",
            "message": "WhatsApp couldn't start verifying this number. Any setup fee is refunded automatically. Please try again shortly."
        })))
        .mount(&mock_server)
        .await;

    let client = retrying_client(&mock_server.uri());
    let err = client
        .whatsapp()
        .signup()
        .create_with_options(CreateWhatsAppSignupRequest::new(SENDER).business_account_id(WABA_ID))
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        Error::Api {
            status_code: 502,
            ..
        }
    ));
    assert_eq!(err.code(), Some("whatsapp_verification_start_failed"));
    assert_eq!(request_count(&mock_server).await, 1);
}

#[tokio::test]
async fn test_signup_create_with_options_by_code_is_sent_once_on_a_timeout() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/whatsapp/signup"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(verifying_signup_json())
                .set_delay(std::time::Duration::from_millis(1500)),
        )
        .mount(&mock_server)
        .await;

    let config = sendly::SendlyConfig::new()
        .base_url(mock_server.uri())
        .timeout(std::time::Duration::from_millis(300))
        .max_retries(3);
    let client = sendly::Sendly::with_config(TEST_API_KEY, config);
    let err = client
        .whatsapp()
        .signup()
        .create_with_options(CreateWhatsAppSignupRequest::new(SENDER).business_account_id(WABA_ID))
        .await
        .unwrap_err();

    assert!(matches!(err, Error::Timeout));
    assert_eq!(request_count(&mock_server).await, 1);
}

#[tokio::test]
async fn test_signup_create_with_options_refuses_a_blank_business_account_id() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    for blank in ["", "   "] {
        let err = client
            .whatsapp()
            .signup()
            .create_with_options(CreateWhatsAppSignupRequest::new(SENDER).business_account_id(blank))
            .await
            .unwrap_err();

        assert!(matches!(err, Error::Validation { .. }), "{err:?}");
    }
    assert!(mock_server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn test_signup_verify_unknown_id_is_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/whatsapp/signup/{SIGNUP_ID}/verify")))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(json!({ "error": "signup_not_found" })),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .whatsapp()
        .signup()
        .verify(SIGNUP_ID, "482913")
        .await
        .unwrap_err();

    assert!(matches!(err, Error::NotFound { .. }));
}

#[tokio::test]
async fn test_signup_resend_without_a_method_sends_an_empty_object() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/whatsapp/signup/{SIGNUP_ID}/resend")))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(verifying_signup_json()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let signup = client
        .whatsapp()
        .signup()
        .resend(SIGNUP_ID, None)
        .await
        .unwrap();

    assert_eq!(signup.status, WhatsAppSignupStatus::Verifying);
}

#[tokio::test]
async fn test_signup_resend_by_voice() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/whatsapp/signup/{SIGNUP_ID}/resend")))
        .and(body_json(json!({ "verificationMethod": "voice" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(verifying_signup_json()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let signup = client
        .whatsapp()
        .signup()
        .resend(SIGNUP_ID, Some(WhatsAppVerificationMethod::Voice))
        .await
        .unwrap();

    assert_eq!(
        signup.verification_method,
        Some(WhatsAppVerificationMethod::Voice)
    );
}

#[tokio::test]
async fn test_signup_resend_too_soon_says_how_long_to_wait() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/whatsapp/signup/{SIGNUP_ID}/resend")))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({
                    "error": "whatsapp_verification_resend_too_soon",
                    "message": "Wait 30 seconds before requesting another code.",
                    "retryAfter": 30
                }))
                .insert_header("Retry-After", "30"),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .whatsapp()
        .signup()
        .resend(SIGNUP_ID, None)
        .await
        .unwrap_err();

    assert!(matches!(err, Error::RateLimit { .. }));
    assert_eq!(err.code(), Some("whatsapp_verification_resend_too_soon"));
    assert_eq!(err.retry_after(), Some(30));
    assert!(err.is_retryable());
}

#[test]
fn test_verification_method_tolerates_unknown_values() {
    let method: WhatsAppVerificationMethod = serde_json::from_value(json!("flash_call")).unwrap();
    assert_eq!(method, WhatsAppVerificationMethod::Unknown);
}

// ==================== Calls and call webhooks carry a channel ====================

fn call_json(channel: Option<&str>) -> serde_json::Value {
    let mut call = json!({
        "id": "6f1c2d3e-4a5b-4c6d-8e9f-0a1b2c3d4e5f",
        "object": "call",
        "kind": "pstn",
        "direction": "inbound",
        "status": "completed",
        "handledBy": "dashboard",
        "agentId": null,
        "from": "+14155550177",
        "to": SENDER,
        "callerName": null,
        "calleeName": null,
        "startedAt": "2026-10-01T09:00:00.000Z",
        "answeredAt": "2026-10-01T09:00:05.000Z",
        "endedAt": "2026-10-01T09:01:05.000Z",
        "durationSecs": 60,
        "creditsCharged": 2,
        "billing": "settled",
        "hangupClass": "normal",
        "recordingStatus": null,
        "metadata": {}
    });
    if let Some(channel) = channel {
        call["channel"] = json!(channel);
    }
    call
}

#[tokio::test]
async fn test_call_reads_its_channel() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/calls/c_whatsapp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(call_json(Some("whatsapp"))))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/calls/c_new"))
        .respond_with(ResponseTemplate::new(200).set_body_json(call_json(Some("carrier_pigeon"))))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/calls/c_old"))
        .respond_with(ResponseTemplate::new(200).set_body_json(call_json(None)))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let whatsapp = client.calls().get("c_whatsapp").await.unwrap();
    let unknown = client.calls().get("c_new").await.unwrap();
    let old = client.calls().get("c_old").await.unwrap();

    assert_eq!(whatsapp.channel, Some(CallChannel::WhatsApp));
    assert_eq!(unknown.channel, Some(CallChannel::Unknown));
    assert!(old.channel.is_none());
}

#[test]
fn test_call_channel_values() {
    for (wire, channel) in [
        ("phone", CallChannel::Phone),
        ("whatsapp", CallChannel::WhatsApp),
        ("browser", CallChannel::Browser),
    ] {
        let decoded: CallChannel = serde_json::from_value(json!(wire)).unwrap();
        assert_eq!(decoded, channel);
        assert_eq!(channel.as_str(), wire);
        assert_eq!(channel.to_string(), wire);
    }
}

#[test]
fn test_call_webhook_payload_channel_decodes_as_call_channel() {
    let secret = "whsec_test";
    let payload = json!({
        "id": "evt_c1",
        "type": "call.completed",
        "api_version": "2024-01",
        "created": 1,
        "livemode": true,
        "data": { "object": {
            "id": "call_1",
            "object": "call",
            "kind": "pstn",
            "channel": "whatsapp",
            "direction": "inbound",
            "status": "completed",
            "handled_by": "dashboard",
            "from": "+14155550177",
            "to": SENDER
        }}
    })
    .to_string();
    let signature = {
        use hmac::{Hmac, Mac};
        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
    };

    let event = Webhooks::parse_event(&payload, &signature, secret, None).unwrap();

    #[derive(serde::Deserialize)]
    struct CallEvent {
        channel: CallChannel,
    }
    let call: CallEvent = event.object_as().unwrap();
    assert_eq!(call.channel, CallChannel::WhatsApp);
}

// ==================== WhatsApp send outcomes ====================

#[tokio::test]
async fn test_send_whatsapp_unconfirmed_is_a_409_and_is_not_retried() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "whatsapp_send_unconfirmed",
            "errorCode": "E024",
            "message": "We couldn't confirm whether WhatsApp accepted this message. It has been marked failed and refunded, but it may still be delivered. Check before sending it again, or it could arrive twice."
        })))
        .mount(&mock_server)
        .await;

    let client = retrying_client(&mock_server.uri());
    let err = client
        .messages()
        .send_whatsapp(
            SendWhatsAppMessageRequest::new("+14155550123", SENDER)
                .with_text("Your order is ready"),
        )
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        Error::Api {
            status_code: 409,
            ..
        }
    ));
    assert_eq!(err.code(), Some("whatsapp_send_unconfirmed"));
    assert!(!err.is_retryable());
    assert_eq!(request_count(&mock_server).await, 1);
}

#[tokio::test]
async fn test_send_whatsapp_never_reached_is_a_502_and_is_not_retried() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(502).set_body_json(json!({
            "error": "whatsapp_send_failed",
            "errorCode": "E028",
            "message": "The message couldn't be delivered."
        })))
        .mount(&mock_server)
        .await;

    let client = retrying_client(&mock_server.uri());
    let err = client
        .messages()
        .send_whatsapp(
            SendWhatsAppMessageRequest::new("+14155550123", SENDER)
                .with_text("Your order is ready"),
        )
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        Error::Api {
            status_code: 502,
            ..
        }
    ));
    assert_eq!(err.code(), Some("whatsapp_send_failed"));
    assert_eq!(request_count(&mock_server).await, 1);
}

// ==================== A reused connection that drops after the request ====================

async fn read_request(socket: &mut tokio::net::TcpStream) -> Option<String> {
    use tokio::io::AsyncReadExt;

    let mut received = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        if let Some(head_end) = received.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&received[..head_end]).to_lowercase();
            let length: usize = head
                .lines()
                .find_map(|line| line.strip_prefix("content-length:"))
                .and_then(|value| value.trim().parse().ok())
                .unwrap_or(0);
            if received.len() >= head_end + 4 + length {
                return Some(String::from_utf8_lossy(&received).into_owned());
            }
        }
        let n = socket.read(&mut buf).await.ok()?;
        if n == 0 {
            return None;
        }
        received.extend_from_slice(&buf[..n]);
    }
}

type RecordedRequests = std::sync::Arc<std::sync::Mutex<Vec<(usize, String)>>>;

async fn keep_alive_server_that_drops_each_post() -> (String, RecordedRequests) {
    use tokio::io::AsyncWriteExt;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    let requests = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let recorded = requests.clone();
    tokio::spawn(async move {
        for connection in 1.. {
            let Ok((mut socket, _)) = listener.accept().await else {
                return;
            };
            let recorded = recorded.clone();
            tokio::spawn(async move {
                while let Some(request) = read_request(&mut socket).await {
                    let is_get = request.starts_with("GET ");
                    recorded.lock().unwrap().push((connection, request));
                    if !is_get {
                        return;
                    }
                    let body = verifying_signup_json().to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n{body}",
                        body.len()
                    );
                    if socket.write_all(response.as_bytes()).await.is_err() {
                        return;
                    }
                }
            });
        }
    });
    (base_url, requests)
}

fn assert_sent_once_on_the_reused_connection(requests: &RecordedRequests, body: &str) {
    let requests = requests.lock().unwrap();
    let methods: Vec<(usize, &str)> = requests
        .iter()
        .map(|(connection, request)| (*connection, request.split(' ').next().unwrap_or("")))
        .collect();
    assert_eq!(methods, [(1, "GET"), (1, "POST")]);
    assert!(requests[1].1.contains(body), "{}", requests[1].1);
}

#[tokio::test]
async fn test_signup_verify_is_not_resent_when_a_reused_connection_drops() {
    let (base_url, requests) = keep_alive_server_that_drops_each_post().await;
    let client = retrying_client(&base_url);
    client.whatsapp().signup().get(SIGNUP_ID).await.unwrap();

    let err = client
        .whatsapp()
        .signup()
        .verify(SIGNUP_ID, "482913")
        .await
        .unwrap_err();

    assert!(matches!(err, Error::Http(_)), "{err:?}");
    assert_sent_once_on_the_reused_connection(&requests, r#""code":"482913""#);
}

#[tokio::test]
async fn test_signup_create_with_options_by_code_is_not_resent_when_a_reused_connection_drops() {
    let (base_url, requests) = keep_alive_server_that_drops_each_post().await;
    let client = retrying_client(&base_url);
    client.whatsapp().signup().get(SIGNUP_ID).await.unwrap();

    let err = client
        .whatsapp()
        .signup()
        .create_with_options(CreateWhatsAppSignupRequest::new(SENDER).business_account_id(WABA_ID))
        .await
        .unwrap_err();

    assert!(matches!(err, Error::Http(_)), "{err:?}");
    assert_sent_once_on_the_reused_connection(
        &requests,
        &format!(r#""businessAccountId":"{WABA_ID}""#),
    );
}

#[tokio::test]
async fn test_upload_profile_photo_is_not_resent_when_a_reused_connection_drops() {
    let (base_url, requests) = keep_alive_server_that_drops_each_post().await;
    let client = retrying_client(&base_url);
    client.whatsapp().signup().get(SIGNUP_ID).await.unwrap();

    let err = client
        .whatsapp()
        .senders()
        .upload_profile_photo_bytes(SENDER, PNG.to_vec(), "logo.png", "image/png")
        .await
        .unwrap_err();

    assert!(matches!(err, Error::Http(_)), "{err:?}");
    assert_sent_once_on_the_reused_connection(&requests, r#"name="file""#);
}
