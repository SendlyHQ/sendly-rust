mod common;

use common::{create_test_client, setup_mock_server};
use sendly::{
    BackfillOptions, CreateWebhookRequest, RecoveryPreviewOptions, RedeliverOptions,
    UpdateWebhookRequest, WebhookMode,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

fn webhook_row() -> serde_json::Value {
    json!({
        "id": "whk_1",
        "url": "https://x.example",
        "events": ["message.delivered"],
        "mode": "all",
        "is_active": true,
        "failure_count": 0,
        "circuit_state": "closed",
        "api_version": null,
        "metadata": {},
        "created_at": "2026-09-20T10:00:00.000Z",
        "updated_at": "2026-09-25T10:00:00.000Z",
        "total_deliveries": 0,
        "successful_deliveries": 0,
        "success_rate": 0,
        "last_delivery_at": null
    })
}

#[tokio::test]
async fn test_list_decodes_the_bare_array_the_api_sends() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/webhooks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([webhook_row()])))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let webhooks = client.webhooks().list().await.expect("list should decode");

    assert_eq!(webhooks.len(), 1);
    assert_eq!(webhooks[0].id, "whk_1");
}

#[tokio::test]
async fn test_list_still_decodes_a_wrapped_list() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/webhooks"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "webhooks": [webhook_row()] })),
        )
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let webhooks = client.webhooks().list().await.expect("list should decode");

    assert_eq!(webhooks.len(), 1);
}

#[tokio::test]
async fn test_test_reads_status_code_and_response_time_from_the_delivery() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/webhooks/whk_1/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "message": "Test webhook delivered successfully in 87ms",
            "delivery": {
                "id": "del_test",
                "delivery_id": "del_test",
                "webhook_url": "https://x.example",
                "event_type": "webhook.test",
                "status": "delivered",
                "response_time": 87,
                "status_code": 200,
                "response_body": "ok",
                "delivered_at": "2026-09-25T10:00:00.000Z"
            }
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .webhooks()
        .test("whk_1")
        .await
        .expect("test should decode");

    assert!(result.success);
    assert_eq!(result.status_code, 200);
    assert_eq!(result.response_time_ms, 87);
    assert_eq!(
        result.message.as_deref(),
        Some("Test webhook delivered successfully in 87ms")
    );
    assert_eq!(result.delivery.unwrap()["id"], "del_test");
}

#[tokio::test]
async fn test_test_that_the_endpoint_refused_is_a_validation_error() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/webhooks/whk_1/test"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "success": false,
            "message": "Test webhook failed: HTTP 500"
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let error = client.webhooks().test("whk_1").await.unwrap_err();

    assert!(
        matches!(error, sendly::Error::Validation { .. }),
        "{error:?}"
    );
    assert_eq!(error.body().unwrap()["success"], false);
}

#[tokio::test]
async fn test_redeliver_sends_the_options() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/webhooks/whk_1/redeliver"))
        .and(body_json(
            json!({ "since": "2026-09-24T00:00:00Z", "limit": 10 }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "message": "Requeued 3 webhook deliveries",
            "run_id": "run_1",
            "requeued": 3,
            "skipped": 0,
            "truncated": false,
            "window_size": 3,
            "delivery_ids": ["del_1", "del_2", "del_3"],
            "since": "2026-09-24T00:00:00.000Z",
            "until": "2026-09-25T10:00:00.000Z",
            "limit": 10
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .webhooks()
        .redeliver(
            "whk_1",
            RedeliverOptions {
                since: Some("2026-09-24T00:00:00Z".to_string()),
                limit: Some(10),
                ..Default::default()
            },
        )
        .await
        .expect("redeliver should succeed");

    assert_eq!(result["requeued"], 3);
}

#[tokio::test]
async fn test_backfill_sends_the_options() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/webhooks/whk_1/backfill"))
        .and(body_json(json!({ "event_types": ["message.delivered"] })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "message": "Synthesized 1 missed webhook delivery",
            "run_id": "run_2",
            "synthesized": 1,
            "by_type": { "message.delivered": 1 },
            "truncated": false,
            "candidates_scanned": 1,
            "delivery_ids": ["del_4"],
            "since": "2026-09-24T10:00:00.000Z",
            "until": "2026-09-25T10:00:00.000Z"
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    client
        .webhooks()
        .backfill(
            "whk_1",
            BackfillOptions {
                event_types: Some(vec!["message.delivered".to_string()]),
                ..Default::default()
            },
        )
        .await
        .expect("backfill should succeed");
}

fn recovery_preview_body() -> serde_json::Value {
    json!({
        "circuitState": "open",
        "circuitOpenedAt": "2026-09-25T08:00:00.000Z",
        "failureCount": 12,
        "auditFailures": {
            "total": 3,
            "byStatus": { "failed": 2, "cancelled": 1 },
            "byEventType": { "message.delivered": 2, "message.opt_out": 1 }
        },
        "missingFromMessages": { "total": 4, "byEventType": { "message.sent": 4 } },
        "histogram": [{ "bucket": "2026-09-25T08:00:00.000Z", "missing": 4, "failed": 3 }],
        "samples": [{
            "id": "msg_1",
            "toMasked": "+1555***4567",
            "eventType": "message.delivered",
            "occurredAt": "2026-09-25T08:30:00.000Z"
        }],
        "windowSuggestion": {
            "since": "2026-09-25T08:00:00.000Z",
            "until": "2026-09-25T10:00:00.000Z"
        },
        "since": "2026-09-24T10:00:00.000Z",
        "until": "2026-09-25T10:00:00.000Z"
    })
}

#[tokio::test]
async fn test_recovery_preview_sends_the_window_and_decodes_the_camel_case_preview() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/webhooks/whk_1/recovery-preview"))
        .and(query_param("since", "2026-09-24T10:00:00.000Z"))
        .and(query_param("until", "2026-09-25T10:00:00.000Z"))
        .and(query_param(
            "event_types",
            "message.delivered,message.failed",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(recovery_preview_body()))
        .expect(1)
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let preview = client
        .webhooks()
        .recovery_preview(
            "whk_1",
            RecoveryPreviewOptions::new()
                .since("2026-09-24T10:00:00.000Z")
                .until("2026-09-25T10:00:00.000Z")
                .event_types(["message.delivered", "message.failed"]),
        )
        .await
        .expect("recovery_preview should decode");

    assert_eq!(preview.circuit_state, "open");
    assert_eq!(
        preview.circuit_opened_at.as_deref(),
        Some("2026-09-25T08:00:00.000Z")
    );
    assert_eq!(preview.failure_count, 12);
    assert_eq!(preview.audit_failures.total, 3);
    assert_eq!(preview.audit_failures.by_status["cancelled"], 1);
    assert_eq!(preview.audit_failures.by_event_type["message.opt_out"], 1);
    assert_eq!(
        preview.missing_from_messages.by_event_type["message.sent"],
        4
    );
    assert_eq!(preview.histogram[0].failed, 3);
    assert_eq!(preview.samples[0].to_masked, "+1555***4567");
    assert_eq!(preview.samples[0].occurred_at, "2026-09-25T08:30:00.000Z");
    assert_eq!(
        preview.window_suggestion.as_ref().map(|w| w.since.as_str()),
        Some("2026-09-25T08:00:00.000Z")
    );
    assert_eq!(preview.until, "2026-09-25T10:00:00.000Z");
}

#[tokio::test]
async fn test_recovery_preview_without_options_sends_no_query() {
    let mock_server = setup_mock_server().await;
    let mut body = recovery_preview_body();
    body["circuitOpenedAt"] = serde_json::Value::Null;
    body["windowSuggestion"] = serde_json::Value::Null;
    Mock::given(method("GET"))
        .and(path("/webhooks/whk_1/recovery-preview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let preview = client
        .webhooks()
        .recovery_preview("whk_1", RecoveryPreviewOptions::new())
        .await
        .expect("recovery_preview should decode");

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(requests[0].url.query(), None);
    assert!(preview.circuit_opened_at.is_none());
    assert!(preview.window_suggestion.is_none());
}

#[tokio::test]
async fn test_create_with_options_sends_description_mode_and_metadata() {
    let mock_server = setup_mock_server().await;
    let mut created = webhook_row();
    created["description"] = json!("Delivery receipts");
    created["mode"] = json!("test");
    created["metadata"] = json!({ "team": "billing", "priority": 2 });
    created["secret"] = json!("whsec_abc");
    Mock::given(method("POST"))
        .and(path("/webhooks"))
        .and(body_json(json!({
            "url": "https://x.example",
            "events": ["message.delivered", "message.failed"],
            "description": "Delivery receipts",
            "mode": "test",
            "metadata": { "team": "billing", "priority": 2 }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(created))
        .expect(1)
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let response = client
        .webhooks()
        .create_with_options(
            CreateWebhookRequest::new("https://x.example", ["message.delivered", "message.failed"])
                .description("Delivery receipts")
                .mode(WebhookMode::Test)
                .metadata_entry("team", "billing")
                .metadata_entry("priority", 2),
        )
        .await
        .expect("create_with_options should decode");

    let webhook = response.get_webhook().expect("the created webhook");
    assert_eq!(response.secret, "whsec_abc");
    assert_eq!(webhook.description.as_deref(), Some("Delivery receipts"));
    assert_eq!(webhook.mode, WebhookMode::Test);
    assert_eq!(
        webhook.metadata,
        Some(json!({ "team": "billing", "priority": 2 }))
    );
}

#[tokio::test]
async fn test_create_sends_only_url_and_events() {
    let mock_server = setup_mock_server().await;
    let mut created = webhook_row();
    created["secret"] = json!("whsec_abc");
    Mock::given(method("POST"))
        .and(path("/webhooks"))
        .and(body_json(json!({
            "url": "https://x.example",
            "events": ["message.delivered"]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(created))
        .expect(1)
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let response = client
        .webhooks()
        .create("https://x.example", vec!["message.delivered"])
        .await
        .expect("create should decode");

    let webhook = response.get_webhook().expect("the created webhook");
    assert_eq!(webhook.description, None);
    assert_eq!(webhook.metadata, Some(json!({})));
}

#[tokio::test]
async fn test_update_sends_description_mode_and_metadata() {
    let mock_server = setup_mock_server().await;
    let mut updated = webhook_row();
    updated["description"] = json!("Receipts");
    updated["mode"] = json!("live");
    updated["metadata"] = json!({ "team": "ops" });
    Mock::given(method("PATCH"))
        .and(path("/webhooks/whk_1"))
        .and(body_json(json!({
            "events": ["message.failed"],
            "description": "Receipts",
            "is_active": false,
            "mode": "live",
            "metadata": { "team": "ops" }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(updated))
        .expect(1)
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let mut metadata = std::collections::HashMap::new();
    metadata.insert("team".to_string(), json!("ops"));
    let webhook = client
        .webhooks()
        .update(
            "whk_1",
            UpdateWebhookRequest::new()
                .events(["message.failed"])
                .description("Receipts")
                .is_active(false)
                .mode(WebhookMode::Live)
                .metadata(metadata),
        )
        .await
        .expect("update should decode");

    assert_eq!(webhook.description.as_deref(), Some("Receipts"));
    assert_eq!(webhook.mode, WebhookMode::Live);
    assert_eq!(webhook.metadata, Some(json!({ "team": "ops" })));
}

#[tokio::test]
async fn test_update_clears_description_and_metadata_with_null() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/webhooks/whk_1"))
        .and(body_json(json!({
            "url": "https://y.example",
            "description": null,
            "metadata": null
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(webhook_row()))
        .expect(1)
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let webhook = client
        .webhooks()
        .update(
            "whk_1",
            UpdateWebhookRequest::new()
                .url("https://y.example")
                .clear_description()
                .clear_metadata(),
        )
        .await
        .expect("update should decode");

    assert_eq!(webhook.description, None);
}

#[tokio::test]
async fn test_get_decodes_the_failure_and_circuit_times() {
    let mock_server = setup_mock_server().await;
    let mut row = webhook_row();
    row["circuit_state"] = json!("open");
    row["last_failure_at"] = json!("2026-09-25T08:30:00.000Z");
    row["circuit_opened_at"] = json!("2026-09-25T08:00:00.000Z");
    row["description"] = json!("Receipts");
    Mock::given(method("GET"))
        .and(path("/webhooks/whk_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(row))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let webhook = client
        .webhooks()
        .get("whk_1")
        .await
        .expect("get should decode");

    assert!(webhook.is_circuit_open());
    assert_eq!(
        webhook.last_failure_at.as_deref(),
        Some("2026-09-25T08:30:00.000Z")
    );
    assert_eq!(
        webhook.circuit_opened_at.as_deref(),
        Some("2026-09-25T08:00:00.000Z")
    );
    assert_eq!(webhook.description.as_deref(), Some("Receipts"));
}
