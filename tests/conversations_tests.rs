mod common;

use common::{create_test_client, setup_mock_server};
use sendly::ReplyToConversationRequest;
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

fn send_response(text: &str) -> serde_json::Value {
    json!({
        "id": "msg_reply",
        "to": "+15555550100",
        "from": "+18005550199",
        "text": text,
        "status": "queued",
        "direction": "outbound",
        "error": null,
        "segments": 1,
        "creditsUsed": 2,
        "senderType": "number_pool",
        "createdAt": "2026-09-25T10:00:00.000Z",
        "metadata": {},
        "senderNote": "Message will be sent from a toll-free number in your number pool."
    })
}

#[tokio::test]
async fn test_add_labels_returns_the_labels_the_api_sends() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/conversations/conv_1/labels"))
        .and(body_json(json!({ "labelIds": ["lbl_1"] })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{
                "id": "lbl_1",
                "userId": "user_1",
                "organizationId": "org_1",
                "name": "VIP",
                "color": "#6b7280",
                "description": null,
                "createdAt": "2026-09-25T10:00:00.000Z"
            }]
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let labels = client
        .conversations()
        .add_labels("conv_1", vec!["lbl_1".to_string()])
        .await
        .expect("add_labels should decode the label list");

    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].id, "lbl_1");
    assert_eq!(labels[0].name, "VIP");
}

#[tokio::test]
async fn test_remove_label_accepts_the_empty_204() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("DELETE"))
        .and(path("/conversations/conv_1/labels/lbl_1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    client
        .conversations()
        .remove_label("conv_1", "lbl_1")
        .await
        .expect("remove_label should accept 204 No Content");
}

#[tokio::test]
async fn test_reply_decodes_the_live_send_response() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/conversations/conv_1/messages"))
        .and(body_json(json!({ "text": "On our way" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(send_response("On our way")))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let message = client
        .conversations()
        .reply(
            "conv_1",
            ReplyToConversationRequest {
                text: "On our way".to_string(),
                message_type: None,
                metadata: None,
                media_urls: None,
            },
        )
        .await
        .expect("reply should decode");

    assert_eq!(message.id, "msg_reply");
    assert_eq!(message.sender_type, Some(sendly::SenderType::NumberPool));
}

#[tokio::test]
async fn test_reply_still_needs_text_or_media() {
    let mock_server = setup_mock_server().await;
    let client = create_test_client(&mock_server.uri());

    let error = client
        .conversations()
        .reply(
            "conv_1",
            ReplyToConversationRequest {
                text: String::new(),
                message_type: None,
                metadata: None,
                media_urls: Some(vec![]),
            },
        )
        .await
        .unwrap_err();

    assert!(
        matches!(error, sendly::Error::Validation { .. }),
        "{error:?}"
    );
    assert!(mock_server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn test_reply_sends_a_media_only_reply() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/conversations/conv_1/messages"))
        .and(body_json(
            json!({ "mediaUrls": ["https://x.example/a.jpg"] }),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(send_response("")))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    client
        .conversations()
        .reply(
            "conv_1",
            ReplyToConversationRequest {
                text: String::new(),
                message_type: None,
                metadata: None,
                media_urls: Some(vec!["https://x.example/a.jpg".to_string()]),
            },
        )
        .await
        .expect("a media-only reply should reach the API");
}
