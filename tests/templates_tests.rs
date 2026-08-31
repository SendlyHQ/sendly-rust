mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use sendly::{CreateTemplateRequest, ListTemplatesOptions, TemplateType, UpdateTemplateRequest};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, ResponseTemplate};

// Every mock below mounts only the real path. The resource used to call
// /verify/templates, which the API has never served, so a regression would
// miss the mock and surface as a 404 rather than a silent pass.

fn template_json(id: &str, name: &str, text: &str, status: &str) -> serde_json::Value {
    json!({
        "id": id,
        "name": name,
        "text": text,
        "variables": [{ "key": "name", "type": "string", "fallback": "there" }],
        "is_preset": false,
        "preset_slug": null,
        "status": status,
        "version": 1,
        "published_at": null,
        "created_at": "2026-08-25T10:00:00Z",
        "updated_at": "2026-08-25T10:00:00Z"
    })
}

#[tokio::test]
async fn test_templates_list_uses_templates_path() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/templates"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "templates": [template_json("tpl_abc", "Welcome", "Hi {{name}}", "draft")]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .templates()
        .list(ListTemplatesOptions::new().limit(10))
        .await
        .expect("list should succeed");

    assert_eq!(result.templates.len(), 1);
    assert_eq!(result.templates[0].id, "tpl_abc");
    assert_eq!(result.templates[0].text, "Hi {{name}}");
    assert_eq!(result.templates[0].variable_specs[0].key, "name");
    assert_eq!(
        result.templates[0].variable_specs[0].fallback.as_deref(),
        Some("there")
    );
    assert!(result.templates[0].is_custom());
}

#[tokio::test]
async fn test_templates_get_uses_templates_path() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/templates/tpl_abc"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(template_json("tpl_abc", "Welcome", "Hi {{name}}", "draft")),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let template = client
        .templates()
        .get("tpl_abc")
        .await
        .expect("get should succeed");

    assert_eq!(template.id, "tpl_abc");
    assert!(!template.is_published());
}

#[tokio::test]
async fn test_templates_create_posts_text_field() {
    let mock_server = setup_mock_server().await;
    // The API reads `text`; the request used to serialize the field as `body`,
    // which would have been rejected as a missing required field.
    Mock::given(method("POST"))
        .and(path("/templates"))
        .and(body_json(json!({ "name": "Welcome", "text": "Hi {{name}}" })))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(template_json("tpl_abc", "Welcome", "Hi {{name}}", "draft")),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let template = client
        .templates()
        .create(CreateTemplateRequest::new("Welcome", "Hi {{name}}"))
        .await
        .expect("create should succeed");

    assert_eq!(template.id, "tpl_abc");
    assert_eq!(template.status.as_deref(), Some("draft"));
}

#[tokio::test]
async fn test_templates_update_patches_text_field() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/templates/tpl_abc"))
        .and(body_json(json!({ "text": "Hi {{name}} v2" })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(template_json("tpl_abc", "Welcome", "Hi {{name}} v2", "draft")),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let template = client
        .templates()
        .update("tpl_abc", UpdateTemplateRequest::new().text("Hi {{name}} v2"))
        .await
        .expect("update should succeed");

    assert_eq!(template.text, "Hi {{name}} v2");
}

#[tokio::test]
async fn test_templates_publish_sends_empty_object_body() {
    let mock_server = setup_mock_server().await;
    // A unit body serializes to the JSON literal `null`, which the API's body
    // parser rejects outright. Publish must send `{}`.
    Mock::given(method("POST"))
        .and(path("/templates/tpl_abc/publish"))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(template_json(
            "tpl_abc",
            "Welcome",
            "Hi {{name}}",
            "published",
        )))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let template = client
        .templates()
        .publish("tpl_abc")
        .await
        .expect("publish should succeed");

    assert!(template.is_published());
}

#[tokio::test]
async fn test_templates_delete_accepts_204_no_content() {
    let mock_server = setup_mock_server().await;
    // The API answers 204 with an empty body; decoding that as JSON would fail.
    Mock::given(method("DELETE"))
        .and(path("/templates/tpl_abc"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .templates()
        .delete("tpl_abc")
        .await
        .expect("delete should succeed");

    assert!(result.success);
}

#[tokio::test]
#[allow(deprecated)]
async fn test_templates_deprecated_fields_are_populated() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/templates/tpl_abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(template_json(
            "tpl_abc",
            "Welcome",
            "Hi {{name}}",
            "published",
        )))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let template = client
        .templates()
        .get("tpl_abc")
        .await
        .expect("get should succeed");

    assert_eq!(template.body, template.text);
    assert_eq!(template.variables, vec!["name".to_string()]);
    assert_eq!(template.template_type, TemplateType::Custom);
    assert!(!template.is_preset());
    assert!(template.is_custom());
    assert!(template.is_published);
    assert!(!template.is_default);
    assert_eq!(template.locale, None);

    let encoded = serde_json::to_value(&template).expect("serialize should succeed");
    assert!(encoded.get("body").is_none());
    assert_eq!(encoded["text"], "Hi {{name}}");
    assert_eq!(encoded["variables"][0]["key"], "name");
}

#[tokio::test]
#[allow(deprecated)]
async fn test_templates_update_deprecated_body_builder_sends_text() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PATCH"))
        .and(path("/templates/tpl_abc"))
        .and(body_json(json!({ "text": "Hi {{name}} v2" })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(template_json("tpl_abc", "Welcome", "Hi {{name}} v2", "draft")),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let template = client
        .templates()
        .update("tpl_abc", UpdateTemplateRequest::new().body("Hi {{name}} v2"))
        .await
        .expect("update should succeed");

    assert_eq!(template.text, "Hi {{name}} v2");
}

#[tokio::test]
async fn test_templates_accepts_numeric_variable_fallback() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/templates/tpl_num"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "tpl_num",
            "name": "Order",
            "text": "Order {{qty}} of {{sku}}",
            "variables": [
                { "key": "qty", "type": "number", "fallback": 1 },
                { "key": "sku", "type": "string", "fallback": "unknown" }
            ],
            "is_preset": false,
            "status": "draft",
            "version": 1
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let template = client
        .templates()
        .get("tpl_num")
        .await
        .expect("get should succeed");

    assert_eq!(template.variable_specs[0].fallback.as_deref(), Some("1"));
    assert_eq!(
        template.variable_specs[1].fallback.as_deref(),
        Some("unknown")
    );
}

#[test]
#[allow(deprecated)]
fn test_templates_update_body_and_text_are_one_value() {
    let text_last = UpdateTemplateRequest::new().body("B").text("A");
    assert_eq!(text_last.text.as_deref(), Some("A"));
    assert_eq!(text_last.body.as_deref(), Some("A"));

    let body_last = UpdateTemplateRequest::new().text("A").body("B");
    assert_eq!(body_last.text.as_deref(), Some("B"));
    assert_eq!(body_last.body.as_deref(), Some("B"));
}
