mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use regex::Regex;
use sendly::webhooks::Webhooks;
use sendly::{
    CallBilling, CallDirection, CallHandledBy, CallKind, CallRecordingStatus, CallStatus,
    CallTranscriptSpeaker, CreateCallRequest, Error, IdempotentRequestOptions, ListCallsOptions,
};
use serde_json::{json, Value};
use wiremock::matchers::{body_json, header, header_exists, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const AUTO_KEY_PATTERN: &str =
    r"^sendly-rust-retry-[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

const CALL_ID: &str = "6f1c2d3e-4a5b-4c6d-8e9f-0a1b2c3d4e5f";
const AGENT_ID: &str = "3c4d5e6f-7081-4293-a4b5-c6d7e8f90a1b";

fn call_json(status: &str) -> Value {
    json!({
        "id": CALL_ID,
        "object": "call",
        "kind": "pstn",
        "direction": "outbound",
        "status": status,
        "handledBy": "agent",
        "agentId": AGENT_ID,
        "from": "+15555550188",
        "to": "+15555550123",
        "callerName": "Front Desk",
        "calleeName": "+15555550123",
        "startedAt": "2026-09-12T14:03:11.000Z",
        "answeredAt": null,
        "endedAt": null,
        "durationSecs": 0,
        "creditsCharged": 0,
        "billing": "metered",
        "hangupClass": null,
        "recordingStatus": null,
        "metadata": { "crmId": "lead_8812" }
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

// ==================== create() Tests ====================

#[tokio::test]
async fn test_create_sends_full_body_and_idempotency_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/calls"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .and(header_exists("Idempotency-Key"))
        .and(body_json(json!({
            "to": "+15555550123",
            "agentId": AGENT_ID,
            "from": "+15555550188",
            "context": "You are calling Jordan to confirm the 3pm appointment on Tuesday.",
            "metadata": { "crmId": "lead_8812" }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(call_json("ringing")))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let call = client
        .calls()
        .create(
            CreateCallRequest::new("+15555550123", AGENT_ID)
                .from_number("+15555550188")
                .context("You are calling Jordan to confirm the 3pm appointment on Tuesday.")
                .metadata_entry("crmId", "lead_8812"),
        )
        .await
        .expect("create should succeed");

    assert_eq!(call.id, CALL_ID);
    assert_eq!(call.object, "call");
    assert_eq!(call.status, CallStatus::Ringing);
    assert!(call.is_live());
    assert!(!call.is_ended());
    assert_eq!(call.kind, CallKind::Pstn);
    assert_eq!(call.direction, CallDirection::Outbound);
    assert_eq!(call.handled_by, CallHandledBy::Agent);
    assert_eq!(call.agent_id.as_deref(), Some(AGENT_ID));
    assert_eq!(call.from_number.as_deref(), Some("+15555550188"));
    assert_eq!(call.to.as_deref(), Some("+15555550123"));
    assert_eq!(call.caller_name.as_deref(), Some("Front Desk"));
    assert_eq!(call.billing, CallBilling::Metered);
    assert_eq!(call.credits_charged, 0);
    assert_eq!(call.duration_secs, 0);
    assert_eq!(call.hangup_class, None);
    assert_eq!(call.recording_status, None);
    assert_eq!(call.metadata.get("crmId").map(String::as_str), Some("lead_8812"));
    assert!(call.transcript.is_none());

    let key = idempotency_key_of_request(&mock_server, 0)
        .await
        .expect("Idempotency-Key present");
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_create_omits_unset_optional_fields() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/calls"))
        .and(body_json(json!({ "to": "+15555550123", "agentId": AGENT_ID })))
        .respond_with(ResponseTemplate::new(201).set_body_json(call_json("ringing")))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .calls()
        .create(CreateCallRequest::new("+15555550123", AGENT_ID))
        .await;

    assert!(result.is_ok(), "{:?}", result.err());
}

#[tokio::test]
async fn test_create_with_options_sends_custom_idempotency_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/calls"))
        .and(header("Idempotency-Key", "call-lead_8812-2026-09-12"))
        .respond_with(ResponseTemplate::new(201).set_body_json(call_json("ringing")))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .calls()
        .create_with_options(
            CreateCallRequest::new("+15555550123", AGENT_ID),
            IdempotentRequestOptions::new().idempotency_key("call-lead_8812-2026-09-12"),
        )
        .await;

    assert!(result.is_ok(), "{:?}", result.err());
    assert_eq!(
        idempotency_key_of_request(&mock_server, 0).await.as_deref(),
        Some("call-lead_8812-2026-09-12")
    );
}

#[tokio::test]
async fn test_create_requires_to_and_agent_id_before_sending() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let missing_to = client
        .calls()
        .create(CreateCallRequest::new("", AGENT_ID))
        .await;
    match missing_to.unwrap_err() {
        Error::Validation { message } => assert_eq!(message, "to is required"),
        other => panic!("expected Validation, got {:?}", other),
    }

    let missing_agent = client
        .calls()
        .create(CreateCallRequest::new("+15555550123", "  "))
        .await;
    match missing_agent.unwrap_err() {
        Error::Validation { message } => assert_eq!(message, "agentId is required"),
        other => panic!("expected Validation, got {:?}", other),
    }

    let requests = mock_server.received_requests().await.unwrap();
    assert!(requests.is_empty(), "no request should reach the server");
}

#[tokio::test]
async fn test_create_insufficient_credits_maps_to_credits_error() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/calls"))
        .respond_with(ResponseTemplate::new(402).set_body_json(json!({
            "error": "insufficient_credits",
            "message": "Calls cost 10 credits a minute. Current balance: 4.",
            "creditsNeeded": 10,
            "currentBalance": 4
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .calls()
        .create(CreateCallRequest::new("+15555550123", AGENT_ID))
        .await;

    match result.unwrap_err() {
        Error::InsufficientCredits { message } => {
            assert_eq!(message, "Calls cost 10 credits a minute. Current balance: 4.")
        }
        other => panic!("expected InsufficientCredits, got {:?}", other),
    }
}

#[tokio::test]
async fn test_create_e911_required_surfaces_code() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/calls"))
        .respond_with(ResponseTemplate::new(428).set_body_json(json!({
            "error": "e911_required",
            "message": "Register an emergency address for this number before placing calls. It's required by US law."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .calls()
        .create(CreateCallRequest::new("+15555550123", AGENT_ID))
        .await;

    match result.unwrap_err() {
        Error::Api {
            status_code,
            code,
            message,
        } => {
            assert_eq!(status_code, 428);
            assert_eq!(code.as_deref(), Some("e911_required"));
            assert!(message.starts_with("Register an emergency address"));
        }
        other => panic!("expected Api, got {:?}", other),
    }
}

#[tokio::test]
async fn test_create_lines_busy_surfaces_code() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/calls"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "lines_busy",
            "message": "Your workspace's lines are all in use. Try again in a moment."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .calls()
        .create(CreateCallRequest::new("+15555550123", AGENT_ID))
        .await;

    match result.unwrap_err() {
        Error::Api {
            status_code, code, ..
        } => {
            assert_eq!(status_code, 409);
            assert_eq!(code.as_deref(), Some("lines_busy"));
        }
        other => panic!("expected Api, got {:?}", other),
    }
}

#[tokio::test]
async fn test_create_agent_required_maps_to_validation() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/calls"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "agent_required",
            "message": "Calls placed over the API are answered by an AI agent. Pass agentId."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .calls()
        .create(CreateCallRequest::new("+15555550123", AGENT_ID))
        .await;

    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("Pass agentId")),
        other => panic!("expected Validation, got {:?}", other),
    }
}

#[tokio::test]
async fn test_create_voice_not_enabled_maps_to_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/calls"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "voice_not_enabled",
            "message": "Voice is not enabled for your account."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .calls()
        .create(CreateCallRequest::new("+15555550123", AGENT_ID))
        .await;

    match result.unwrap_err() {
        Error::NotFound { message } => {
            assert_eq!(message, "Voice is not enabled for your account.")
        }
        other => panic!("expected NotFound, got {:?}", other),
    }
}

// ==================== list() Tests ====================

#[tokio::test]
async fn test_list_encodes_every_filter() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/calls"))
        .and(query_param("limit", "20"))
        .and(query_param("offset", "40"))
        .and(query_param("status", "completed"))
        .and(query_param("direction", "outbound"))
        .and(query_param("kind", "pstn"))
        .and(query_param("agentId", AGENT_ID))
        .and(query_param("to", "+15555550123"))
        .and(query_param("from", "+15555550188"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [call_json("completed"), call_json("no_answer")],
            "pagination": { "total": 132, "limit": 20, "offset": 40, "hasMore": true }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let page = client
        .calls()
        .list(Some(
            ListCallsOptions::new()
                .limit(20)
                .offset(40)
                .status(CallStatus::Completed)
                .direction(CallDirection::Outbound)
                .kind(CallKind::Pstn)
                .agent_id(AGENT_ID)
                .to("+15555550123")
                .from_number("+15555550188"),
        ))
        .await
        .expect("list should succeed");

    assert_eq!(page.data.len(), 2);
    assert_eq!(page.data[0].status, CallStatus::Completed);
    assert_eq!(page.data[1].status, CallStatus::NoAnswer);
    assert_eq!(page.pagination.total, 132);
    assert_eq!(page.pagination.limit, 20);
    assert_eq!(page.pagination.offset, 40);
    assert!(page.pagination.has_more);
}

#[tokio::test]
async fn test_list_clamps_limit_to_100() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/calls"))
        .and(query_param("limit", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "pagination": { "total": 0, "limit": 100, "offset": 0, "hasMore": false }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let page = client
        .calls()
        .list(Some(ListCallsOptions::new().limit(500)))
        .await
        .expect("list should succeed");

    assert!(page.data.is_empty());
    assert!(!page.pagination.has_more);
}

#[tokio::test]
async fn test_list_without_options_sends_no_query() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/calls"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "pagination": { "total": 0, "limit": 50, "offset": 0, "hasMore": false }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let page = client.calls().list(None).await.expect("list should succeed");
    assert!(page.data.is_empty());

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(requests[0].url.query(), None);
}

// ==================== get() Tests ====================

#[tokio::test]
async fn test_get_agent_call_carries_transcript() {
    let mock_server = setup_mock_server().await;
    let mut body = call_json("completed");
    body["answeredAt"] = json!("2026-09-12T14:03:19.000Z");
    body["endedAt"] = json!("2026-09-12T14:05:02.000Z");
    body["durationSecs"] = json!(103);
    body["creditsCharged"] = json!(20);
    body["billing"] = json!("settled");
    body["hangupClass"] = json!("agent_agent_hangup");
    body["recordingStatus"] = json!("ready");
    body["transcript"] = json!([
        { "speaker": "agent", "text": "Hi Jordan, this is the front desk.", "atMs": 1200 },
        { "speaker": "caller", "text": "Yes, Tuesday at 3 works.", "atMs": 6400 }
    ]);
    Mock::given(method("GET"))
        .and(path(format!("/calls/{}", CALL_ID)))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let call = client.calls().get(CALL_ID).await.expect("get should succeed");

    assert_eq!(call.status, CallStatus::Completed);
    assert!(call.is_ended());
    assert_eq!(call.billing, CallBilling::Settled);
    assert_eq!(call.duration_secs, 103);
    assert_eq!(call.credits_charged, 20);
    assert_eq!(call.hangup_class.as_deref(), Some("agent_agent_hangup"));
    assert_eq!(call.recording_status, Some(CallRecordingStatus::Ready));
    assert_eq!(call.answered_at.as_deref(), Some("2026-09-12T14:03:19.000Z"));

    let transcript = call.transcript.expect("agent calls carry a transcript");
    assert_eq!(transcript.len(), 2);
    assert_eq!(transcript[0].speaker, CallTranscriptSpeaker::Agent);
    assert_eq!(transcript[0].at_ms, 1200);
    assert_eq!(transcript[1].speaker, CallTranscriptSpeaker::Caller);
    assert_eq!(transcript[1].text, "Yes, Tuesday at 3 works.");
}

#[tokio::test]
async fn test_get_dashboard_call_has_no_transcript_and_empty_metadata() {
    let mock_server = setup_mock_server().await;
    let mut body = call_json("completed");
    body["handledBy"] = json!("dashboard");
    body["agentId"] = json!(null);
    body["direction"] = json!("inbound");
    body["metadata"] = json!({});
    body["billing"] = json!("settled");
    Mock::given(method("GET"))
        .and(path(format!("/calls/{}", CALL_ID)))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let call = client.calls().get(CALL_ID).await.expect("get should succeed");

    assert_eq!(call.handled_by, CallHandledBy::Dashboard);
    assert_eq!(call.direction, CallDirection::Inbound);
    assert_eq!(call.agent_id, None);
    assert!(call.metadata.is_empty());
    assert!(call.transcript.is_none());
}

#[tokio::test]
async fn test_get_internal_call_tolerates_null_numbers_and_unknown_values() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/calls/{}", CALL_ID)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": CALL_ID,
            "object": "call",
            "kind": "internal",
            "direction": "inbound",
            "status": "something_new",
            "handledBy": "dashboard",
            "agentId": null,
            "from": null,
            "to": null,
            "callerName": "Sam",
            "calleeName": "Lee",
            "startedAt": "2026-09-12T14:03:11.000Z",
            "answeredAt": null,
            "endedAt": null,
            "durationSecs": 0,
            "creditsCharged": 0,
            "billing": "unbilled",
            "hangupClass": null,
            "recordingStatus": "later_state",
            "metadata": {}
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let call = client.calls().get(CALL_ID).await.expect("get should succeed");

    assert_eq!(call.kind, CallKind::Internal);
    assert_eq!(call.from_number, None);
    assert_eq!(call.to, None);
    assert_eq!(call.status, CallStatus::Unknown);
    assert!(!call.is_live());
    assert!(!call.is_ended());
    assert_eq!(call.billing, CallBilling::Unbilled);
    assert_eq!(call.recording_status, Some(CallRecordingStatus::Unknown));
}

#[tokio::test]
async fn test_is_ended_only_for_terminal_statuses() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());
    let cases = [
        ("ringing", false, true),
        ("active", false, true),
        ("completed", true, false),
        ("no_answer", true, false),
        ("busy", true, false),
        ("cancelled", true, false),
        ("declined", true, false),
        ("failed", true, false),
        ("suspended", false, false),
        ("something_new", false, false),
    ];
    for (status, ended, live) in cases {
        let scoped = Mock::given(method("GET"))
            .and(path(format!("/calls/{}", CALL_ID)))
            .respond_with(ResponseTemplate::new(200).set_body_json(call_json(status)))
            .mount_as_scoped(&mock_server)
            .await;
        let call = client.calls().get(CALL_ID).await.expect("get should succeed");
        assert_eq!(call.is_ended(), ended, "is_ended for {status}");
        assert_eq!(call.is_live(), live, "is_live for {status}");
        drop(scoped);
    }
}

#[tokio::test]
async fn test_get_percent_encodes_the_id() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/calls/call%2F..%2Faccount"))
        .respond_with(ResponseTemplate::new(200).set_body_json(call_json("completed")))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.calls().get("call/../account").await;

    assert!(result.is_ok(), "{:?}", result.err());
}

#[tokio::test]
async fn test_get_not_found_maps_to_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/calls/{}", CALL_ID)))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "call_not_found",
            "message": "No call with that id is in this workspace."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.calls().get(CALL_ID).await;

    match result.unwrap_err() {
        Error::NotFound { message } => {
            assert_eq!(message, "No call with that id is in this workspace.")
        }
        other => panic!("expected NotFound, got {:?}", other),
    }
}

#[tokio::test]
async fn test_id_methods_require_an_id() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    assert!(matches!(
        client.calls().get("").await.unwrap_err(),
        Error::Validation { .. }
    ));
    assert!(matches!(
        client.calls().hangup(" ").await.unwrap_err(),
        Error::Validation { .. }
    ));
    assert!(matches!(
        client.calls().recording("").await.unwrap_err(),
        Error::Validation { .. }
    ));

    let requests = mock_server.received_requests().await.unwrap();
    assert!(requests.is_empty(), "no request should reach the server");
}

// ==================== hangup() Tests ====================

#[tokio::test]
async fn test_hangup_ringing_call_is_cancelled() {
    let mock_server = setup_mock_server().await;
    let mut body = call_json("cancelled");
    body["endedAt"] = json!("2026-09-12T14:03:20.000Z");
    body["billing"] = json!("settled");
    body["hangupClass"] = json!("caller_cancelled");
    Mock::given(method("POST"))
        .and(path(format!("/calls/{}/hangup", CALL_ID)))
        .and(header_exists("Idempotency-Key"))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let call = client
        .calls()
        .hangup(CALL_ID)
        .await
        .expect("hangup should succeed");

    assert_eq!(call.status, CallStatus::Cancelled);
    assert_eq!(call.hangup_class.as_deref(), Some("caller_cancelled"));
    assert_eq!(call.billing, CallBilling::Settled);
    assert_eq!(call.credits_charged, 0);

    let key = idempotency_key_of_request(&mock_server, 0)
        .await
        .expect("Idempotency-Key present");
    assert!(Regex::new(AUTO_KEY_PATTERN).unwrap().is_match(&key));
}

#[tokio::test]
async fn test_hangup_active_call_is_completed() {
    let mock_server = setup_mock_server().await;
    let mut body = call_json("completed");
    body["answeredAt"] = json!("2026-09-12T14:03:19.000Z");
    body["endedAt"] = json!("2026-09-12T14:04:40.000Z");
    body["durationSecs"] = json!(81);
    body["creditsCharged"] = json!(20);
    body["billing"] = json!("settled");
    body["hangupClass"] = json!("normal");
    Mock::given(method("POST"))
        .and(path(format!("/calls/{}/hangup", CALL_ID)))
        .and(header("Idempotency-Key", "hangup-once"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let call = client
        .calls()
        .hangup_with_options(
            CALL_ID,
            IdempotentRequestOptions::new().idempotency_key("hangup-once"),
        )
        .await
        .expect("hangup should succeed");

    assert_eq!(call.status, CallStatus::Completed);
    assert_eq!(call.hangup_class.as_deref(), Some("normal"));
    assert_eq!(call.duration_secs, 81);
    assert_eq!(call.credits_charged, 20);
}

#[tokio::test]
async fn test_hangup_ended_call_returns_it_unchanged() {
    let mock_server = setup_mock_server().await;
    let mut body = call_json("no_answer");
    body["endedAt"] = json!("2026-09-12T14:04:11.000Z");
    body["billing"] = json!("settled");
    body["hangupClass"] = json!("ring_timeout");
    Mock::given(method("POST"))
        .and(path(format!("/calls/{}/hangup", CALL_ID)))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let call = client
        .calls()
        .hangup(CALL_ID)
        .await
        .expect("hangup on an ended call is not an error");

    assert_eq!(call.status, CallStatus::NoAnswer);
    assert_eq!(call.hangup_class.as_deref(), Some("ring_timeout"));
}

// ==================== recording() Tests ====================

#[tokio::test]
async fn test_recording_ready_carries_signed_url() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/calls/{}/recording", CALL_ID)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "callId": CALL_ID,
            "status": "ready",
            "url": "https://media.example/recordings/call.ogg?sig=abc",
            "expiresAt": "2026-09-12T14:10:00.000Z",
            "contentType": "audio/ogg"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let recording = client
        .calls()
        .recording(CALL_ID)
        .await
        .expect("recording should succeed");

    assert_eq!(recording.call_id, CALL_ID);
    assert_eq!(recording.status, CallRecordingStatus::Ready);
    assert_eq!(
        recording.url.as_deref(),
        Some("https://media.example/recordings/call.ogg?sig=abc")
    );
    assert_eq!(
        recording.expires_at.as_deref(),
        Some("2026-09-12T14:10:00.000Z")
    );
    assert_eq!(recording.content_type.as_deref(), Some("audio/ogg"));
}

#[tokio::test]
async fn test_recording_none_has_null_url() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/calls/{}/recording", CALL_ID)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "callId": CALL_ID,
            "status": "none",
            "url": null,
            "expiresAt": null,
            "contentType": null
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let recording = client
        .calls()
        .recording(CALL_ID)
        .await
        .expect("recording should succeed");

    assert_eq!(recording.status, CallRecordingStatus::None);
    assert_eq!(recording.url, None);
    assert_eq!(recording.expires_at, None);
    assert_eq!(recording.content_type, None);
}

#[tokio::test]
async fn test_recording_still_recording_has_null_url() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/calls/{}/recording", CALL_ID)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "callId": CALL_ID,
            "status": "recording",
            "url": null,
            "expiresAt": null,
            "contentType": null
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let recording = client
        .calls()
        .recording(CALL_ID)
        .await
        .expect("recording should succeed");

    assert_eq!(recording.status, CallRecordingStatus::Recording);
    assert_eq!(recording.url, None);
}

// ==================== Webhook object Tests ====================

#[test]
fn test_call_completed_webhook_round_trips_billing_and_metadata() {
    let payload = json!({
        "id": "evt_call_1",
        "type": "call.completed",
        "api_version": "2024-01",
        "created": 1,
        "livemode": true,
        "data": {
            "object": {
                "id": CALL_ID,
                "object": "call",
                "kind": "pstn",
                "direction": "outbound",
                "status": "completed",
                "handled_by": "agent",
                "agent_id": AGENT_ID,
                "from": "+15555550188",
                "to": "+15555550123",
                "caller_name": "Front Desk",
                "callee_name": "+15555550123",
                "started_at": "2026-09-12T14:03:11.000Z",
                "answered_at": "2026-09-12T14:03:19.000Z",
                "ended_at": "2026-09-12T14:05:02.000Z",
                "duration_secs": 103,
                "credits_charged": 20,
                "billing": "settled",
                "hangup_class": "normal",
                "recording_status": "ready",
                "metadata": { "crmId": "lead_8812" },
                "organization_id": "org_1"
            }
        }
    })
    .to_string();
    let signature = Webhooks::generate_signature(&payload, "whsec_test", None);

    let event = Webhooks::parse_event(&payload, &signature, "whsec_test", None)
        .expect("call.completed should parse");

    assert_eq!(event.object["billing"], "settled");
    assert_eq!(event.object["metadata"]["crmId"], "lead_8812");
    assert_eq!(event.object["hangup_class"], "normal");
    assert_eq!(
        event.object.as_object().map(|o| o.len()),
        Some(21),
        "the call webhook object carries 21 keys including organization_id"
    );
}
