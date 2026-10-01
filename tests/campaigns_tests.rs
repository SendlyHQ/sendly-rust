mod common;

use common::{create_test_client, mock_strict_json_parser, setup_mock_server};
use sendly::{
    BatchStatus, CampaignStatus, CreateCampaignRequest, ListCampaignsOptions,
    ScheduleCampaignRequest,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

fn campaign_row(id: &str, status: &str) -> serde_json::Value {
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
        "batchId": "batch_1",
        "totalRecipients": 120,
        "estimatedCredits": 240,
        "sentCount": 118,
        "deliveredCount": 110,
        "failedCount": 2,
        "creditsUsed": 236,
        "creditsRefunded": 4,
        "createdAt": "2026-09-20T10:00:00.000Z",
        "updatedAt": "2026-09-25T10:00:00.000Z",
        "sentAt": "2026-09-25T09:00:00.000Z",
        "completedAt": "2026-09-25T09:05:00.000Z"
    })
}

fn public_campaign(id: &str, status: &str) -> serde_json::Value {
    let mut campaign = campaign_row(id, status);
    let row = campaign.as_object_mut().unwrap();
    row.insert("text".to_string(), json!("20% off this weekend"));
    row.insert("contact_list_ids".to_string(), json!(["list_1"]));
    row.insert("created_at".to_string(), json!("2026-09-20T10:00:00.000Z"));
    row.insert("updated_at".to_string(), json!("2026-09-25T10:00:00.000Z"));
    campaign
}

#[tokio::test]
async fn test_get_decodes_the_public_campaign_with_both_timestamp_spellings() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/campaigns/camp_1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(public_campaign("camp_1", "completed")),
        )
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let campaign = client
        .campaigns()
        .get("camp_1")
        .await
        .expect("get should decode");

    assert_eq!(campaign.text, "20% off this weekend");
    assert_eq!(campaign.contact_list_ids, vec!["list_1"]);
    assert_eq!(
        campaign.created_at.as_deref(),
        Some("2026-09-20T10:00:00.000Z")
    );
    assert_eq!(
        campaign.updated_at.as_deref(),
        Some("2026-09-25T10:00:00.000Z")
    );
    assert_eq!(
        campaign.completed_at.as_deref(),
        Some("2026-09-25T09:05:00.000Z")
    );
    assert_eq!(campaign.credits_used, Some(236.0));
}

#[tokio::test]
async fn test_get_still_reads_a_campaign_row_without_the_public_fields() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/campaigns/camp_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(campaign_row("camp_1", "draft")))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let campaign = client
        .campaigns()
        .get("camp_1")
        .await
        .expect("get should decode");

    assert_eq!(campaign.text, "20% off this weekend");
    assert_eq!(campaign.contact_list_ids, vec!["list_1"]);
    assert_eq!(
        campaign.created_at.as_deref(),
        Some("2026-09-20T10:00:00.000Z")
    );
}

#[tokio::test]
async fn test_list_decodes_public_campaigns() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/campaigns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "campaigns": [public_campaign("camp_1", "completed")],
            "total": 1,
            "limit": 50,
            "offset": 0
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let list = client
        .campaigns()
        .list(ListCampaignsOptions::new())
        .await
        .expect("list should decode");

    assert_eq!(list.campaigns.len(), 1);
    assert_eq!(
        list.campaigns[0].created_at.as_deref(),
        Some("2026-09-20T10:00:00.000Z")
    );
}

#[tokio::test]
async fn test_create_decodes_the_public_campaign() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/campaigns"))
        .respond_with(ResponseTemplate::new(201).set_body_json(public_campaign("camp_1", "draft")))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let campaign = client
        .campaigns()
        .create(CreateCampaignRequest::new(
            "Spring sale",
            "20% off this weekend",
            vec!["list_1".to_string()],
        ))
        .await
        .expect("create should decode");

    assert_eq!(campaign.id, "camp_1");
}

#[tokio::test]
async fn test_campaign_counts_read_total_recipients_and_sent_at() {
    let mut body = campaign_row("camp_1", "completed");
    body.as_object_mut()
        .unwrap()
        .insert("text".to_string(), json!("20% off this weekend"));
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/campaigns/camp_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let campaign = client
        .campaigns()
        .get("camp_1")
        .await
        .expect("get should decode");

    assert_eq!(campaign.recipient_count, 120);
    assert_eq!(
        campaign.started_at.as_deref(),
        Some("2026-09-25T09:00:00.000Z")
    );
    assert_eq!(campaign.sent_count, 118);
}

#[tokio::test]
async fn test_send_returns_the_batch_result() {
    let mock_server = setup_mock_server().await;
    mock_strict_json_parser().mount(&mock_server).await;
    Mock::given(method("POST"))
        .and(path("/campaigns/camp_1/send"))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "batchId": "batch_1",
            "status": "processing",
            "total": 3,
            "sent": 0,
            "failed": 0,
            "creditsUsed": 6,
            "creditsRefunded": 0,
            "messages": []
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let batch = client
        .campaigns()
        .send("camp_1")
        .await
        .expect("send should decode the batch result");

    assert_eq!(batch.batch_id, "batch_1");
    assert_eq!(batch.status, BatchStatus::Processing);
    assert_eq!(batch.credits_used, 6);
}

#[tokio::test]
async fn test_send_from_sends_the_number() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/campaigns/camp_1/send"))
        .and(body_json(json!({ "from": "+18005550199" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "batchId": "batch_2",
            "status": "processing",
            "total": 3,
            "sent": 0,
            "failed": 0,
            "creditsUsed": 6,
            "creditsRefunded": 0,
            "messages": []
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let batch = client
        .campaigns()
        .send_from("camp_1", "+18005550199")
        .await
        .expect("send_from should succeed");

    assert_eq!(batch.batch_id, "batch_2");
}

#[tokio::test]
async fn test_list_filters_by_completed() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/campaigns"))
        .and(query_param("status", "completed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "campaigns": [public_campaign("camp_1", "completed")],
            "total": 1,
            "limit": 50,
            "offset": 0
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let list = client
        .campaigns()
        .list(ListCampaignsOptions::new().status(CampaignStatus::Completed))
        .await
        .expect("list should succeed");

    assert_eq!(list.campaigns[0].status, "completed");
}

#[tokio::test]
async fn test_schedule_sends_scheduled_at_in_camel_case() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/campaigns/camp_1/schedule"))
        .and(body_json(json!({
            "scheduledAt": "2026-10-01T15:00:00Z",
            "timezone": "America/New_York"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(public_campaign("camp_1", "scheduled")),
        )
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/campaigns/camp_1/schedule"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "invalid_request",
            "message": "scheduledAt is required"
        })))
        .with_priority(10)
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    client
        .campaigns()
        .schedule(
            "camp_1",
            ScheduleCampaignRequest::new("2026-10-01T15:00:00Z").timezone("America/New_York"),
        )
        .await
        .expect("schedule should succeed");
}
