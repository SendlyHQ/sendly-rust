mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use sendly::{CreateApiKeyRequest, ListTransactionsOptions, TransactionType};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn test_transactions_uses_credits_transactions_path() {
    let mock_server = setup_mock_server().await;
    // Only the real path is mounted. The resource used to call
    // /account/transactions, which has no registration, so a regression
    // misses this mock and fails with a 404 instead of passing quietly.
    Mock::given(method("GET"))
        .and(path("/credits/transactions"))
        .and(query_param("limit", "5"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "transactions": [
                {
                    "id": "ctx_abc123",
                    "amount": -2,
                    "balance_after": 98,
                    "type": "usage",
                    "description": "SMS send",
                    "created_at": "2026-08-25T10:00:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .account()
        .transactions(Some(ListTransactionsOptions::new().limit(5)))
        .await
        .expect("transactions should succeed");

    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].id, "ctx_abc123");
    assert_eq!(result.data[0].transaction_type, TransactionType::Usage);
    assert_eq!(result.data[0].balance_after, 98);
    assert_eq!(
        result.data[0].created_at.as_deref(),
        Some("2026-08-25T10:00:00Z")
    );
}

#[tokio::test]
async fn test_transactions_empty_history_is_not_an_error() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/credits/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "transactions": [] })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .account()
        .transactions(None)
        .await
        .expect("empty history should decode");

    assert!(result.data.is_empty());
}

#[tokio::test]
async fn test_credits_reads_reserved_balance() {
    for billing_mode in ["prepaid", "pooled"] {
        let mock_server = setup_mock_server().await;
        Mock::given(method("GET"))
            .and(path("/account/credits"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "balance": 500,
                "reservedBalance": 40,
                "availableBalance": 460,
                "billingMode": billing_mode,
                "recentTransactions": [
                    { "id": "ctx_1", "amount": -2, "type": "usage", "description": "SMS send", "createdAt": "2026-09-25T10:00:00.000Z" }
                ]
            })))
            .mount(&mock_server)
            .await;
        let client = create_test_client(&mock_server.uri());

        let credits = client
            .account()
            .credits()
            .await
            .expect("credits should decode");

        assert_eq!(credits.balance, 500);
        assert_eq!(credits.reserved_credits, 40, "{billing_mode}");
        assert_eq!(credits.available_balance, 460);
        assert_eq!(credits.billing_mode.as_deref(), Some(billing_mode));
    }
}

#[tokio::test]
async fn test_transactions_keep_every_type_the_ledger_records() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/credits/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "transactions": [
                { "id": "ctx_1", "amount": -100, "balance_after": 900, "type": "transfer", "description": "Transfer to workspace", "created_at": "2026-09-25T10:00:00.000Z" },
                { "id": "ctx_2", "amount": 500, "balance_after": 1000, "type": "admin_grant", "description": "Admin adjustment: goodwill", "created_at": "2026-09-24T10:00:00.000Z" },
                { "id": "ctx_3", "amount": 1000, "balance_after": 500, "type": "admin_seed", "description": null, "created_at": "2026-09-23T10:00:00.000Z" },
                { "id": "ctx_4", "amount": 5, "balance_after": 505, "type": "brand_new", "description": null, "created_at": "2026-09-22T10:00:00.000Z" }
            ]
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let list = client
        .account()
        .transactions(None)
        .await
        .expect("every ledger type must decode");

    let types: Vec<TransactionType> = list
        .data
        .iter()
        .map(|t| t.transaction_type.clone())
        .collect();
    assert_eq!(
        types,
        vec![
            TransactionType::Transfer,
            TransactionType::AdminGrant,
            TransactionType::AdminSeed,
            TransactionType::Unknown
        ]
    );
    assert_eq!(list.data[0].balance_after, 900);
    assert_eq!(list.data[2].description, None);
}

#[tokio::test]
async fn test_transactions_filter_by_the_new_types_and_never_by_unknown() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/credits/transactions"))
        .and(query_param("type", "admin_grant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "transactions": [] })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    client
        .account()
        .transactions(Some(
            ListTransactionsOptions::new().transaction_type(TransactionType::AdminGrant),
        ))
        .await
        .expect("filtered transactions should succeed");
    let unfiltered = ListTransactionsOptions::new().transaction_type(TransactionType::Unknown);
    let _ = client.account().transactions(Some(unfiltered)).await;

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(requests[0].url.query(), Some("type=admin_grant"));
    assert_eq!(requests[1].url.query(), None);
}

fn account_body(verification: serde_json::Value, key_type: &str) -> serde_json::Value {
    json!({
        "user": { "id": "user_1", "email": "ops@example.com", "createdAt": "2026-01-02T03:04:05.000Z" },
        "organization": { "id": "org_1", "name": "Acme", "isPersonal": false },
        "credits": { "balance": "0", "reservedBalance": "0" },
        "verification": verification,
        "apiKey": {
            "id": "key_1",
            "name": "Production",
            "type": key_type,
            "scopes": ["sms:send"],
            "createdAt": "2026-01-02T03:04:05.000Z",
            "lastUsedAt": null
        },
        "limits": { "messagesPerMinute": 60, "messagesPerDay": if key_type == "test" { 100 } else { 10000 } }
    })
}

#[tokio::test]
async fn test_get_reads_the_account_the_api_sends() {
    let verified = json!({
        "status": "verified",
        "type": "toll_free",
        "region": "US",
        "submittedAt": "2026-01-03T00:00:00.000Z",
        "updatedAt": "2026-01-05T00:00:00.000Z"
    });
    for (verification, expect_unverified) in [(serde_json::Value::Null, true), (verified, false)] {
        let mock_server = setup_mock_server().await;
        Mock::given(method("GET"))
            .and(path("/account"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(account_body(verification, "live")),
            )
            .mount(&mock_server)
            .await;
        let client = create_test_client(&mock_server.uri());

        let account = client.account().get().await.expect("account should decode");

        assert_eq!(account.id, "user_1");
        assert_eq!(account.email, "ops@example.com");
        assert_eq!(
            account.created_at.as_deref(),
            Some("2026-01-02T03:04:05.000Z")
        );
        assert_eq!(account.limits.messages_per_day, 10000);
        assert_eq!(account.limits.messages_per_minute, Some(60));
        let organization = account.organization.as_ref().unwrap();
        assert_eq!(organization.id, "org_1");
        assert_eq!(organization.name.as_deref(), Some("Acme"));
        assert!(!organization.is_personal);
        match &account.business_verification {
            Some(verification) => {
                assert_eq!(verification.status.as_deref(), Some("verified"));
                assert_eq!(verification.verification_type.as_deref(), Some("toll_free"));
            }
            None => assert!(expect_unverified),
        }
    }
}

#[tokio::test]
async fn test_get_reads_the_test_key_limit() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/account"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(account_body(serde_json::Value::Null, "test")),
        )
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let account = client.account().get().await.expect("account should decode");

    assert_eq!(account.limits.messages_per_day, 100);
}

#[tokio::test]
async fn test_get_is_an_error_when_the_account_is_missing() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/account"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let error = client.account().get().await.unwrap_err();

    assert!(matches!(error, sendly::Error::Json(_)), "{error:?}");
}

fn created_key_body(key_type: &str) -> serde_json::Value {
    json!({
        "id": "key_1",
        "name": "Prod",
        "key": "sk_live_v1_new_secret",
        "keyPrefix": "sk_live_v1_new",
        "type": key_type,
        "createdAt": "2026-09-25T10:00:00.000Z",
        "expiresAt": null,
        "apiKey": {
            "id": "key_1",
            "name": "Prod",
            "type": key_type,
            "prefix": "sk_live_v1_new...",
            "scopes": ["sms:send", "sms:read"],
            "permissions": ["sms:send", "sms:read"],
            "isActive": true,
            "isRevoked": false,
            "createdAt": "2026-09-25T10:00:00.000Z",
            "lastUsedAt": null,
            "expiresAt": null
        }
    })
}

#[tokio::test]
async fn test_create_api_key_sends_the_type_and_scopes() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/account/keys"))
        .and(body_json(
            json!({ "name": "Prod", "type": "live", "scopes": ["sms:send", "sms:read"] }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(created_key_body("live")))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let created = client
        .account()
        .create_api_key_with_options(
            CreateApiKeyRequest::new("Prod")
                .key_type("live")
                .scopes(vec!["sms:send", "sms:read"]),
        )
        .await
        .expect("create should succeed");

    assert_eq!(created.key, "sk_live_v1_new_secret");
    assert_eq!(created.id.as_deref(), Some("key_1"));
    assert_eq!(created.key_type.as_deref(), Some("live"));
    let api_key = created.api_key.unwrap();
    assert_eq!(api_key.id, "key_1");
    assert_eq!(api_key.key_type.as_deref(), Some("live"));
    assert_eq!(
        api_key.scopes,
        Some(vec!["sms:send".to_string(), "sms:read".to_string()])
    );
}

#[tokio::test]
async fn test_create_api_key_leaves_out_the_type_by_default() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/account/keys"))
        .and(body_json(json!({ "name": "x" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(created_key_body("test")))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let created = client
        .account()
        .create_api_key("x")
        .await
        .expect("create should succeed");

    assert_eq!(created.key_type.as_deref(), Some("test"));
}

#[tokio::test]
async fn test_create_api_key_sends_expires_at() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/account/keys"))
        .and(body_json(
            json!({ "name": "CI", "expiresAt": "2027-01-01T00:00:00Z" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(created_key_body("test")))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    client
        .account()
        .create_api_key_with_options(
            CreateApiKeyRequest::new("CI").expires_at("2027-01-01T00:00:00Z"),
        )
        .await
        .expect("create should succeed");
}

#[tokio::test]
async fn test_get_api_key_usage_reads_the_summary() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/account/keys/key_1/usage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "keyId": "key_1",
            "keyName": "Production",
            "summary": { "totalRequests": 12, "totalCredits": 24, "lastUsed": "2026-09-25T10:00:00.000Z" },
            "recentRequests": [
                { "endpoint": "/api/v1/messages", "method": "POST", "statusCode": 201, "creditsUsed": 2, "createdAt": "2026-09-25T10:00:00.000Z" },
                { "endpoint": "/api/v1/messages", "method": "POST", "statusCode": 402, "creditsUsed": 0, "createdAt": "2026-09-25T09:59:00.000Z" }
            ],
            "endpointBreakdown": [{ "endpoint": "POST /api/v1/messages", "count": 12 }]
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let usage = client
        .account()
        .get_api_key_usage("key_1")
        .await
        .expect("usage should decode");

    assert_eq!(usage.total_requests, 12);
    assert_eq!(usage.credits_used, 24);
    assert_eq!(
        usage.last_request_at.as_deref(),
        Some("2026-09-25T10:00:00.000Z")
    );
    assert_eq!(usage.recent_requests.len(), 2);
    assert_eq!(usage.recent_requests[1].status_code, Some(402));
    assert_eq!(
        usage.endpoint_breakdown[0].endpoint,
        "POST /api/v1/messages"
    );
    assert_eq!(usage.endpoint_breakdown[0].count, 12);
}

fn usage_parts(
    usage: &sendly::ApiKeyUsage,
) -> (
    &[sendly::ApiKeyRequestRecord],
    &[sendly::ApiKeyEndpointCount],
) {
    (&usage.recent_requests, &usage.endpoint_breakdown)
}

#[tokio::test]
async fn test_api_key_usage_types_can_be_named() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/account/keys/key_1/usage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "keyId": "key_1",
            "keyName": "Production",
            "summary": { "totalRequests": 1, "totalCredits": 2, "lastUsed": "2026-09-25T10:00:00.000Z" },
            "recentRequests": [
                { "endpoint": "/api/v1/messages", "method": "POST", "statusCode": 201, "creditsUsed": 2, "createdAt": "2026-09-25T10:00:00.000Z" }
            ],
            "endpointBreakdown": [{ "endpoint": "POST /api/v1/messages", "count": 1 }]
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let usage = client.account().get_api_key_usage("key_1").await.unwrap();
    let (recent, breakdown) = usage_parts(&usage);

    assert_eq!(recent[0].status_code, Some(201));
    assert_eq!(breakdown[0].count, 1);
}
