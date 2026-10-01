mod common;

use common::{create_test_client, setup_mock_server};
use sendly::{InheritVerificationOptions, ListWorkspacesOptions, ProvisionWorkspaceRequest};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

fn workspace_summary() -> serde_json::Value {
    json!({
        "totalCredits": 250,
        "pendingVerifications": [],
        "verificationBreakdown": { "verified": 1, "pending": 0, "submitted": 0, "rejected": 0, "unverified": 0 }
    })
}

#[tokio::test]
async fn test_workspaces_list_decodes_the_paginated_list() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/enterprise/workspaces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "workspaces": [{
                "id": "ws_1",
                "name": "Acme East",
                "slug": "acme-east",
                "status": "active",
                "suspendedAt": null,
                "suspendReason": null,
                "verificationStatus": "verified",
                "rejectionReason": null,
                "verificationType": "toll_free",
                "tollFreeNumber": "+18005550199",
                "creditBalance": 250,
                "keyCount": 2,
                "messages30d": 40,
                "delivered30d": 38,
                "failed30d": 2,
                "monthlyMessageQuota": null,
                "messagesThisMonth": 12,
                "quotaResetAt": null,
                "createdAt": "2026-09-01T10:00:00.000Z",
                "tags": [{ "id": "tag_1", "name": "east", "color": "#6b7280" }]
            }],
            "pagination": { "total": 1, "limit": 50, "page": 1, "totalPages": 1, "hasMore": false },
            "summary": workspace_summary(),
            "maxWorkspaces": 10,
            "workspacesUsed": 1
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let list = client
        .enterprise()
        .workspaces()
        .list()
        .await
        .expect("workspace list should decode");

    assert_eq!(list.workspaces.len(), 1);
    let workspace = &list.workspaces[0];
    assert_eq!(workspace.id, "ws_1");
    assert_eq!(workspace.credit_balance, 250);
    assert_eq!(workspace.verification_status.as_deref(), Some("verified"));
    assert_eq!(workspace.key_count, 2);
    assert_eq!(workspace.messages_30d, 40);
    assert_eq!(list.pagination.as_ref().unwrap().total, 1);
    assert_eq!(list.max_workspaces, 10);
    assert_eq!(list.workspaces_used, 1);
}

#[tokio::test]
async fn test_workspaces_list_with_options_sends_the_filters() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/enterprise/workspaces"))
        .and(query_param("page", "2"))
        .and(query_param("limit", "10"))
        .and(query_param("search", "acme"))
        .and(query_param("verification", "verified"))
        .and(query_param("sort", "credits_desc"))
        .and(query_param("tags", "tag_1,tag_2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "workspaces": [],
            "pagination": { "total": 0, "limit": 10, "page": 2, "totalPages": 0, "hasMore": false },
            "summary": workspace_summary(),
            "maxWorkspaces": 10,
            "workspacesUsed": 1
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let list = client
        .enterprise()
        .workspaces()
        .list_with_options(
            ListWorkspacesOptions::new()
                .page(2)
                .limit(10)
                .search("acme")
                .verification("verified")
                .sort("credits_desc")
                .tags(vec!["tag_1", "tag_2"]),
        )
        .await
        .expect("filtered list should decode");

    assert!(list.workspaces.is_empty());
}

#[tokio::test]
async fn test_workspaces_list_decodes_the_compact_list() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/enterprise/workspaces"))
        .and(query_param("compact", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "workspaces": [{ "id": "ws_1", "name": "Acme East", "creditBalance": 250 }],
            "summary": workspace_summary(),
            "maxWorkspaces": 10,
            "workspacesUsed": 1
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let list = client
        .enterprise()
        .workspaces()
        .list_with_options(ListWorkspacesOptions::new().compact(true))
        .await
        .expect("compact workspace list should decode");

    assert_eq!(list.workspaces[0].credit_balance, 250);
    assert!(list.pagination.is_none());
}

#[tokio::test]
async fn test_webhooks_set_keeps_the_signing_secret() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/enterprise/webhooks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "url": "https://hooks.example.com/enterprise",
            "events": null,
            "workspaces": null,
            "signingSecret": "abc"
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let webhook = client
        .enterprise()
        .webhooks()
        .set("https://hooks.example.com/enterprise")
        .await
        .expect("set should decode");

    assert_eq!(webhook.signing_secret.as_deref(), Some("abc"));
    assert_eq!(
        webhook.url.as_deref(),
        Some("https://hooks.example.com/enterprise")
    );
    assert_eq!(webhook.events, None);
}

#[tokio::test]
async fn test_webhooks_rotate_secret_keeps_the_new_secret() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/enterprise/webhooks/rotate-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "secret": "def",
            "rotated_at": "2026-09-25T10:00:00.000Z",
            "message": "Webhook signing secret rotated. Save this secret - it won't be shown again."
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let webhook = client
        .enterprise()
        .webhooks()
        .rotate_secret()
        .await
        .expect("rotate_secret should decode");

    assert_eq!(webhook.signing_secret.as_deref(), Some("def"));
    assert_eq!(
        webhook.rotated_at.as_deref(),
        Some("2026-09-25T10:00:00.000Z")
    );
}

#[tokio::test]
async fn test_webhooks_test_keeps_a_failed_request() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/enterprise/webhooks/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": false,
            "error": "Request timed out (10s)"
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .enterprise()
        .webhooks()
        .test()
        .await
        .expect("test should decode");

    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("Request timed out (10s)"));
}

#[tokio::test]
async fn test_analytics_credits_reads_the_totals() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/enterprise/analytics/credits"))
        .and(query_param("period", "30d"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "period": "30d",
            "totalBalance": 5000,
            "totalLifetime": 8000,
            "totalUsed": 3000,
            "workspaceCount": 4
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let credits = client
        .enterprise()
        .analytics()
        .credits(Some(sendly::AnalyticsPeriod::new().period("30d")))
        .await
        .expect("credits analytics should decode");

    assert_eq!(credits.period, "30d");
    assert_eq!(credits.total_balance, 5000);
    assert_eq!(credits.total_lifetime, 8000);
    assert_eq!(credits.total_used, 3000);
    assert_eq!(credits.workspace_count, 4);
}

#[tokio::test]
async fn test_provision_keeps_the_generated_pages_and_urls() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/enterprise/workspaces/provision"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "workspace": { "id": "ws_1", "name": "Acme East", "slug": "acme-east" },
            "verification": { "id": "bv_1", "status": "pending" },
            "credits": { "transferred": 500 },
            "key": { "id": "key_1", "key": "sk_live_v1_new" },
            "webhook": { "id": "whk_1", "url": "https://hooks.example.com/acme" },
            "optInPage": { "id": "page_1", "slug": "acme-east-x1y2", "url": "https://sendly.live/opt-in/acme-east-x1y2" },
            "legalPages": {
                "privacyUrl": "https://sendly.live/legal/acme-east-ab12-privacy",
                "termsUrl": "https://sendly.live/legal/acme-east-ab12-terms",
                "privacyPageId": "lp_1",
                "termsPageId": "lp_2"
            },
            "businessPage": { "error": "Business page generation failed" },
            "apiBaseUrl": "https://sendly.live",
            "dashboardUrl": "https://sendly.live/enterprise/workspaces/ws_1"
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let provisioned = client
        .enterprise()
        .provision(ProvisionWorkspaceRequest::new("Acme East").generate_opt_in_page(true))
        .await
        .expect("provision should decode");

    let opt_in_page = provisioned.opt_in_page.unwrap();
    assert_eq!(
        opt_in_page.url.as_deref(),
        Some("https://sendly.live/opt-in/acme-east-x1y2")
    );
    assert_eq!(opt_in_page.error, None);
    let legal_pages = provisioned.legal_pages.unwrap();
    assert_eq!(
        legal_pages.terms_url.as_deref(),
        Some("https://sendly.live/legal/acme-east-ab12-terms")
    );
    assert_eq!(
        provisioned.business_page.unwrap().error.as_deref(),
        Some("Business page generation failed")
    );
    assert_eq!(
        provisioned.api_base_url.as_deref(),
        Some("https://sendly.live")
    );
    assert_eq!(
        provisioned.dashboard_url.as_deref(),
        Some("https://sendly.live/enterprise/workspaces/ws_1")
    );
}

#[tokio::test]
async fn test_inherit_verification_can_order_a_new_number() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/enterprise/workspaces/ws_1/verification/inherit"))
        .and(body_json(
            json!({ "sourceWorkspaceId": "org_src", "purchaseNewNumber": true }),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "verificationId": "bv_new",
            "status": "pending",
            "type": "toll_free",
            "tollFreeNumber": "+18005550142",
            "inheritedFrom": "org_src",
            "newNumber": true
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let inherited = client
        .enterprise()
        .workspaces()
        .inherit_verification_with_options(
            "ws_1",
            InheritVerificationOptions::new("org_src").purchase_new_number(true),
        )
        .await
        .expect("inherit should decode");

    assert!(inherited.new_number);
    assert_eq!(inherited.toll_free_number.as_deref(), Some("+18005550142"));
}

#[tokio::test]
async fn test_inherit_verification_leaves_out_the_flag_by_default() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/enterprise/workspaces/ws_1/verification/inherit"))
        .and(body_json(json!({ "sourceWorkspaceId": "org_src" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "verificationId": "bv_shared",
            "status": "verified",
            "type": "toll_free",
            "tollFreeNumber": "+18005550199",
            "inheritedFrom": "org_src"
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let shared = client
        .enterprise()
        .workspaces()
        .inherit_verification_with_options("ws_1", InheritVerificationOptions::new("org_src"))
        .await
        .expect("inherit should decode");
    let legacy = client
        .enterprise()
        .workspaces()
        .inherit_verification("ws_1", "org_src")
        .await
        .expect("inherit should decode");

    assert!(!shared.new_number);
    assert_eq!(legacy.verification_id.as_deref(), Some("bv_shared"));
}

#[tokio::test]
async fn test_workspace_test_webhook_reads_the_status_code_from_the_delivery() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/enterprise/workspaces/ws_1/webhooks/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": false,
            "message": "Test webhook failed: HTTP 503",
            "delivery": {
                "id": "del_test",
                "delivery_id": "del_test",
                "webhook_url": "https://hooks.example.com/acme",
                "event_type": "webhook.test",
                "status": "failed",
                "response_time": 120,
                "status_code": 503,
                "response_body": "unavailable",
                "error": "HTTP 503"
            }
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .enterprise()
        .workspaces()
        .test_webhook("ws_1")
        .await
        .expect("test_webhook should decode");

    assert!(!result.success);
    assert_eq!(result.status_code, Some(503));
}

#[tokio::test]
async fn test_workspaces_list_never_sends_a_zero_limit() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/enterprise/workspaces"))
        .and(query_param("limit", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "workspaces": [],
            "pagination": { "total": 3, "limit": 0, "page": 1, "totalPages": null, "hasMore": false },
            "summary": workspace_summary(),
            "maxWorkspaces": 10,
            "workspacesUsed": 3
        })))
        .with_priority(1)
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/enterprise/workspaces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "workspaces": [],
            "pagination": { "total": 3, "limit": 1, "page": 1, "totalPages": 3, "hasMore": true },
            "summary": workspace_summary(),
            "maxWorkspaces": 10,
            "workspacesUsed": 3
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());
    let workspaces = client.enterprise().workspaces();

    workspaces
        .list_with_options(ListWorkspacesOptions::new().limit(0))
        .await
        .expect("a zero limit from the builder should not reach the API");
    workspaces
        .list_with_options(ListWorkspacesOptions {
            limit: Some(0),
            ..Default::default()
        })
        .await
        .expect("a zero limit set directly should not reach the API");

    let queries: Vec<Option<String>> = mock_server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .map(|r| r.url.query().map(str::to_string))
        .collect();
    assert_eq!(queries, vec![Some("limit=1".to_string()), None]);
}
