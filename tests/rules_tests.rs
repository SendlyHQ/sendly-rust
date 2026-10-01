mod common;

use common::{create_test_client, setup_mock_server};
use sendly::{CreateRuleRequest, RuleActions, RuleConditions, RuleMatch, UpdateRuleRequest};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

fn rule_row(
    id: &str,
    conditions: serde_json::Value,
    actions: serde_json::Value,
) -> serde_json::Value {
    json!({
        "id": id,
        "userId": "user_1",
        "organizationId": "org_1",
        "name": "Complaints",
        "conditions": conditions,
        "actions": actions,
        "enabled": true,
        "priority": 0,
        "createdAt": "2026-09-25T10:00:00.000Z",
        "updatedAt": "2026-09-25T10:00:00.000Z"
    })
}

#[tokio::test]
async fn test_list_decodes_object_rules_and_array_rules_from_older_sdks() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/rules"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                rule_row("rule_1", json!({ "intent": "complaint" }), json!({ "addLabels": ["lbl_1"] })),
                rule_row("rule_2", json!([{ "intent": "x" }]), json!([{ "addLabels": ["l"] }]))
            ]
        })))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let rules = client
        .rules()
        .list()
        .await
        .expect("object-shaped rules must decode");

    assert_eq!(rules.data.len(), 2);
    assert_eq!(
        rules.data[0].conditions.intent,
        Some(RuleMatch::One("complaint".to_string()))
    );
    assert_eq!(rules.data[0].actions.add_labels, vec!["lbl_1"]);
    assert_eq!(rules.data[0].enabled, Some(true));
    assert_eq!(
        rules.data[1].conditions.intent,
        Some(RuleMatch::One("x".to_string()))
    );
    assert_eq!(rules.data[1].actions.add_labels, vec!["l"]);
}

#[tokio::test]
async fn test_create_sends_conditions_and_actions_as_objects() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/rules"))
        .respond_with(ResponseTemplate::new(201).set_body_json(rule_row(
            "rule_1",
            json!({ "intent": "complaint" }),
            json!({ "addLabels": ["lbl_1"] }),
        )))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let rule = client
        .rules()
        .create(CreateRuleRequest {
            name: "Complaints".to_string(),
            conditions: RuleConditions::new().intent("complaint"),
            actions: RuleActions::new().add_labels(vec!["lbl_1"]),
            priority: None,
        })
        .await
        .expect("create should succeed");

    assert_eq!(rule.id, "rule_1");
    let requests = mock_server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(
        body,
        json!({ "name": "Complaints", "conditions": { "intent": "complaint" }, "actions": { "addLabels": ["lbl_1"] } })
    );
}

#[tokio::test]
async fn test_update_can_switch_a_rule_off() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/rules/rule_1"))
        .and(wiremock::matchers::body_json(json!({ "enabled": false })))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut rule = rule_row(
                "rule_1",
                json!({ "intent": ["complaint", "refund"] }),
                json!({ "addLabels": [], "closeConversation": true }),
            );
            rule["enabled"] = json!(false);
            rule
        }))
        .mount(&mock_server)
        .await;
    let client = create_test_client(&mock_server.uri());

    let rule = client
        .rules()
        .update(
            "rule_1",
            UpdateRuleRequest {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .await
        .expect("update should succeed");

    assert_eq!(rule.enabled, Some(false));
    assert_eq!(
        rule.conditions.intent,
        Some(RuleMatch::Any(vec![
            "complaint".to_string(),
            "refund".to_string()
        ]))
    );
    assert_eq!(rule.actions.close_conversation, Some(true));
}

#[test]
fn test_conditions_accept_a_list_of_string_literals() {
    let conditions = RuleConditions::new()
        .intent(vec!["complaint", "refund"])
        .sentiment(vec![String::from("negative")]);

    assert_eq!(
        serde_json::to_value(&conditions).unwrap(),
        json!({ "intent": ["complaint", "refund"], "sentiment": ["negative"] })
    );
}
