mod common;

use common::{create_test_client, mock_strict_json_parser, setup_mock_server};
use sendly::Error;
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn server_expecting_empty_object(
    route: &str,
    status: u16,
    body: serde_json::Value,
) -> MockServer {
    let mock_server = setup_mock_server().await;
    mock_strict_json_parser().mount(&mock_server).await;
    Mock::given(method("POST"))
        .and(path(route))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .mount(&mock_server)
        .await;
    mock_server
}

fn conversation_row(status: &str, unread_count: i32) -> serde_json::Value {
    json!({
        "id": "conv_1",
        "userId": "user_1",
        "organizationId": "org_1",
        "phoneNumber": "+15555550100",
        "status": status,
        "unreadCount": unread_count,
        "messageCount": 3,
        "lastMessageText": "Thanks!",
        "lastMessageAt": "2026-09-25T10:00:00.000Z",
        "lastMessageDirection": "inbound",
        "channel": "sms",
        "metadata": {},
        "tags": [],
        "contactId": null,
        "createdAt": "2026-09-20T10:00:00.000Z",
        "updatedAt": "2026-09-25T10:00:00.000Z"
    })
}

fn public_campaign(id: &str, status: &str) -> serde_json::Value {
    json!({
        "id": id,
        "userId": "user_1",
        "organizationId": "org_1",
        "name": "Spring sale",
        "status": status,
        "messageText": "20% off this weekend",
        "fromSender": null,
        "targetType": "contact_list",
        "targetListId": "list_1",
        "manualRecipients": null,
        "excludeOptedOut": true,
        "sendNow": false,
        "scheduledAt": null,
        "timezone": "America/New_York",
        "batchId": null,
        "totalRecipients": 3,
        "estimatedCredits": 6,
        "sentCount": 0,
        "deliveredCount": 0,
        "failedCount": 0,
        "creditsUsed": 0,
        "creditsRefunded": 0,
        "createdAt": "2026-09-20T10:00:00.000Z",
        "updatedAt": "2026-09-25T10:00:00.000Z",
        "sentAt": null,
        "completedAt": null,
        "text": "20% off this weekend",
        "contact_list_ids": ["list_1"],
        "created_at": "2026-09-20T10:00:00.000Z",
        "updated_at": "2026-09-25T10:00:00.000Z"
    })
}

fn formatted_template(id: &str, name: &str) -> serde_json::Value {
    json!({
        "id": id,
        "name": name,
        "text": "Your code is {{code}}",
        "variables": [{ "key": "code", "type": "string" }],
        "is_preset": false,
        "status": "draft",
        "version": 1,
        "published_at": null,
        "created_at": "2026-09-25T10:00:00.000Z",
        "updated_at": "2026-09-25T10:00:00.000Z"
    })
}

fn formatted_delivery() -> serde_json::Value {
    json!({
        "id": "del_1",
        "webhook_id": "whk_1",
        "event_id": "evt_1",
        "event_type": "message.delivered",
        "status": "pending",
        "success": false,
        "response_status_code": null,
        "http_status": 0,
        "response_time": null,
        "response_time_ms": 0,
        "response_body": null,
        "error_message": null,
        "error_code": null,
        "attempt_number": 2,
        "max_attempts": 6,
        "next_retry_at": null,
        "created_at": "2026-09-25T10:00:00.000Z",
        "delivered_at": null
    })
}

fn webhook_row() -> serde_json::Value {
    json!({
        "id": "whk_1",
        "url": "https://hooks.example.com/sendly",
        "events": ["message.delivered"],
        "mode": "all",
        "is_active": true,
        "failure_count": 0,
        "circuit_state": "closed",
        "api_version": "2024-01",
        "metadata": {},
        "created_at": "2026-09-20T10:00:00.000Z",
        "updated_at": "2026-09-25T10:00:00.000Z",
        "total_deliveries": 4,
        "successful_deliveries": 4,
        "success_rate": 100,
        "last_delivery_at": null
    })
}

fn test_webhook_result() -> serde_json::Value {
    json!({
        "success": true,
        "message": "Test webhook delivered successfully in 87ms",
        "delivery": {
            "id": "del_test",
            "delivery_id": "del_test",
            "webhook_url": "https://hooks.example.com/sendly",
            "event_type": "webhook.test",
            "status": "delivered",
            "response_time": 87,
            "status_code": 200,
            "response_body": "ok",
            "delivered_at": "2026-09-25T10:00:00.000Z"
        }
    })
}

#[tokio::test]
async fn test_conversations_close_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/conversations/conv_1/close",
        200,
        conversation_row("closed", 0),
    )
    .await;
    let client = create_test_client(&server.uri());

    let conversation = client
        .conversations()
        .close("conv_1")
        .await
        .expect("close should succeed");

    assert_eq!(conversation.id, "conv_1");
}

#[tokio::test]
async fn test_conversations_reopen_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/conversations/conv_1/reopen",
        200,
        conversation_row("active", 0),
    )
    .await;
    let client = create_test_client(&server.uri());

    let conversation = client
        .conversations()
        .reopen("conv_1")
        .await
        .expect("reopen should succeed");

    assert_eq!(conversation.id, "conv_1");
}

#[tokio::test]
async fn test_conversations_mark_read_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/conversations/conv_1/mark-read",
        200,
        conversation_row("active", 0),
    )
    .await;
    let client = create_test_client(&server.uri());

    let conversation = client
        .conversations()
        .mark_read("conv_1")
        .await
        .expect("mark_read should succeed");

    assert_eq!(conversation.unread_count, 0);
}

#[tokio::test]
async fn test_drafts_approve_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/drafts/d_1/approve",
        200,
        json!({
            "id": "d_1",
            "userId": "user_1",
            "organizationId": "org_1",
            "conversationId": "c_1",
            "text": "hi",
            "mediaUrls": [],
            "metadata": {},
            "status": "approved",
            "source": "ai",
            "createdBy": null,
            "reviewedBy": "user_1",
            "reviewedAt": "2026-09-25T10:00:00.000Z",
            "rejectionReason": null,
            "messageId": "msg_1",
            "createdAt": "2026-09-25T09:00:00.000Z",
            "updatedAt": "2026-09-25T10:00:00.000Z",
            "message": {
                "id": "msg_1",
                "to": "+15555550100",
                "from": "+18005550199",
                "text": "hi",
                "status": "queued",
                "direction": "outbound",
                "error": null,
                "segments": 1,
                "creditsUsed": 2,
                "senderType": "number_pool",
                "createdAt": "2026-09-25T10:00:00.000Z",
                "metadata": {},
                "senderNote": "Message will be sent from a toll-free number in your number pool."
            }
        }),
    )
    .await;
    let client = create_test_client(&server.uri());

    let draft = client
        .drafts()
        .approve("d_1")
        .await
        .expect("approve should succeed");

    assert_eq!(draft.status, sendly::DraftStatus::Approved);
    assert_eq!(draft.message_id.as_deref(), Some("msg_1"));
}

#[tokio::test]
async fn test_campaigns_send_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/campaigns/camp_1/send",
        200,
        json!({
            "batchId": "batch_1",
            "status": "processing",
            "total": 3,
            "sent": 0,
            "failed": 0,
            "creditsUsed": 6,
            "creditsRefunded": 0,
            "messages": []
        }),
    )
    .await;
    let client = create_test_client(&server.uri());

    let batch = client
        .campaigns()
        .send("camp_1")
        .await
        .expect("send should succeed");

    assert_eq!(batch.batch_id, "batch_1");
}

#[tokio::test]
async fn test_campaigns_cancel_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/campaigns/camp_1/cancel",
        200,
        public_campaign("camp_1", "cancelled"),
    )
    .await;
    let client = create_test_client(&server.uri());

    let campaign = client
        .campaigns()
        .cancel("camp_1")
        .await
        .expect("cancel should succeed");

    assert_eq!(campaign.status, "cancelled");
}

#[tokio::test]
async fn test_campaigns_clone_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/campaigns/camp_1/clone",
        201,
        public_campaign("camp_2", "draft"),
    )
    .await;
    let client = create_test_client(&server.uri());

    let campaign = client
        .campaigns()
        .clone("camp_1")
        .await
        .expect("clone should succeed");

    assert_eq!(campaign.id, "camp_2");
}

#[tokio::test]
async fn test_templates_clone_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/templates/tpl_1/clone",
        201,
        formatted_template("tpl_2", "OTP (Copy)"),
    )
    .await;
    let client = create_test_client(&server.uri());

    let template = client
        .templates()
        .clone("tpl_1")
        .await
        .expect("clone should succeed");

    assert_eq!(template.id, "tpl_2");
}

#[tokio::test]
#[allow(deprecated)]
async fn test_templates_unpublish_reaches_the_router_and_gets_its_404() {
    let server = server_expecting_empty_object(
        "/verify/templates/tpl_1/unpublish",
        404,
        json!({ "error": "Not found" }),
    )
    .await;
    let client = create_test_client(&server.uri());

    let error = client.templates().unpublish("tpl_1").await.unwrap_err();

    assert!(matches!(error, Error::NotFound { .. }), "got {error:?}");
}

#[tokio::test]
async fn test_verify_resend_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/verify/ver_1/resend",
        200,
        json!({
            "id": "ver_1",
            "status": "pending",
            "phone": "+15555550100",
            "expires_at": "2026-09-25T10:05:00.000Z",
            "sandbox": false,
            "message": "OTP resent successfully"
        }),
    )
    .await;
    let client = create_test_client(&server.uri());

    let verification = client
        .verify()
        .resend("ver_1")
        .await
        .expect("resend should succeed");

    assert_eq!(verification.id, "ver_1");
}

#[tokio::test]
async fn test_webhooks_test_sends_empty_object() {
    let server =
        server_expecting_empty_object("/webhooks/whk_1/test", 200, test_webhook_result()).await;
    let client = create_test_client(&server.uri());

    let result = client
        .webhooks()
        .test("whk_1")
        .await
        .expect("test should succeed");

    assert!(result.success);
    assert_eq!(result.status_code, 200);
}

#[tokio::test]
async fn test_webhooks_reset_circuit_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/webhooks/whk_1/reset-circuit",
        200,
        json!({ "message": "Circuit breaker reset", "webhook": webhook_row() }),
    )
    .await;
    let client = create_test_client(&server.uri());

    let result = client
        .webhooks()
        .reset_circuit("whk_1")
        .await
        .expect("reset_circuit should succeed");

    assert_eq!(result["message"], "Circuit breaker reset");
}

#[tokio::test]
async fn test_webhooks_rotate_secret_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/webhooks/whk_1/rotate-secret",
        200,
        json!({
            "success": true,
            "id": "whk_1",
            "secret": "whsec_new",
            "new_secret": "whsec_new",
            "new_secret_version": 2,
            "grace_period_hours": 24,
            "rotated_at": "2026-09-25T10:00:00.000Z",
            "message": "Webhook secret rotated successfully. Save this secret - it won't be shown again."
        }),
    )
    .await;
    let client = create_test_client(&server.uri());

    let rotation = client
        .webhooks()
        .rotate_secret("whk_1")
        .await
        .expect("rotate_secret should succeed");

    assert_eq!(rotation.secret, "whsec_new");
}

#[tokio::test]
async fn test_webhooks_retry_delivery_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/webhooks/whk_1/deliveries/del_1/retry",
        200,
        json!({ "delivery": formatted_delivery(), "requeued": true }),
    )
    .await;
    let client = create_test_client(&server.uri());

    let delivery = client
        .webhooks()
        .retry_delivery("whk_1", "del_1")
        .await
        .expect("retry_delivery should succeed");

    assert_eq!(delivery.id, "del_1");
}

#[tokio::test]
async fn test_enterprise_workspace_test_webhook_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/enterprise/workspaces/ws_1/webhooks/test",
        200,
        test_webhook_result(),
    )
    .await;
    let client = create_test_client(&server.uri());

    let result = client
        .enterprise()
        .workspaces()
        .test_webhook("ws_1")
        .await
        .expect("test_webhook should succeed");

    assert!(result.success);
}

#[tokio::test]
async fn test_enterprise_workspace_resume_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/enterprise/workspaces/ws_1/resume",
        200,
        json!({ "id": "ws_1", "status": "active" }),
    )
    .await;
    let client = create_test_client(&server.uri());

    let result = client
        .enterprise()
        .workspaces()
        .resume("ws_1")
        .await
        .expect("resume should succeed");

    assert_eq!(result.status, "active");
}

#[tokio::test]
async fn test_enterprise_webhooks_test_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/enterprise/webhooks/test",
        200,
        json!({ "success": true, "statusCode": 200, "statusText": "OK" }),
    )
    .await;
    let client = create_test_client(&server.uri());

    let result = client
        .enterprise()
        .webhooks()
        .test()
        .await
        .expect("test should succeed");

    assert_eq!(result.status_code, Some(200));
}

#[tokio::test]
async fn test_enterprise_webhooks_rotate_secret_sends_empty_object() {
    let server = server_expecting_empty_object(
        "/enterprise/webhooks/rotate-secret",
        200,
        json!({
            "success": true,
            "secret": "0f3c9a",
            "rotated_at": "2026-09-25T10:00:00.000Z",
            "message": "Webhook signing secret rotated. Save this secret - it won't be shown again."
        }),
    )
    .await;
    let client = create_test_client(&server.uri());

    let webhook = client
        .enterprise()
        .webhooks()
        .rotate_secret()
        .await
        .expect("rotate_secret should succeed");

    assert_eq!(webhook.signing_secret.as_deref(), Some("0f3c9a"));
}
