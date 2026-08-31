mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use sendly::ListTransactionsOptions;
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
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
                    "type": "usage",
                    "description": "SMS send",
                    "createdAt": "2026-08-25T10:00:00Z"
                }
            ],
            "total": 1,
            "hasMore": false
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
    assert_eq!(result.total, 1);
    assert!(!result.has_more);
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
