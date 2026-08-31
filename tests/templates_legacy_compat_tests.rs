mod common;

use common::{create_test_client, setup_mock_server};
use sendly::{
    CreateTemplateRequest, ListTemplatesOptions, Template, TemplateType, UpdateTemplateRequest,
};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

// A consumer written against the 3.37.1 template surface: the payload shape it
// recorded, the fields it reads, and the builders it calls. Nothing in this file
// may reference anything added after that release.

fn legacy_template_json() -> serde_json::Value {
    json!({
        "id": "tpl_abc",
        "name": "Welcome",
        "body": "Hi {{name}}, your code is {{code}}",
        "type": "preset",
        "locale": "en-US",
        "variables": ["name", "code"],
        "isDefault": true,
        "isPublished": true,
        "createdAt": "2026-08-25T10:00:00Z",
        "updatedAt": "2026-08-25T10:00:00Z"
    })
}

#[test]
#[allow(deprecated)]
fn test_legacy_payload_still_decodes_from_str() {
    let recorded = legacy_template_json().to_string();
    let template: Template =
        serde_json::from_str(&recorded).expect("legacy payload should still decode");

    assert_eq!(template.id, "tpl_abc");
    assert_eq!(template.name, "Welcome");
    assert_eq!(template.body, "Hi {{name}}, your code is {{code}}");
    assert_eq!(template.template_type, TemplateType::Preset);
    assert_eq!(template.locale.as_deref(), Some("en-US"));
    assert_eq!(
        template.variables,
        vec!["name".to_string(), "code".to_string()]
    );
    assert!(template.is_default);
    assert!(template.is_published);
    assert!(template.is_preset());
    assert!(!template.is_custom());
    assert_eq!(template.created_at.as_deref(), Some("2026-08-25T10:00:00Z"));
    assert_eq!(template.updated_at.as_deref(), Some("2026-08-25T10:00:00Z"));
}

#[test]
#[allow(deprecated)]
fn test_legacy_type_field_drives_preset_consistently() {
    let custom: Template = serde_json::from_value(json!({
        "id": "tpl_custom",
        "name": "Reminder",
        "body": "See you at {{time}}",
        "type": "custom",
        "variables": ["time"]
    }))
    .expect("legacy custom payload should decode");

    assert_eq!(custom.template_type, TemplateType::Custom);
    assert!(!custom.is_preset);
    assert!(custom.is_custom());

    let preset: Template = serde_json::from_value(json!({
        "id": "tpl_preset",
        "name": "Reminder",
        "body": "See you at {{time}}",
        "type": "preset"
    }))
    .expect("legacy preset payload should decode");

    assert_eq!(preset.template_type, TemplateType::Preset);
    assert!(preset.is_preset);
    assert!(preset.is_preset());
    assert!(!preset.is_custom());
}

#[tokio::test]
#[allow(deprecated)]
async fn test_legacy_shaped_mock_still_decodes_through_client() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/templates/tpl_abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(legacy_template_json()))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let template = client
        .templates()
        .get("tpl_abc")
        .await
        .expect("get should succeed against a legacy-shaped mock");

    assert_eq!(template.body, "Hi {{name}}, your code is {{code}}");
    assert_eq!(template.template_type, TemplateType::Preset);
    assert_eq!(template.variables.len(), 2);
    assert!(template.is_published);
    assert!(template.is_published());
}

#[test]
#[allow(deprecated)]
fn test_legacy_builders_still_compile() {
    let create = CreateTemplateRequest::new("Welcome", "Hi {{name}}")
        .locale("en-US")
        .published(true);
    assert_eq!(create.body, "Hi {{name}}");
    assert_eq!(create.locale.as_deref(), Some("en-US"));
    assert_eq!(create.is_published, Some(true));

    let update = UpdateTemplateRequest::new()
        .name("Welcome v2")
        .body("Hi {{name}} v2")
        .locale("en-US")
        .published(false);
    assert_eq!(update.name.as_deref(), Some("Welcome v2"));
    assert_eq!(update.body.as_deref(), Some("Hi {{name}} v2"));
    assert_eq!(update.locale.as_deref(), Some("en-US"));
    assert_eq!(update.is_published, Some(false));

    let options = ListTemplatesOptions::new()
        .limit(10)
        .template_type(TemplateType::Preset)
        .locale("en-US");
    assert_eq!(options.limit, Some(10));
    assert_eq!(options.template_type, Some(TemplateType::Preset));
    assert_eq!(options.locale.as_deref(), Some("en-US"));
}
