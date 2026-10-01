mod common;

use common::{create_test_client, setup_mock_server};
use sendly::{BackfillOptions, RedeliverOptions};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
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
