mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use regex::Regex;
use sendly::{
    CreateVoiceAgentRequest, Error, IdempotentRequestOptions, RegisterEmergencyAddressRequest,
    UpdateVoiceAgentRequest, UpdateVoiceNumberRequest, VoiceAgentToolsInput, VoiceMode,
};
use serde_json::{json, Value};
use wiremock::matchers::{body_json, header, header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const AUTO_KEY_PATTERN: &str =
    r"^sendly-rust-retry-[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

const NUMBER_ID: &str = "5f0c1c2e-2a44-4d4b-9d51-0a9b0f6f4a11";
const PHONE: &str = "+15555550188";
const ENCODED_PHONE: &str = "%2B15555550188";
const AGENT_ID: &str = "3c4d5e6f-7081-4293-a4b5-c6d7e8f90a1b";

fn number_json() -> Value {
    json!({
        "id": NUMBER_ID,
        "object": "voice_number",
        "phoneNumber": PHONE,
        "phoneNumberType": "local",
        "countryCode": "US",
        "isDefault": true,
        "voiceEnabled": true,
        "voiceMode": "agent",
        "agentId": AGENT_ID,
        "emergencyAddress": {
            "status": "active",
            "address": {
                "street": "500 Example Ave",
                "unit": "Suite 2",
                "city": "Austin",
                "state": "TX",
                "zip": "78701",
                "country": "US"
            }
        },
        "ratePerMinute": { "inbound": 2, "outbound": 2, "agent": 10 }
    })
}

fn agent_json() -> Value {
    json!({
        "id": AGENT_ID,
        "object": "voice_agent",
        "name": "Front desk",
        "enabled": true,
        "voice": "ashley",
        "voiceLabel": "Ashley (US, warm)",
        "language": "en-US",
        "greeting": "Thanks for calling Acme, how can I help?",
        "instructions": "Answer questions about opening hours.",
        "tools": { "sendSms": false, "transferTo": null },
        "canSendSms": true,
        "callsHandled": 12,
        "avgDurationSecs": 74,
        "createdAt": "2026-09-14T17:00:00.000Z",
        "updatedAt": "2026-09-14T17:05:00.000Z"
    })
}

async fn idempotency_key_of_request(mock_server: &MockServer, index: usize) -> Option<String> {
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
}

async fn body_of_request(mock_server: &MockServer, index: usize) -> Value {
    let requests = mock_server
        .received_requests()
        .await
        .expect("request recording is enabled");
    serde_json::from_slice(&requests.get(index).expect("request at index").body)
        .expect("request body is JSON")
}

// ==================== numbers() Tests ====================

#[tokio::test]
async fn test_numbers_list_unwraps_data() {
    let mock_server = setup_mock_server().await;
    let mut off = number_json();
    off["id"] = json!("7a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d");
    off["phoneNumber"] = json!("+15555550199");
    off["isDefault"] = json!(false);
    off["voiceEnabled"] = json!(false);
    off["voiceMode"] = json!("none");
    off["agentId"] = json!(null);
    off["emergencyAddress"] = json!(null);
    Mock::given(method("GET"))
        .and(path("/voice/numbers"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "data": [number_json(), off] })),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let numbers = client
        .voice()
        .numbers()
        .list()
        .await
        .expect("list should succeed");

    assert_eq!(numbers.data.len(), 2);

    let first = &numbers.data[0];
    assert_eq!(first.id, NUMBER_ID);
    assert_eq!(first.object, "voice_number");
    assert_eq!(first.phone_number, PHONE);
    assert_eq!(first.phone_number_type.as_deref(), Some("local"));
    assert_eq!(first.country_code.as_deref(), Some("US"));
    assert!(first.is_default);
    assert!(first.voice_enabled);
    assert_eq!(first.voice_mode, VoiceMode::Agent);
    assert_eq!(first.agent_id.as_deref(), Some(AGENT_ID));
    let emergency = first
        .emergency_address
        .as_ref()
        .expect("emergency address registered");
    assert!(emergency.is_active());
    let address = emergency.address.as_ref().expect("address on file");
    assert_eq!(address.street, "500 Example Ave");
    assert_eq!(address.unit.as_deref(), Some("Suite 2"));
    assert_eq!(address.city, "Austin");
    assert_eq!(address.state, "TX");
    assert_eq!(address.zip, "78701");
    assert_eq!(address.country, "US");
    assert_eq!(first.rate_per_minute.inbound, 2);
    assert_eq!(first.rate_per_minute.outbound, 2);
    assert_eq!(first.rate_per_minute.agent, 10);

    let second = &numbers.data[1];
    assert!(!second.voice_enabled);
    assert_eq!(second.voice_mode, VoiceMode::None);
    assert_eq!(second.agent_id, None);
    assert!(second.emergency_address.is_none());
}

#[tokio::test]
async fn test_numbers_get_percent_encodes_the_plus() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/voice/numbers/{}", ENCODED_PHONE)))
        .respond_with(ResponseTemplate::new(200).set_body_json(number_json()))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let number = client
        .voice()
        .numbers()
        .get(PHONE)
        .await
        .expect("get should succeed");

    assert_eq!(number.phone_number, PHONE);
    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(requests[0].url.path(), "/voice/numbers/%2B15555550188");
}

#[tokio::test]
async fn test_numbers_get_by_id_and_unknown_mode() {
    let mock_server = setup_mock_server().await;
    let mut body = number_json();
    body["voiceMode"] = json!("something_new");
    body["emergencyAddress"] = json!({ "status": "provisioning", "address": null });
    Mock::given(method("GET"))
        .and(path(format!("/voice/numbers/{}", NUMBER_ID)))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let number = client
        .voice()
        .numbers()
        .get(NUMBER_ID)
        .await
        .expect("get should succeed");

    assert_eq!(number.voice_mode, VoiceMode::Unknown);
    let emergency = number.emergency_address.expect("registration present");
    assert_eq!(emergency.status, "provisioning");
    assert!(!emergency.is_active());
    assert!(emergency.address.is_none());
}

#[tokio::test]
async fn test_numbers_update_sends_camel_case_body_with_agent_id() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path(format!("/voice/numbers/{}", ENCODED_PHONE)))
        .and(header_exists("Idempotency-Key"))
        .and(body_json(json!({
            "voiceEnabled": true,
            "voiceMode": "agent",
            "agentId": AGENT_ID
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(number_json()))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let number = client
        .voice()
        .numbers()
        .update(
            PHONE,
            UpdateVoiceNumberRequest::new()
                .voice_enabled(true)
                .voice_mode(VoiceMode::Agent)
                .agent_id(AGENT_ID),
        )
        .await
        .expect("update should succeed");

    assert_eq!(number.voice_mode, VoiceMode::Agent);
    let body = body_of_request(&mock_server, 0).await;
    assert!(body.get("voiceAgentId").is_none());
    assert!(body.get("voice_enabled").is_none());

    let key = idempotency_key_of_request(&mock_server, 0)
        .await
        .expect("Idempotency-Key present");
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_numbers_update_only_sends_what_changes() {
    let mock_server = setup_mock_server().await;
    let mut body = number_json();
    body["voiceMode"] = json!("ring_dashboard");
    body["agentId"] = json!(null);
    Mock::given(method("PATCH"))
        .and(path(format!("/voice/numbers/{}", NUMBER_ID)))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let number = client
        .voice()
        .numbers()
        .update(
            NUMBER_ID,
            UpdateVoiceNumberRequest::new()
                .voice_mode(VoiceMode::RingDashboard)
                .clear_agent_id(),
        )
        .await
        .expect("update should succeed");
    client
        .voice()
        .numbers()
        .update(NUMBER_ID, UpdateVoiceNumberRequest::new())
        .await
        .expect("an empty update is still sent");

    assert_eq!(number.voice_mode, VoiceMode::RingDashboard);
    assert_eq!(
        body_of_request(&mock_server, 0).await,
        json!({ "voiceMode": "ring_dashboard", "agentId": null })
    );
    assert_eq!(body_of_request(&mock_server, 1).await, json!({}));
}

#[tokio::test]
async fn test_numbers_update_with_options_sends_custom_idempotency_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path(format!("/voice/numbers/{}", ENCODED_PHONE)))
        .and(header("Idempotency-Key", "voice-off-2026-09-15"))
        .and(body_json(json!({ "voiceEnabled": false })))
        .respond_with(ResponseTemplate::new(200).set_body_json(number_json()))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .voice()
        .numbers()
        .update_with_options(
            PHONE,
            UpdateVoiceNumberRequest::new().voice_enabled(false),
            IdempotentRequestOptions::new().idempotency_key("voice-off-2026-09-15"),
        )
        .await;

    assert!(result.is_ok(), "{:?}", result.err());
}

#[tokio::test]
async fn test_numbers_update_agent_disabled_surfaces_code() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path(format!("/voice/numbers/{}", ENCODED_PHONE)))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "agent_disabled",
            "message": "That agent is switched off. Turn it on before pointing a number at it."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .voice()
        .numbers()
        .update(
            PHONE,
            UpdateVoiceNumberRequest::new()
                .voice_mode(VoiceMode::Agent)
                .agent_id(AGENT_ID),
        )
        .await;

    match result.unwrap_err() {
        Error::Api {
            status_code, code, ..
        } => {
            assert_eq!(status_code, 409);
            assert_eq!(code.as_deref(), Some("agent_disabled"));
        }
        other => panic!("expected Api, got {:?}", other),
    }
}

#[tokio::test]
async fn test_register_emergency_address_posts_address_with_idempotency_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/voice/numbers/{}/emergency-address",
            ENCODED_PHONE
        )))
        .and(header_exists("Idempotency-Key"))
        .and(body_json(json!({
            "street": "500 Example Ave",
            "unit": "Suite 2",
            "city": "Austin",
            "state": "TX",
            "zip": "78701"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(number_json()))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let number = client
        .voice()
        .numbers()
        .register_emergency_address(
            PHONE,
            RegisterEmergencyAddressRequest::new("500 Example Ave", "Austin", "TX", "78701")
                .unit("Suite 2"),
        )
        .await
        .expect("registration should succeed");

    assert!(number
        .emergency_address
        .expect("registration present")
        .is_active());
    let key = idempotency_key_of_request(&mock_server, 0)
        .await
        .expect("Idempotency-Key present");
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_register_emergency_address_with_options_and_country() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/voice/numbers/{}/emergency-address",
            NUMBER_ID
        )))
        .and(header("Idempotency-Key", "e911-toronto"))
        .and(body_json(json!({
            "street": "100 Example St",
            "city": "Toronto",
            "state": "ON",
            "zip": "M5V 2T6",
            "country": "CA"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(number_json()))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .voice()
        .numbers()
        .register_emergency_address_with_options(
            NUMBER_ID,
            RegisterEmergencyAddressRequest::new("100 Example St", "Toronto", "ON", "M5V 2T6")
                .country("CA"),
            IdempotentRequestOptions::new().idempotency_key("e911-toronto"),
        )
        .await;

    assert!(result.is_ok(), "{:?}", result.err());
}

#[tokio::test]
async fn test_register_emergency_address_requires_address_fields_before_sending() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let cases = [
        (
            RegisterEmergencyAddressRequest::new(" ", "Austin", "TX", "78701"),
            "street is required",
        ),
        (
            RegisterEmergencyAddressRequest::new("500 Example Ave", "", "TX", "78701"),
            "city is required",
        ),
        (
            RegisterEmergencyAddressRequest::new("500 Example Ave", "Austin", "", "78701"),
            "state is required",
        ),
        (
            RegisterEmergencyAddressRequest::new("500 Example Ave", "Austin", "TX", "  "),
            "zip is required",
        ),
    ];
    for (request, expected) in cases {
        match client
            .voice()
            .numbers()
            .register_emergency_address(PHONE, request)
            .await
            .unwrap_err()
        {
            Error::Validation { message } => assert_eq!(message, expected),
            other => panic!("expected Validation, got {:?}", other),
        }
    }

    let requests = mock_server.received_requests().await.unwrap();
    assert!(requests.is_empty(), "no request should reach the server");
}

#[tokio::test]
async fn test_register_emergency_address_unvalidated_maps_to_validation() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/voice/numbers/{}/emergency-address",
            ENCODED_PHONE
        )))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "error": "invalid_address",
            "message": "We couldn't validate that address.",
            "suggested": {
                "street": "500 Example Avenue",
                "city": "Austin",
                "state": "TX",
                "zip": "78701",
                "country": "US"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .voice()
        .numbers()
        .register_emergency_address(
            PHONE,
            RegisterEmergencyAddressRequest::new("500 Example Ave", "Austin", "TX", "78701"),
        )
        .await;

    match result.unwrap_err() {
        Error::Validation { message } => assert_eq!(message, "We couldn't validate that address."),
        other => panic!("expected Validation, got {:?}", other),
    }
}

#[tokio::test]
async fn test_number_and_agent_methods_require_an_identifier() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    assert!(matches!(
        client.voice().numbers().get("").await.unwrap_err(),
        Error::Validation { .. }
    ));
    assert!(matches!(
        client
            .voice()
            .numbers()
            .update(" ", UpdateVoiceNumberRequest::new().voice_enabled(true))
            .await
            .unwrap_err(),
        Error::Validation { .. }
    ));
    assert!(matches!(
        client
            .voice()
            .numbers()
            .register_emergency_address(
                "",
                RegisterEmergencyAddressRequest::new("500 Example Ave", "Austin", "TX", "78701"),
            )
            .await
            .unwrap_err(),
        Error::Validation { .. }
    ));
    assert!(matches!(
        client.voice().agents().get("").await.unwrap_err(),
        Error::Validation { .. }
    ));
    assert!(matches!(
        client
            .voice()
            .agents()
            .update("  ", UpdateVoiceAgentRequest::new().enabled(false))
            .await
            .unwrap_err(),
        Error::Validation { .. }
    ));
    match client.voice().agents().delete("").await.unwrap_err() {
        Error::Validation { message } => assert_eq!(message, "Agent id is required"),
        other => panic!("expected Validation, got {:?}", other),
    }

    let requests = mock_server.received_requests().await.unwrap();
    assert!(requests.is_empty(), "no request should reach the server");
}

#[tokio::test]
async fn test_voice_not_enabled_maps_to_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/voice/numbers"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "voice_not_enabled",
            "message": "Voice is not enabled for your account."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    match client.voice().numbers().list().await.unwrap_err() {
        Error::NotFound { message } => assert_eq!(message, "Voice is not enabled for your account."),
        other => panic!("expected NotFound, got {:?}", other),
    }
}

// ==================== agents() Tests ====================

#[tokio::test]
async fn test_agents_list_unwraps_data() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/voice/agents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [agent_json()] })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let agents = client
        .voice()
        .agents()
        .list()
        .await
        .expect("list should succeed");

    assert_eq!(agents.data.len(), 1);
    let agent = &agents.data[0];
    assert_eq!(agent.id, AGENT_ID);
    assert_eq!(agent.object, "voice_agent");
    assert_eq!(agent.name, "Front desk");
    assert!(agent.enabled);
    assert_eq!(agent.voice, "ashley");
    assert_eq!(agent.voice_label, "Ashley (US, warm)");
    assert_eq!(agent.language, "en-US");
    assert_eq!(agent.greeting, "Thanks for calling Acme, how can I help?");
    assert_eq!(agent.instructions, "Answer questions about opening hours.");
    assert!(!agent.tools.send_sms);
    assert_eq!(agent.tools.transfer_to, None);
    assert!(agent.can_send_sms);
    assert_eq!(agent.calls_handled, 12);
    assert_eq!(agent.avg_duration_secs, 74);
    assert_eq!(agent.created_at, "2026-09-14T17:00:00.000Z");
    assert_eq!(agent.updated_at, "2026-09-14T17:05:00.000Z");
}

#[tokio::test]
async fn test_agents_create_sends_body_and_idempotency_key() {
    let mock_server = setup_mock_server().await;
    let mut body = agent_json();
    body["tools"] = json!({ "sendSms": true, "transferTo": "+15125550142" });
    body["callsHandled"] = json!(0);
    body["avgDurationSecs"] = json!(0);
    Mock::given(method("POST"))
        .and(path("/voice/agents"))
        .and(header_exists("Idempotency-Key"))
        .and(body_json(json!({
            "name": "Front desk",
            "voice": "ashley",
            "language": "en-US",
            "greeting": "Thanks for calling Acme, how can I help?",
            "instructions": "Answer questions about opening hours.",
            "tools": { "sendSms": true, "transferTo": "+15125550142" }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let agent = client
        .voice()
        .agents()
        .create(
            CreateVoiceAgentRequest::new("Front desk")
                .voice("ashley")
                .language("en-US")
                .greeting("Thanks for calling Acme, how can I help?")
                .instructions("Answer questions about opening hours.")
                .tools(
                    VoiceAgentToolsInput::new()
                        .send_sms(true)
                        .transfer_to("+15125550142"),
                ),
        )
        .await
        .expect("create should succeed");

    assert_eq!(agent.id, AGENT_ID);
    assert!(agent.tools.send_sms);
    assert_eq!(agent.tools.transfer_to.as_deref(), Some("+15125550142"));
    assert_eq!(agent.calls_handled, 0);

    let key = idempotency_key_of_request(&mock_server, 0)
        .await
        .expect("Idempotency-Key present");
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_agents_create_with_only_a_name_and_custom_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/voice/agents"))
        .and(header("Idempotency-Key", "agent-front-desk"))
        .and(body_json(json!({ "name": "Front desk" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(agent_json()))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .voice()
        .agents()
        .create_with_options(
            CreateVoiceAgentRequest::new("Front desk"),
            IdempotentRequestOptions::new().idempotency_key("agent-front-desk"),
        )
        .await;

    assert!(result.is_ok(), "{:?}", result.err());
}

#[tokio::test]
async fn test_agents_create_requires_a_name_before_sending() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    match client
        .voice()
        .agents()
        .create(CreateVoiceAgentRequest::new("   "))
        .await
        .unwrap_err()
    {
        Error::Validation { message } => assert_eq!(message, "name is required"),
        other => panic!("expected Validation, got {:?}", other),
    }

    let requests = mock_server.received_requests().await.unwrap();
    assert!(requests.is_empty(), "no request should reach the server");
}

#[tokio::test]
async fn test_agents_create_limit_surfaces_code() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/voice/agents"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "agent_limit",
            "message": "You've reached the agent limit for this workspace."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .voice()
        .agents()
        .create(CreateVoiceAgentRequest::new("Front desk"))
        .await;

    match result.unwrap_err() {
        Error::Api {
            status_code, code, ..
        } => {
            assert_eq!(status_code, 409);
            assert_eq!(code.as_deref(), Some("agent_limit"));
        }
        other => panic!("expected Api, got {:?}", other),
    }
}

#[tokio::test]
async fn test_agents_get_percent_encodes_the_id() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/voice/agents/agent%2F..%2Faccount"))
        .respond_with(ResponseTemplate::new(200).set_body_json(agent_json()))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.voice().agents().get("agent/../account").await;

    assert!(result.is_ok(), "{:?}", result.err());
}

#[tokio::test]
async fn test_agents_update_sends_only_changed_fields() {
    let mock_server = setup_mock_server().await;
    let mut body = agent_json();
    body["enabled"] = json!(false);
    Mock::given(method("PATCH"))
        .and(path(format!("/voice/agents/{}", AGENT_ID)))
        .and(header_exists("Idempotency-Key"))
        .and(body_json(json!({
            "enabled": false,
            "greeting": "",
            "tools": { "transferTo": null }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let agent = client
        .voice()
        .agents()
        .update(
            AGENT_ID,
            UpdateVoiceAgentRequest::new()
                .enabled(false)
                .greeting("")
                .tools(VoiceAgentToolsInput::new().clear_transfer_to()),
        )
        .await
        .expect("update should succeed");

    assert!(!agent.enabled);
}

#[tokio::test]
async fn test_agents_delete_returns_confirmation() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/voice/agents/{}", AGENT_ID)))
        .and(header_exists("Idempotency-Key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": AGENT_ID,
            "object": "voice_agent",
            "deleted": true
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let deleted = client
        .voice()
        .agents()
        .delete(AGENT_ID)
        .await
        .expect("delete should succeed");

    assert_eq!(deleted.id, AGENT_ID);
    assert_eq!(deleted.object, "voice_agent");
    assert!(deleted.deleted);

    let requests = mock_server.received_requests().await.unwrap();
    assert!(requests[0].body.is_empty(), "DELETE sends no body");
    let key = idempotency_key_of_request(&mock_server, 0)
        .await
        .expect("Idempotency-Key present");
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_agents_delete_with_options_sends_custom_idempotency_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/voice/agents/{}", AGENT_ID)))
        .and(header("Idempotency-Key", "delete-front-desk"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": AGENT_ID,
            "object": "voice_agent",
            "deleted": true
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .voice()
        .agents()
        .delete_with_options(
            AGENT_ID,
            IdempotentRequestOptions::new().idempotency_key("delete-front-desk"),
        )
        .await;

    assert!(result.is_ok(), "{:?}", result.err());
}

#[tokio::test]
async fn test_agents_delete_in_use_surfaces_code() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/voice/agents/{}", AGENT_ID)))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "agent_in_use",
            "message": "This agent answers 1 number. Point it elsewhere first.",
            "numbers": [PHONE]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.voice().agents().delete(AGENT_ID).await;

    match result.unwrap_err() {
        Error::Api {
            status_code,
            code,
            message,
        } => {
            assert_eq!(status_code, 409);
            assert_eq!(code.as_deref(), Some("agent_in_use"));
            assert_eq!(message, "This agent answers 1 number. Point it elsewhere first.");
        }
        other => panic!("expected Api, got {:?}", other),
    }
}

#[tokio::test]
async fn test_agents_get_not_found_maps_to_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/voice/agents/{}", AGENT_ID)))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "agent_not_found",
            "message": "That agent doesn't exist in this workspace."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    match client.voice().agents().get(AGENT_ID).await.unwrap_err() {
        Error::NotFound { message } => {
            assert_eq!(message, "That agent doesn't exist in this workspace.")
        }
        other => panic!("expected NotFound, got {:?}", other),
    }
}

// ==================== voices() Tests ====================

#[tokio::test]
async fn test_voices_list_unwraps_data() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/voice/voices"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                { "id": "ashley", "label": "Ashley (US, warm)", "language": "en" },
                { "id": "diego", "label": "Diego (Spanish, MX)", "language": "es" }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let voices = client
        .voice()
        .voices()
        .list()
        .await
        .expect("list should succeed");

    assert_eq!(voices.data.len(), 2);
    assert_eq!(voices.data[0].id, "ashley");
    assert_eq!(voices.data[0].label, "Ashley (US, warm)");
    assert_eq!(voices.data[1].language, "es");
}

#[test]
fn test_voice_mode_spells_like_the_api() {
    assert_eq!(VoiceMode::None.as_str(), "none");
    assert_eq!(VoiceMode::RingDashboard.to_string(), "ring_dashboard");
    assert_eq!(VoiceMode::Agent.as_str(), "agent");
    assert_eq!(
        serde_json::to_value(VoiceMode::RingDashboard).unwrap(),
        json!("ring_dashboard")
    );
}
