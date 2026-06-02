mod common;

use common::{create_test_client, setup_mock_server};
use sendly::{BuyNumberRequest, Error, ListAvailableNumbersOptions};
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

// ==================== list_countries() Tests ====================

#[tokio::test]
async fn test_list_countries_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/numbers/countries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "countries": [
                { "code": "GB", "name": "United Kingdom", "numberTypes": ["mobile", "local"] },
                { "code": "US", "name": "United States", "numberTypes": ["local", "toll_free"] }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.numbers().list_countries().await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.countries.len(), 2);
    assert_eq!(response.countries[0].code, "GB");
    assert_eq!(response.countries[0].name, "United Kingdom");
    assert_eq!(response.countries[0].number_types, vec!["mobile", "local"]);
}

// ==================== list_available() Tests ====================

#[tokio::test]
async fn test_list_available_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/numbers/available"))
        .and(query_param("country", "GB"))
        .and(query_param("type", "mobile"))
        .and(query_param("contains", "7700"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "numbers": [
                {
                    "phoneNumber": "+447700900123",
                    "country": "GB",
                    "numberType": "mobile",
                    "monthlyCost": "1.50",
                    "currency": "USD"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .numbers()
        .list_available(ListAvailableNumbersOptions::new("GB", "mobile").contains("7700"))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.numbers.len(), 1);
    let n = &response.numbers[0];
    assert_eq!(n.phone_number, "+447700900123");
    assert_eq!(n.country, "GB");
    assert_eq!(n.number_type, "mobile");
    assert_eq!(n.monthly_cost, "1.50");
    assert_eq!(n.currency, "USD");
}

#[tokio::test]
async fn test_list_available_requires_country() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .numbers()
        .list_available(ListAvailableNumbersOptions::new("", "mobile"))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("country")),
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_list_available_requires_type() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .numbers()
        .list_available(ListAvailableNumbersOptions::new("GB", ""))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("type")),
        _ => panic!("Expected Validation error"),
    }
}

// ==================== list() Tests ====================

#[tokio::test]
async fn test_list_owned_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/numbers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "numbers": [
                {
                    "id": "num_abc123",
                    "phoneNumber": "+447700900123",
                    "status": "active",
                    "source": "purchased",
                    "countryCode": "GB",
                    "phoneNumberType": "mobile",
                    "monthlyCostCents": 150
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.numbers().list().await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.numbers.len(), 1);
    let n = &response.numbers[0];
    assert_eq!(n.id, "num_abc123");
    assert_eq!(n.phone_number, "+447700900123");
    assert_eq!(n.status, "active");
    assert_eq!(n.source.as_deref(), Some("purchased"));
    assert_eq!(n.country_code.as_deref(), Some("GB"));
    assert_eq!(n.phone_number_type.as_deref(), Some("mobile"));
    assert_eq!(n.monthly_cost_cents, 150);
}

// ==================== buy() Tests ====================

#[tokio::test]
async fn test_buy_provisioning() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/numbers/buy"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "status": "provisioning",
            "number": {
                "id": "num_abc123",
                "phoneNumber": "+447700900123",
                "status": "provisioning"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .numbers()
        .buy(BuyNumberRequest::new(
            "+447700900123",
            "GB",
            "mobile",
            "1.50",
        ))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, "provisioning");
    assert!(response.action.is_none());
    let number = response.number.expect("number should be present");
    assert_eq!(number.id, "num_abc123");
    assert_eq!(number.phone_number, "+447700900123");
    assert_eq!(number.status, "provisioning");
    // Buy's trimmed `number` omits these fields.
    assert!(number.source.is_none());
    assert!(number.country_code.is_none());
    assert!(number.phone_number_type.is_none());
}

#[tokio::test]
async fn test_buy_documents_required_carries_action() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/numbers/buy"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "status": "documents_required",
            "requirements": [
                { "field": "proof_of_address", "type": "document" }
            ],
            "action": {
                "url": "https://sendly.live/action/upload-docs/xyz",
                "code": "ABC23456",
                "actionCode": "0123456789abcdef0123456789abcdef",
                "expiresAt": 1768471200000_i64
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .numbers()
        .buy(BuyNumberRequest::new(
            "+447700900123",
            "GB",
            "mobile",
            "1.50",
        ))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status, "documents_required");
    assert!(response.number.is_none());
    let requirements = response.requirements.expect("requirements should be present");
    assert_eq!(requirements.len(), 1);
    let action = response.action.expect("action should be present");
    assert_eq!(action.url, "https://sendly.live/action/upload-docs/xyz");
    assert_eq!(action.code, "ABC23456");
    assert_eq!(
        action.action_code.as_deref(),
        Some("0123456789abcdef0123456789abcdef")
    );
    assert_eq!(action.expires_at, Some(1768471200000));
}

#[tokio::test]
async fn test_buy_with_action_code() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/numbers/buy"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "status": "provisioning"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .numbers()
        .buy(
            BuyNumberRequest::new("+447700900123", "GB", "mobile", "1.50")
                .action_code("ABC123"),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, "provisioning");
}
