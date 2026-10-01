mod common;

use common::{create_test_client, setup_mock_server};
use sendly::business_upgrade::{BrnType, EntityType, StartUpgradeRequest};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn test_start_decodes_the_202_the_api_sends() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/workspaces/ws_1/upgrade"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "success": true,
            "pendingVerificationId": "bv_new",
            "status": "provisioning",
            "message": "Your business entity upgrade is being provisioned. Your current number stays active until the new one is approved (typically 1-2 weeks)."
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let started = client
        .business_upgrade()
        .start(
            "ws_1",
            StartUpgradeRequest::new(
                "Acme Holdings LLC",
                "12-3456789",
                BrnType::Ein,
                "US",
                EntityType::PrivateProfit,
            ),
            None,
        )
        .await
        .expect("the 202 body should decode");

    assert_eq!(started.pending_verification_id, "bv_new");
    assert_eq!(started.status.as_deref(), Some("provisioning"));
    assert_eq!(started.toll_free_number, None);
    assert!(started.success);
}
