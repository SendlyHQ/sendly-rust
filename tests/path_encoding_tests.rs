mod common;

use common::{create_test_client, setup_mock_server};
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn paths_received(mock_server: &MockServer) -> Vec<String> {
    mock_server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .map(|r| r.url.path().to_string())
        .collect()
}

async fn catch_all_server() -> MockServer {
    let mock_server = setup_mock_server().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({ "error": "not_found" })))
        .mount(&mock_server)
        .await;
    mock_server
}

#[tokio::test]
async fn test_webhook_ids_are_percent_encoded() {
    let mock_server = catch_all_server().await;
    let client = create_test_client(&mock_server.uri());

    let _ = client.webhooks().get("../account/keys").await;
    let _ = client.webhooks().list_deliveries("a/b", None).await;
    let _ = client
        .webhooks()
        .get_delivery("whk_1", "../../account")
        .await;

    assert_eq!(
        paths_received(&mock_server).await,
        vec![
            "/webhooks/..%2Faccount%2Fkeys",
            "/webhooks/a%2Fb/deliveries",
            "/webhooks/whk_1/deliveries/..%2F..%2Faccount",
        ]
    );
}

#[tokio::test]
async fn test_api_key_ids_are_percent_encoded() {
    let mock_server = catch_all_server().await;
    let client = create_test_client(&mock_server.uri());

    let _ = client.account().get_api_key("../credits").await;
    let _ = client.account().get_api_key_usage("key?x=1").await;
    let _ = client.account().revoke_api_key("a/b").await;

    assert_eq!(
        paths_received(&mock_server).await,
        vec![
            "/account/keys/..%2Fcredits",
            "/account/keys/key%3Fx%3D1/usage",
            "/account/keys/a%2Fb/revoke",
        ]
    );
}

#[tokio::test]
async fn test_enterprise_ids_are_percent_encoded() {
    let mock_server = catch_all_server().await;
    let client = create_test_client(&mock_server.uri());

    let _ = client.enterprise().workspaces().get("../../account").await;
    let _ = client
        .enterprise()
        .workspaces()
        .revoke_key("ws_1", "key/1")
        .await;
    let _ = client
        .enterprise()
        .workspaces()
        .delete_webhooks("ws_1", Some("whk_1&all=true"))
        .await;

    let requests = mock_server.received_requests().await.unwrap();
    let urls: Vec<String> = requests
        .iter()
        .map(|r| match r.url.query() {
            Some(query) => format!("{}?{}", r.url.path(), query),
            None => r.url.path().to_string(),
        })
        .collect();
    assert_eq!(
        urls,
        vec![
            "/enterprise/workspaces/..%2F..%2Faccount",
            "/enterprise/workspaces/ws_1/keys/key%2F1",
            "/enterprise/workspaces/ws_1/webhooks?webhookId=whk_1%26all%3Dtrue",
        ]
    );
}

fn assert_rejected<T: std::fmt::Debug>(call: &str, result: sendly::Result<T>) {
    match result {
        Err(sendly::Error::Validation { code: None, .. }) => {}
        other => panic!("{call}: expected a client-side validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_dot_segment_ids_send_no_request() {
    let mock_server = catch_all_server().await;
    let client = create_test_client(&mock_server.uri());
    let workspaces = client.enterprise().workspaces();

    assert_rejected("revoke_key", workspaces.revoke_key("ws_1", "..").await);
    assert_rejected(
        "delete_opt_in_page",
        workspaces.delete_opt_in_page("ws_1", "..").await,
    );
    assert_rejected(
        "cancel_invitation",
        workspaces.cancel_invitation("ws_1", "..").await,
    );
    assert_rejected("workspaces.get", workspaces.get(".").await);
    assert_rejected("get_api_key", client.account().get_api_key("..").await);
    assert_rejected(
        "revoke_api_key",
        client.account().revoke_api_key("..").await,
    );
    assert_rejected("webhooks.delete", client.webhooks().delete(".").await);
    assert_rejected(
        "get_delivery",
        client.webhooks().get_delivery("whk_1", "..").await,
    );
    assert_rejected(
        "remove_contact",
        client
            .contacts()
            .lists()
            .remove_contact("list_1", "..")
            .await,
    );
    assert_rejected("contacts.delete", client.contacts().delete("..").await);
    assert_rejected("campaigns.delete", client.campaigns().delete("..").await);
    assert_rejected("messages.get", client.messages().get("..").await);
    assert_rejected("templates.delete", client.templates().delete("..").await);
    assert_rejected("calls.get", client.calls().get("..").await);
    assert_rejected(
        "voice.agents.delete",
        client.voice().agents().delete("..").await,
    );
    assert_rejected("numbers.get", client.numbers().get("..").await);
    assert_rejected("verify.get", client.verify().get("..").await);
    assert_rejected("labels.delete", client.labels().delete("..").await);
    assert_rejected("rules.delete", client.rules().delete("..").await);
    assert_rejected("drafts.get", client.drafts().get("..").await);
    assert_rejected(
        "conversations.get",
        client.conversations().get("..", None).await,
    );
    assert_rejected(
        "whatsapp.templates.delete",
        client.whatsapp().templates().delete("..").await,
    );

    assert_eq!(paths_received(&mock_server).await, Vec::<String>::new());
}

#[tokio::test]
async fn test_ids_with_dots_inside_are_still_sent() {
    let mock_server = catch_all_server().await;
    let client = create_test_client(&mock_server.uri());

    let _ = client.webhooks().get("whk..1").await;
    let _ = client.account().get_api_key("...").await;

    assert_eq!(
        paths_received(&mock_server).await,
        vec!["/webhooks/whk..1", "/account/keys/..."]
    );
}
