mod common;

use common::{create_test_client, setup_mock_server};
use sendly::CheckNumbersRequest;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn test_check_numbers_reads_already_running() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/contacts/lookup"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "alreadyRunning": true,
            "message": "A carrier lookup is already in progress for this scope. Results will appear as it completes."
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let result = client
        .contacts()
        .check_numbers(CheckNumbersRequest::default())
        .await
        .expect("check_numbers should decode");

    assert!(result.already_running);
}
