mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use regex::Regex;
use sendly::{
    CreateRcsAgentRequest, Error, IdempotentRequestOptions, RcsAgentBasicsInput,
    RcsAgentWebsiteContact, RcsBrandAddressInput, RcsBrandContactInput, RcsBrandInput, RcsCampaign,
    RcsConsentSettings, RcsCustomerStage, RcsInteraction, RcsOptInMethod, RcsRequestLaunchRequest,
    RcsReviewStatus, RcsTestDeviceInput, RcsTesting, UpdateRcsAgentRequest,
};
use serde_json::{json, Value};
use wiremock::matchers::{body_json, header, header_exists, method, path};
use wiremock::{Mock, ResponseTemplate};

const AUTO_KEY_PATTERN: &str =
    r"^sendly-rust-retry-[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

const NOT_ENABLED_MESSAGE: &str = "RCS registration isn't enabled for this account yet.";

fn brand_json() -> Value {
    json!({
        "id": "brd_abc123",
        "reviewStatus": "draft",
        "customerStage": "draft",
        "displayName": "Acme Coffee",
        "legalName": "Acme Coffee LLC",
        "legalEntityType": "LIMITED_LIABILITY_COMPANY",
        "organizationType": "PRIVATE_PROFIT",
        "stockSymbol": null,
        "websiteUrl": "https://acme.example",
        "ein": "12-3456789",
        "address": {
            "line1": "100 Main St",
            "line2": null,
            "city": "Chicago",
            "state": "IL",
            "postalCode": "60601",
            "countryCode": "US"
        },
        "contact": {
            "firstName": "Sam",
            "lastName": "Lee",
            "title": null,
            "email": "sam@acme.example",
            "phoneNumber": "+13125550100"
        },
        "reviewNote": null,
        "rejectionReason": null,
        "submittedForReviewAt": null,
        "sentToCarrierAt": null,
        "verifiedAt": null,
        "createdAt": "2026-09-01T10:00:00Z",
        "updatedAt": "2026-09-01T10:00:00Z"
    })
}

fn device_json() -> Value {
    json!({
        "id": "rtd_abc123",
        "phoneNumber": "+13125550100",
        "label": "Sam's Pixel",
        "inviteStatus": "PENDING",
        "createdAt": "2026-09-02T10:00:00Z"
    })
}

fn agent_json(review_status: &str, customer_stage: &str) -> Value {
    json!({
        "id": "rcs_agent_abc123",
        "brandId": "brd_abc123",
        "status": "draft",
        "reviewStatus": review_status,
        "customerStage": customer_stage,
        "displayName": "Acme Coffee",
        "useCase": "MULTI_USE",
        "hostingRegion": null,
        "basics": {
            "displayName": "Acme Coffee",
            "useCase": "MULTI_USE",
            "hostingRegion": null,
            "description": "Order updates and support for Acme Coffee customers",
            "logoUrl": "https://acme.example/rcs/logo.png",
            "heroUrl": "https://acme.example/rcs/hero.png",
            "brandColor": "#0B6E4F",
            "privacyPolicyUrl": "https://acme.example/privacy",
            "termsAndConditionsUrl": "https://acme.example/terms",
            "website": { "url": "https://acme.example", "label": "Visit our site" }
        },
        "campaign": {
            "agentOverview": "Order confirmations and support replies",
            "interactions": [
                { "interactionType": "TRANSACTIONAL_UPDATES", "description": "Order status" }
            ],
            "messageExamples": ["Your order #4821 is ready for pickup!"],
            "consentSettings": {
                "optInMethods": [{ "methodType": "WEBSITE", "description": "Checkout checkbox" }],
                "callToAction": "Text me order updates",
                "doubleOptIn": false
            }
        },
        "testing": { "testUrl": "https://acme.example/rcs-test", "messageId": null, "additionalInformation": null },
        "reviewNote": null,
        "rejectionReason": null,
        "testDevices": [device_json()],
        "submittedForReviewAt": null,
        "basicsSubmittedAt": null,
        "launchSubmittedAt": null,
        "liveAt": null,
        "createdAt": "2026-09-01T11:00:00Z",
        "updatedAt": "2026-09-01T11:00:00Z"
    })
}

fn not_enabled() -> ResponseTemplate {
    ResponseTemplate::new(404).set_body_json(json!({
        "error": "rcs_not_enabled",
        "message": NOT_ENABLED_MESSAGE
    }))
}

async fn idempotency_key_of_request(mock_server: &wiremock::MockServer, index: usize) -> String {
    let name = wiremock::http::HeaderName::from_string("idempotency-key".to_string()).unwrap();
    let requests = mock_server
        .received_requests()
        .await
        .expect("request recording is enabled");
    requests
        .get(index)
        .expect("request at index")
        .headers
        .get(&name)
        .map(|values| values.last().as_str().to_string())
        .expect("Idempotency-Key present")
}

// ==================== registration().get() Tests ====================

#[tokio::test]
async fn test_registration_get_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/registration"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "brand": brand_json(),
            "agent": agent_json("approved_for_carrier", "testing"),
            "devices": [device_json()],
            "stage": "testing",
            "usEligible": true
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().registration().get().await;

    assert!(result.is_ok());
    let registration = result.unwrap();
    assert_eq!(registration.stage, RcsCustomerStage::Testing);
    assert!(registration.us_eligible);
    let brand = registration.brand.expect("brand present");
    assert_eq!(brand.id, "brd_abc123");
    assert_eq!(brand.review_status, RcsReviewStatus::Draft);
    assert_eq!(brand.customer_stage, RcsCustomerStage::Draft);
    assert_eq!(brand.legal_entity_type, "LIMITED_LIABILITY_COMPANY");
    assert_eq!(brand.address.city, "Chicago");
    assert_eq!(brand.address.country_code, "US");
    assert!(brand.address.line2.is_none());
    assert_eq!(brand.contact.first_name, "Sam");
    assert_eq!(brand.contact.phone_number, "+13125550100");
    assert!(brand.contact.title.is_none());
    assert!(brand.stock_symbol.is_none());
    assert!(brand.verified_at.is_none());
    let agent = registration.agent.expect("agent present");
    assert_eq!(agent.id, "rcs_agent_abc123");
    assert_eq!(agent.review_status, RcsReviewStatus::ApprovedForCarrier);
    assert_eq!(agent.customer_stage, RcsCustomerStage::Testing);
    assert_eq!(agent.basics.display_name, "Acme Coffee");
    assert_eq!(
        agent.basics.logo_url.as_deref(),
        Some("https://acme.example/rcs/logo.png")
    );
    assert_eq!(
        agent
            .basics
            .website
            .as_ref()
            .and_then(|w| w.label.as_deref()),
        Some("Visit our site")
    );
    assert_eq!(registration.devices.len(), 1);
    assert_eq!(
        registration.devices[0].invite_status.as_deref(),
        Some("PENDING")
    );
}

#[tokio::test]
async fn test_registration_get_empty_workspace() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/registration"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "brand": null,
            "agent": null,
            "devices": [],
            "stage": "draft",
            "usEligible": true
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let registration = client.rcs().registration().get().await.unwrap();

    assert!(registration.brand.is_none());
    assert!(registration.agent.is_none());
    assert!(registration.devices.is_empty());
    assert_eq!(registration.stage, RcsCustomerStage::Draft);
    assert_eq!(registration.stage.as_str(), "draft");
}

#[tokio::test]
async fn test_registration_get_not_enabled() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/registration"))
        .respond_with(not_enabled())
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().registration().get().await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::NotFound { message } => assert_eq!(message, NOT_ENABLED_MESSAGE),
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

// ==================== dossier().get() Tests ====================

#[tokio::test]
async fn test_dossier_get_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/dossier"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "brand": {
                "legalName": "Acme Coffee LLC",
                "displayName": "Acme Coffee",
                "ein": "12-3456789",
                "organizationType": "PRIVATE_PROFIT",
                "websiteUrl": "https://acme.example",
                "address": {
                    "line1": "100 Main St",
                    "city": "Chicago",
                    "state": "IL",
                    "postalCode": "60601",
                    "countryCode": "US"
                },
                "contact": {
                    "firstName": "Sam",
                    "lastName": "Lee",
                    "email": "sam@acme.example",
                    "phoneNumber": "+13125550100"
                }
            },
            "usEligible": true,
            "source": "tendlc"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let dossier = client.rcs().dossier().get().await.unwrap();

    assert_eq!(dossier.source, "tendlc");
    assert!(dossier.us_eligible);
    assert_eq!(dossier.brand.legal_name.as_deref(), Some("Acme Coffee LLC"));
    assert_eq!(dossier.brand.ein.as_deref(), Some("12-3456789"));
    assert!(dossier.brand.legal_entity_type.is_none());
    assert!(dossier.brand.stock_symbol.is_none());
    let address = dossier.brand.address.as_ref().expect("address present");
    assert_eq!(address.postal_code.as_deref(), Some("60601"));
    assert!(address.line2.is_none());
    let contact = dossier.brand.contact.as_ref().expect("contact present");
    assert_eq!(contact.email.as_deref(), Some("sam@acme.example"));

    let reused = dossier.brand.legal_entity_type("LIMITED_LIABILITY_COMPANY");
    let body = serde_json::to_value(&reused).unwrap();
    assert_eq!(body["legalName"], "Acme Coffee LLC");
    assert_eq!(body["legalEntityType"], "LIMITED_LIABILITY_COMPANY");
    assert!(body.get("stockSymbol").is_none());
}

#[tokio::test]
async fn test_dossier_get_nothing_on_file() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/dossier"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "brand": {},
            "usEligible": true,
            "source": "none"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let dossier = client.rcs().dossier().get().await.unwrap();

    assert_eq!(dossier.source, "none");
    assert_eq!(dossier.brand, RcsBrandInput::new());
}

// ==================== brands().create() Tests ====================

#[tokio::test]
async fn test_brands_create_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/brands"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .and(header_exists("Idempotency-Key"))
        .and(body_json(json!({
            "displayName": "Acme Coffee",
            "legalName": "Acme Coffee LLC",
            "legalEntityType": "LIMITED_LIABILITY_COMPANY",
            "organizationType": "PRIVATE_PROFIT",
            "websiteUrl": "https://acme.example",
            "ein": "12-3456789",
            "address": {
                "line1": "100 Main St",
                "city": "Chicago",
                "state": "IL",
                "postalCode": "60601",
                "countryCode": "US"
            },
            "contact": {
                "firstName": "Sam",
                "lastName": "Lee",
                "email": "sam@acme.example",
                "phoneNumber": "+13125550100"
            }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "brand": brand_json() })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .brands()
        .create(
            RcsBrandInput::new()
                .display_name("Acme Coffee")
                .legal_name("Acme Coffee LLC")
                .legal_entity_type("LIMITED_LIABILITY_COMPANY")
                .organization_type("PRIVATE_PROFIT")
                .website_url("https://acme.example")
                .ein("12-3456789")
                .address(
                    RcsBrandAddressInput::new()
                        .line1("100 Main St")
                        .city("Chicago")
                        .state("IL")
                        .postal_code("60601")
                        .country_code("US"),
                )
                .contact(
                    RcsBrandContactInput::new()
                        .first_name("Sam")
                        .last_name("Lee")
                        .email("sam@acme.example")
                        .phone_number("+13125550100"),
                ),
        )
        .await;

    assert!(result.is_ok());
    let brand = result.unwrap().brand;
    assert_eq!(brand.id, "brd_abc123");
    assert_eq!(brand.review_status, RcsReviewStatus::Draft);
    assert_eq!(brand.customer_stage, RcsCustomerStage::Draft);
    assert_eq!(brand.display_name, "Acme Coffee");
    assert_eq!(brand.ein, "12-3456789");
    assert_eq!(brand.created_at, "2026-09-01T10:00:00Z");

    let key = idempotency_key_of_request(&mock_server, 0).await;
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_brands_create_with_options_sends_custom_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/brands"))
        .and(header("Idempotency-Key", "acme-brand-draft-1"))
        .and(body_json(json!({ "displayName": "Acme Coffee" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "brand": brand_json() })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .brands()
        .create_with_options(
            RcsBrandInput::new().display_name("Acme Coffee"),
            IdempotentRequestOptions::new().idempotency_key("acme-brand-draft-1"),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_brands_create_us_only() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/brands"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "rcs_us_only",
            "message": "RCS registration is available to US businesses for now."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .brands()
        .create(RcsBrandInput::new().address(RcsBrandAddressInput::new().country_code("GB")))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("US businesses")),
        other => panic!("Expected Validation error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_brands_create_not_enabled() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/brands"))
        .respond_with(not_enabled())
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .brands()
        .create(RcsBrandInput::new().display_name("Acme Coffee"))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::NotFound { message } => assert_eq!(message, NOT_ENABLED_MESSAGE),
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

// ==================== brands().update() Tests ====================

#[tokio::test]
async fn test_brands_update_success() {
    let mock_server = setup_mock_server().await;
    let mut updated = brand_json();
    updated["websiteUrl"] = json!("https://acme.example/new");
    updated["contact"]["title"] = json!("Head of Support");
    Mock::given(method("PATCH"))
        .and(path("/rcs/brands/brd_abc123"))
        .and(header_exists("Idempotency-Key"))
        .and(body_json(json!({
            "websiteUrl": "https://acme.example/new",
            "contact": { "title": "Head of Support" }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "brand": updated })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .brands()
        .update(
            "brd_abc123",
            RcsBrandInput::new()
                .website_url("https://acme.example/new")
                .contact(RcsBrandContactInput::new().title("Head of Support")),
        )
        .await;

    assert!(result.is_ok());
    let brand = result.unwrap().brand;
    assert_eq!(brand.website_url, "https://acme.example/new");
    assert_eq!(brand.contact.title.as_deref(), Some("Head of Support"));

    let key = idempotency_key_of_request(&mock_server, 0).await;
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_brands_update_with_options_sends_custom_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/rcs/brands/brd_abc123"))
        .and(header("Idempotency-Key", "acme-brand-fix-2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "brand": brand_json() })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .brands()
        .update_with_options(
            "brd_abc123",
            RcsBrandInput::new().ein("123456789"),
            IdempotentRequestOptions::new().idempotency_key("acme-brand-fix-2"),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_brands_update_field_locked() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/rcs/brands/brd_abc123"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "rcs_field_locked",
            "message": "This registration is being reviewed; we will email you if changes are needed."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .brands()
        .update("brd_abc123", RcsBrandInput::new().ein("123456789"))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api {
            status_code,
            code,
            message,
        } => {
            assert_eq!(status_code, 409);
            assert_eq!(code.as_deref(), Some("rcs_field_locked"));
            assert!(message.contains("being reviewed"));
        }
        other => panic!("Expected Api error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_brands_update_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/rcs/brands/brd_missing"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "rcs_not_found",
            "message": "Brand not found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .brands()
        .update("brd_missing", RcsBrandInput::new().ein("123456789"))
        .await;

    assert!(matches!(result.unwrap_err(), Error::NotFound { .. }));
}

// ==================== agents().create() Tests ====================

#[tokio::test]
async fn test_agents_create_success() {
    let mock_server = setup_mock_server().await;
    let mut created = agent_json("draft", "draft");
    created["campaign"] = Value::Null;
    created["testing"] = Value::Null;
    created["testDevices"] = json!([]);
    Mock::given(method("POST"))
        .and(path("/rcs/agents"))
        .and(header_exists("Idempotency-Key"))
        .and(body_json(json!({
            "brandId": "brd_abc123",
            "displayName": "Acme Coffee",
            "useCase": "MULTI_USE",
            "basics": {
                "description": "Order updates and support for Acme Coffee customers",
                "logoUrl": "https://acme.example/rcs/logo.png",
                "heroUrl": "https://acme.example/rcs/hero.png",
                "brandColor": "#0B6E4F",
                "privacyPolicyUrl": "https://acme.example/privacy",
                "termsAndConditionsUrl": "https://acme.example/terms",
                "website": { "url": "https://acme.example", "label": "Visit our site" }
            }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "agent": created })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .create(
            CreateRcsAgentRequest::new("brd_abc123")
                .display_name("Acme Coffee")
                .use_case("MULTI_USE")
                .basics(
                    RcsAgentBasicsInput::new()
                        .description("Order updates and support for Acme Coffee customers")
                        .logo_url("https://acme.example/rcs/logo.png")
                        .hero_url("https://acme.example/rcs/hero.png")
                        .brand_color("#0B6E4F")
                        .privacy_policy_url("https://acme.example/privacy")
                        .terms_and_conditions_url("https://acme.example/terms")
                        .website(
                            RcsAgentWebsiteContact::new("https://acme.example")
                                .label("Visit our site"),
                        ),
                ),
        )
        .await;

    assert!(result.is_ok());
    let agent = result.unwrap().agent;
    assert_eq!(agent.id, "rcs_agent_abc123");
    assert_eq!(agent.brand_id.as_deref(), Some("brd_abc123"));
    assert_eq!(agent.status, "draft");
    assert_eq!(agent.review_status, RcsReviewStatus::Draft);
    assert_eq!(agent.customer_stage, RcsCustomerStage::Draft);
    assert_eq!(agent.use_case.as_deref(), Some("MULTI_USE"));
    assert!(agent.hosting_region.is_none());
    assert_eq!(agent.basics.brand_color.as_deref(), Some("#0B6E4F"));
    assert!(agent.basics.phone_number.is_none());
    assert!(agent.campaign.is_none());
    assert!(agent.testing.is_none());
    assert!(agent.test_devices.is_empty());
    assert!(agent.live_at.is_none());

    let key = idempotency_key_of_request(&mock_server, 0).await;
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_agents_create_with_options_sends_custom_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/agents"))
        .and(header("Idempotency-Key", "acme-agent-draft-1"))
        .and(body_json(json!({ "brandId": "brd_abc123" })))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(json!({ "agent": agent_json("draft", "draft") })),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .create_with_options(
            CreateRcsAgentRequest::new("brd_abc123"),
            IdempotentRequestOptions::new().idempotency_key("acme-agent-draft-1"),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agents_create_rejects_non_https_media() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/agents"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "rcs_invalid_content",
            "message": "Assets can't be uploaded over the API. Logo, hero, and call-to-action media must be public https:// URLs.",
            "errors": [{ "path": "basics.logoUrl", "message": "Must be a public https:// URL" }]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .create(
            CreateRcsAgentRequest::new("brd_abc123")
                .basics(RcsAgentBasicsInput::new().logo_url("http://acme.example/logo.png")),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("public https:// URLs")),
        other => panic!("Expected Validation error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_agents_create_brand_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/agents"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "rcs_not_found",
            "message": "Brand not found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .create(CreateRcsAgentRequest::new("brd_missing"))
        .await;

    assert!(matches!(result.unwrap_err(), Error::NotFound { .. }));
}

// ==================== agents().get() Tests ====================

#[tokio::test]
async fn test_agents_get_success() {
    let mock_server = setup_mock_server().await;
    let mut agent = agent_json("changes_requested", "changes_requested");
    agent["reviewNote"] = json!("Please use a square logo");
    Mock::given(method("GET"))
        .and(path("/rcs/agents/rcs_agent_abc123"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agent": agent,
            "devices": [device_json()],
            "stage": "changes_requested"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().agents().get("rcs_agent_abc123").await;

    assert!(result.is_ok());
    let detail = result.unwrap();
    assert_eq!(detail.stage, RcsCustomerStage::ChangesRequested);
    assert_eq!(
        detail.agent.review_status,
        RcsReviewStatus::ChangesRequested
    );
    assert_eq!(
        detail.agent.review_note.as_deref(),
        Some("Please use a square logo")
    );
    let campaign = detail.agent.campaign.expect("campaign present");
    assert_eq!(
        campaign.agent_overview.as_deref(),
        Some("Order confirmations and support replies")
    );
    let interactions = campaign.interactions.expect("interactions present");
    assert_eq!(
        interactions[0].interaction_type.as_deref(),
        Some("TRANSACTIONAL_UPDATES")
    );
    assert_eq!(
        campaign.message_examples.as_deref(),
        Some(&["Your order #4821 is ready for pickup!".to_string()][..])
    );
    let consent = campaign.consent_settings.expect("consent present");
    assert_eq!(consent.double_opt_in, Some(false));
    assert_eq!(
        consent.opt_in_methods.as_ref().unwrap()[0]
            .method_type
            .as_deref(),
        Some("WEBSITE")
    );
    assert!(consent.help_response.is_none());
    let testing = detail.agent.testing.expect("testing present");
    assert_eq!(
        testing.test_url.as_deref(),
        Some("https://acme.example/rcs-test")
    );
    assert!(testing.message_id.is_none());
    assert_eq!(detail.devices.len(), 1);
    assert_eq!(detail.devices[0].id, "rtd_abc123");
    assert_eq!(detail.devices[0].label.as_deref(), Some("Sam's Pixel"));
    assert_eq!(detail.agent.test_devices.len(), 1);
}

#[tokio::test]
async fn test_agents_get_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/agents/rcs_agent_missing"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "rcs_not_found",
            "message": "Agent not found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().agents().get("rcs_agent_missing").await;

    match result.unwrap_err() {
        Error::NotFound { message } => assert_eq!(message, "Agent not found"),
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_agents_get_not_enabled() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/agents/rcs_agent_abc123"))
        .respond_with(not_enabled())
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().agents().get("rcs_agent_abc123").await;

    match result.unwrap_err() {
        Error::NotFound { message } => assert_eq!(message, NOT_ENABLED_MESSAGE),
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

// ==================== agents().update() Tests ====================

#[tokio::test]
async fn test_agents_update_campaign_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/rcs/agents/rcs_agent_abc123"))
        .and(header_exists("Idempotency-Key"))
        .and(body_json(json!({
            "campaign": {
                "agentOverview": "Order confirmations, pickup alerts, and support replies",
                "interactions": [
                    { "interactionType": "TRANSACTIONAL_UPDATES", "description": "Order status" }
                ],
                "messageExamples": [
                    "Your order #4821 is being roasted.",
                    "Your order #4821 is ready for pickup!",
                    "Thanks for visiting. Reply HELP for support."
                ],
                "consentSettings": {
                    "optInMethods": [{ "methodType": "WEBSITE", "description": "Checkout checkbox" }],
                    "callToAction": "Text me order updates",
                    "callToActionUrl": "https://acme.example/checkout",
                    "doubleOptIn": false,
                    "optInMessage": "Welcome to Acme Coffee updates. Reply STOP to opt out.",
                    "helpResponse": "Acme Coffee: email help@acme.example for support.",
                    "optOutResponse": "You have been unsubscribed from Acme Coffee updates."
                }
            },
            "testing": { "testUrl": "https://acme.example/rcs-test" }
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "agent": agent_json("approved_for_carrier", "testing") })),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .update(
            "rcs_agent_abc123",
            UpdateRcsAgentRequest::new()
                .campaign(
                    RcsCampaign::new()
                        .agent_overview("Order confirmations, pickup alerts, and support replies")
                        .interactions(vec![RcsInteraction::new(
                            "TRANSACTIONAL_UPDATES",
                            "Order status",
                        )])
                        .message_examples(vec![
                            "Your order #4821 is being roasted.".to_string(),
                            "Your order #4821 is ready for pickup!".to_string(),
                            "Thanks for visiting. Reply HELP for support.".to_string(),
                        ])
                        .consent_settings(
                            RcsConsentSettings::new()
                                .opt_in_methods(vec![RcsOptInMethod::new(
                                    "WEBSITE",
                                    "Checkout checkbox",
                                )])
                                .call_to_action("Text me order updates")
                                .call_to_action_url("https://acme.example/checkout")
                                .double_opt_in(false)
                                .opt_in_message(
                                    "Welcome to Acme Coffee updates. Reply STOP to opt out.",
                                )
                                .help_response("Acme Coffee: email help@acme.example for support.")
                                .opt_out_response(
                                    "You have been unsubscribed from Acme Coffee updates.",
                                ),
                        ),
                )
                .testing(RcsTesting::new().test_url("https://acme.example/rcs-test")),
        )
        .await;

    assert!(result.is_ok());
    let agent = result.unwrap().agent;
    assert_eq!(agent.customer_stage, RcsCustomerStage::Testing);

    let key = idempotency_key_of_request(&mock_server, 0).await;
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_agents_update_clears_sections_with_null() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/rcs/agents/rcs_agent_abc123"))
        .and(header("Idempotency-Key", "acme-agent-reset-3"))
        .and(body_json(json!({
            "displayName": "Acme Coffee Roasters",
            "campaign": null,
            "testing": null
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "agent": agent_json("draft", "draft") })),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .update_with_options(
            "rcs_agent_abc123",
            UpdateRcsAgentRequest::new()
                .display_name("Acme Coffee Roasters")
                .clear_campaign()
                .clear_testing(),
            IdempotentRequestOptions::new().idempotency_key("acme-agent-reset-3"),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agents_update_omits_untouched_sections() {
    let request = UpdateRcsAgentRequest::new().use_case("OTP");
    let body = serde_json::to_value(&request).unwrap();
    assert_eq!(body, json!({ "useCase": "OTP" }));
}

#[tokio::test]
async fn test_agents_update_field_locked() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/rcs/agents/rcs_agent_abc123"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "rcs_field_locked",
            "message": "This registration is being reviewed; we will email you if changes are needed."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .update(
            "rcs_agent_abc123",
            UpdateRcsAgentRequest::new().display_name("Acme"),
        )
        .await;

    match result.unwrap_err() {
        Error::Api {
            status_code, code, ..
        } => {
            assert_eq!(status_code, 409);
            assert_eq!(code.as_deref(), Some("rcs_field_locked"));
        }
        other => panic!("Expected Api error, got {:?}", other),
    }
}

// ==================== agents().set_test_devices() Tests ====================

#[tokio::test]
async fn test_agents_set_test_devices_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PUT"))
        .and(path("/rcs/agents/rcs_agent_abc123/test-devices"))
        .and(header_exists("Idempotency-Key"))
        .and(body_json(json!({
            "devices": [
                { "phoneNumber": "+13125550100", "label": "Sam's Pixel" },
                { "phoneNumber": "+13125550101" }
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "devices": [
                device_json(),
                {
                    "id": "rtd_def456",
                    "phoneNumber": "+13125550101",
                    "label": null,
                    "inviteStatus": null,
                    "createdAt": "2026-09-02T10:05:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .set_test_devices(
            "rcs_agent_abc123",
            vec![
                RcsTestDeviceInput::new("+13125550100").label("Sam's Pixel"),
                RcsTestDeviceInput::new("+13125550101"),
            ],
        )
        .await;

    assert!(result.is_ok());
    let devices = result.unwrap().devices;
    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].phone_number, "+13125550100");
    assert_eq!(devices[0].invite_status.as_deref(), Some("PENDING"));
    assert_eq!(devices[1].id, "rtd_def456");
    assert!(devices[1].label.is_none());
    assert!(devices[1].invite_status.is_none());

    let key = idempotency_key_of_request(&mock_server, 0).await;
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_agents_set_test_devices_with_options_sends_custom_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PUT"))
        .and(path("/rcs/agents/rcs_agent_abc123/test-devices"))
        .and(header("Idempotency-Key", "acme-devices-4"))
        .and(body_json(json!({ "devices": [] })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "devices": [] })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .set_test_devices_with_options(
            "rcs_agent_abc123",
            vec![],
            IdempotentRequestOptions::new().idempotency_key("acme-devices-4"),
        )
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap().devices.is_empty());
}

#[tokio::test]
async fn test_agents_set_test_devices_invalid_number() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PUT"))
        .and(path("/rcs/agents/rcs_agent_abc123/test-devices"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "rcs_invalid_content",
            "message": "Check the highlighted fields.",
            "errors": [{
                "path": "devices.0.phoneNumber",
                "message": "Enter the device's phone number in E.164 format, like +13125550100"
            }]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .set_test_devices("rcs_agent_abc123", vec![RcsTestDeviceInput::new("nope")])
        .await;

    assert!(matches!(result.unwrap_err(), Error::Validation { .. }));
}

// ==================== agents().submit() Tests ====================

#[tokio::test]
async fn test_agents_submit_success() {
    let mock_server = setup_mock_server().await;
    let mut submitted = agent_json("awaiting_review", "in_review");
    submitted["submittedForReviewAt"] = json!("2026-09-03T10:00:00Z");
    Mock::given(method("POST"))
        .and(path("/rcs/agents/rcs_agent_abc123/submit"))
        .and(header_exists("Idempotency-Key"))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agent": submitted,
            "stage": "in_review"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().agents().submit("rcs_agent_abc123").await;

    assert!(result.is_ok());
    let review = result.unwrap();
    assert_eq!(review.stage, RcsCustomerStage::InReview);
    assert_eq!(review.agent.review_status, RcsReviewStatus::AwaitingReview);
    assert_eq!(review.agent.review_status.to_string(), "awaiting_review");
    assert_eq!(
        review.agent.submitted_for_review_at.as_deref(),
        Some("2026-09-03T10:00:00Z")
    );

    let key = idempotency_key_of_request(&mock_server, 0).await;
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_agents_submit_with_options_sends_custom_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/agents/rcs_agent_abc123/submit"))
        .and(header("Idempotency-Key", "rcs-submit-rcs_agent_abc123"))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agent": agent_json("awaiting_review", "in_review"),
            "stage": "in_review"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .submit_with_options(
            "rcs_agent_abc123",
            IdempotentRequestOptions::new().idempotency_key("rcs-submit-rcs_agent_abc123"),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agents_submit_incomplete() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/agents/rcs_agent_abc123/submit"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "rcs_invalid_content",
            "message": "Finish the brand and agent before submitting.",
            "errors": [
                { "path": "brand.ein", "message": "Enter a 9-digit EIN" },
                { "path": "agent.logoUrl", "message": "Must be a public https:// URL" }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().agents().submit("rcs_agent_abc123").await;

    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("before submitting")),
        other => panic!("Expected Validation error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_agents_submit_brand_not_verified() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/agents/rcs_agent_abc123/submit"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "rcs_brand_not_verified",
            "message": "The brand failed verification."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().agents().submit("rcs_agent_abc123").await;

    match result.unwrap_err() {
        Error::Api {
            status_code, code, ..
        } => {
            assert_eq!(status_code, 409);
            assert_eq!(code.as_deref(), Some("rcs_brand_not_verified"));
        }
        other => panic!("Expected Api error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_agents_submit_not_enabled() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/agents/rcs_agent_abc123/submit"))
        .respond_with(not_enabled())
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.rcs().agents().submit("rcs_agent_abc123").await;

    match result.unwrap_err() {
        Error::NotFound { message } => assert_eq!(message, NOT_ENABLED_MESSAGE),
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

// ==================== agents().request_launch() Tests ====================

#[tokio::test]
async fn test_agents_request_launch_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/agents/rcs_agent_abc123/request-launch"))
        .and(header_exists("Idempotency-Key"))
        .and(body_json(json!({
            "testUrl": "https://acme.example/rcs-test",
            "testingAdditionalInformation": "Tap the chip to trigger a reply"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agent": agent_json("launch_requested", "launch_review"),
            "stage": "launch_review"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .request_launch(
            "rcs_agent_abc123",
            Some(
                RcsRequestLaunchRequest::new()
                    .test_url("https://acme.example/rcs-test")
                    .testing_additional_information("Tap the chip to trigger a reply"),
            ),
        )
        .await;

    assert!(result.is_ok());
    let review = result.unwrap();
    assert_eq!(review.stage, RcsCustomerStage::LaunchReview);
    assert_eq!(review.agent.review_status, RcsReviewStatus::LaunchRequested);

    let key = idempotency_key_of_request(&mock_server, 0).await;
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_agents_request_launch_without_body_sends_empty_object() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/agents/rcs_agent_abc123/request-launch"))
        .and(header("Idempotency-Key", "rcs-launch-rcs_agent_abc123"))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agent": agent_json("launch_requested", "launch_review"),
            "stage": "launch_review"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .request_launch_with_options(
            "rcs_agent_abc123",
            None,
            IdempotentRequestOptions::new().idempotency_key("rcs-launch-rcs_agent_abc123"),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agents_request_launch_not_ready() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/agents/rcs_agent_abc123/request-launch"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "rcs_launch_not_ready",
            "message": "This agent isn't ready to launch yet. Finish testing on an invited device first."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .request_launch("rcs_agent_abc123", None)
        .await;

    match result.unwrap_err() {
        Error::Api {
            status_code,
            code,
            message,
        } => {
            assert_eq!(status_code, 409);
            assert_eq!(code.as_deref(), Some("rcs_launch_not_ready"));
            assert!(message.contains("isn't ready to launch"));
        }
        other => panic!("Expected Api error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_agents_request_launch_incomplete_campaign() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rcs/agents/rcs_agent_abc123/request-launch"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "rcs_invalid_content",
            "message": "Finish the campaign before requesting launch.",
            "errors": [{ "path": "campaign.messageExamples", "message": "Add at least 3 message examples" }]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .rcs()
        .agents()
        .request_launch("rcs_agent_abc123", None)
        .await;

    assert!(matches!(result.unwrap_err(), Error::Validation { .. }));
}

// ==================== agents().list() stage Tests ====================

#[tokio::test]
async fn test_agents_list_reads_stage() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rcs/agents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "agents": [
                {
                    "id": "rcsa_abc123",
                    "name": "Acme Inc",
                    "status": "testing",
                    "useCase": "OTP",
                    "sendable": true,
                    "stage": "testing",
                    "createdAt": "2026-07-28T10:00:00Z"
                },
                {
                    "id": "rcsa_def456",
                    "name": "Acme Promos",
                    "status": "draft",
                    "useCase": null,
                    "sendable": false,
                    "createdAt": "2026-07-30T10:00:00Z"
                },
                {
                    "id": "rcsa_ghi789",
                    "name": "Acme Future",
                    "status": "draft",
                    "useCase": null,
                    "sendable": false,
                    "stage": "some_new_stage",
                    "createdAt": "2026-07-31T10:00:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let agents = client.rcs().agents().list().await.unwrap().agents;

    assert_eq!(agents[0].stage, Some(RcsCustomerStage::Testing));
    assert_eq!(agents[1].stage, None);
    assert_eq!(agents[2].stage, Some(RcsCustomerStage::Unknown));
}
