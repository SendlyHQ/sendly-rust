#![recursion_limit = "256"]

mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use sendly::{
    Error, IdempotentRequestOptions, ShortCodeApplicationInput, SubmitShortCodeApplicationRequest,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, ResponseTemplate};

fn lease_json() -> serde_json::Value {
    json!({
        "monthlyCents": 100000,
        "minimumTermMonths": 3,
        "state": "not_started",
        "billingEnabled": true,
        "startedAt": null,
        "nextChargeAt": null,
        "paidThrough": null,
        "termEndsAt": null,
        "endsAt": null,
        "cancelRequestedAt": null,
        "pastDue": null,
        "dueMonths": 0,
        "dueCents": 0
    })
}

fn view_json() -> serde_json::Value {
    let mut lease = lease_json();
    lease["startsAt"] = json!("go_live");
    json!({
        "enabled": true,
        "application": {
            "id": "sc_1",
            "shortCode": null,
            "countryCode": "US",
            "status": "requested",
            "reviewStatus": "awaiting_review",
            "reviewNote": null,
            "orderType": "new",
            "codeType": "random",
            "contentProviderSameAsBrand": true,
            "sampleMessages": ["Acme: your order shipped. Reply STOP to opt out."],
            "useCase": "Order and delivery alerts",
            "optInFlow": "Customers text JOIN to the code after checkout.",
            "optInConfirmation": null,
            "helpResponse": null,
            "stopConfirmation": null,
            "messageFrequency": "Up to 4 messages per order",
            "campaignKeyword": "JOIN",
            "expectedMonthlyVolume": "50000",
            "expectedDailyVolume": "2000",
            "privacyPolicyUrl": "https://acme.example/privacy",
            "termsUrl": "https://acme.example/terms",
            "requestedDigits": null,
            "losingProvider": null,
            "brandContactName": "Ada Lovelace",
            "brandContactEmail": "ada@acme.example",
            "brandContactPhone": "+15125550100",
            "contentProviderLegalName": null,
            "contentProviderEin": null,
            "contentProviderContactName": null,
            "contentProviderContactEmail": null,
            "contentProviderContactPhone": null,
            "brandRegistrationStatus": "registered",
            "contentProviderRegistrationStatus": "not_started",
            "submittedAt": "2026-09-01T10:00:00.000Z",
            "reviewedAt": null,
            "filedAt": null,
            "activatedAt": null,
            "registryRevettingDueAt": null,
            "createdAt": "2026-08-30T10:00:00.000Z",
            "updatedAt": "2026-09-01T10:00:00.000Z"
        },
        "prefill": {
            "source": "tendlc",
            "legalName": "Acme Inc",
            "address": { "city": "Austin", "state": "TX" },
            "contact": { "email": "ada@acme.example" }
        },
        "editable": false,
        "lockMessage": "Sendly is reviewing this application.",
        "canStartNewApplication": false,
        "generatedDefaults": { "useCase": "Order alerts" },
        "requiredDocuments": ["order_brief", "brand_registration"],
        "missingDocuments": ["brand_registration"],
        "documents": [{
            "kind": "order_brief",
            "templateReady": true,
            "signedAt": "2026-09-02T10:00:00.000Z",
            "signedByName": "Ada Lovelace",
            "awaitingSignature": false
        }],
        "quote": {
            "codeType": "random",
            "monthlyUsd": 1000,
            "setupUsd": 999,
            "currency": "USD",
            "autoBilled": false,
            "setupCents": 99900,
            "monthlyCents": 100000,
            "minimumTermMonths": 3,
            "setupChargedAt": "submit",
            "leaseStartsAt": "go_live"
        },
        "billing": {
            "setupFee": {
                "amountCents": 99900,
                "status": "paid",
                "chargedAt": "submit",
                "paidAt": "2026-09-01T10:00:00.000Z",
                "refundedCents": 0,
                "refundedAt": null
            },
            "lease": lease,
            "terms": {
                "version": "2026-09",
                "acceptedVersion": "2026-09",
                "acceptedAt": "2026-09-01T10:00:00.000Z"
            },
            "refundPolicy": "full_refund_before_filing",
            "exempt": null,
            "charges": [{
                "id": "scc_1",
                "kind": "setup",
                "amountCents": 99900,
                "status": "paid",
                "paidAt": "2026-09-01T10:00:00.000Z",
                "refundedCents": 0,
                "refundedAt": null,
                "periodStart": null,
                "periodEnd": null,
                "createdAt": "2026-09-01T10:00:00.000Z"
            }]
        },
        "verification": { "verified": true, "state": "verified", "href": "/verify" },
        "carriers": {
            "approved": 1,
            "total": 4,
            "overall": "in_progress",
            "byCarrier": [{
                "carrier": "TMOBILE",
                "label": "T-Mobile",
                "status": "approved",
                "updatedAt": "2026-09-05T10:00:00.000Z"
            }]
        }
    })
}

#[tokio::test]
async fn test_short_codes_list_decodes_codes_and_billing() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/short_codes"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "shortCodes": [{
                "id": "sc_1",
                "shortCode": "12345",
                "countryCode": "US",
                "status": "active",
                "reviewStatus": "filed",
                "useCase": "Order and delivery alerts",
                "createdAt": "2026-08-30T10:00:00.000Z",
                "billing": {
                    "setupFee": { "status": "paid", "amountCents": 99900 },
                    "lease": lease_json()
                }
            }, {
                "id": "sc_2",
                "shortCode": null,
                "countryCode": "US",
                "status": "a_status_added_later",
                "reviewStatus": "draft",
                "useCase": "",
                "createdAt": "2026-10-01T10:00:00.000Z",
                "billing": {
                    "setupFee": { "status": "unpaid", "amountCents": 99900 },
                    "lease": lease_json()
                }
            }]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let list = client.short_codes().list().await.unwrap();

    assert_eq!(list.short_codes.len(), 2);
    let code = &list.short_codes[0];
    assert_eq!(code.id, "sc_1");
    assert_eq!(code.short_code.as_deref(), Some("12345"));
    assert_eq!(code.status, "active");
    assert_eq!(code.review_status, "filed");
    assert_eq!(code.billing.setup_fee.status, "paid");
    assert_eq!(code.billing.setup_fee.amount_cents, 99900);
    assert_eq!(code.billing.lease.monthly_cents, 100000);
    assert_eq!(code.billing.lease.state, "not_started");
    assert!(code.billing.lease.past_due.is_none());
    assert!(list.short_codes[1].short_code.is_none());
    assert_eq!(list.short_codes[1].status, "a_status_added_later");
}

#[tokio::test]
async fn test_application_get_decodes_the_view() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/short_codes/application"))
        .respond_with(ResponseTemplate::new(200).set_body_json(view_json()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let view = client.short_codes().application().get().await.unwrap();

    assert!(view.enabled);
    assert!(!view.editable);
    assert_eq!(
        view.lock_message.as_deref(),
        Some("Sendly is reviewing this application.")
    );
    assert_eq!(view.application.id.as_deref(), Some("sc_1"));
    assert_eq!(view.application.review_status, "awaiting_review");
    assert_eq!(view.application.campaign_keyword.as_deref(), Some("JOIN"));
    assert!(view.application.help_response.is_none());
    assert!(view.application.content_provider_same_as_brand);
    assert_eq!(view.application.sample_messages.len(), 1);
    assert_eq!(view.prefill.source, "tendlc");
    assert_eq!(view.prefill.legal_name.as_deref(), Some("Acme Inc"));
    assert_eq!(
        view.prefill.address.as_ref().unwrap().city.as_deref(),
        Some("Austin")
    );
    assert_eq!(
        view.generated_defaults
            .as_ref()
            .unwrap()
            .use_case
            .as_deref(),
        Some("Order alerts")
    );
    assert_eq!(view.missing_documents, vec!["brand_registration"]);
    assert!(view.documents[0].template_ready);
    assert_eq!(view.quote.setup_cents, 99900);
    assert_eq!(view.quote.lease_starts_at, "go_live");
    assert_eq!(view.billing.setup_fee.charged_at.as_deref(), Some("submit"));
    assert_eq!(view.billing.lease.starts_at.as_deref(), Some("go_live"));
    assert_eq!(view.billing.terms.version, "2026-09");
    assert_eq!(view.billing.refund_policy, "full_refund_before_filing");
    assert_eq!(view.billing.charges[0].kind, "setup");
    assert!(view.verification.verified);
    assert_eq!(view.carriers.total, 4);
    assert_eq!(view.carriers.by_carrier[0].label, "T-Mobile");
    assert!(view.submitted.is_none());
    assert!(view.payment.is_none());
}

#[tokio::test]
async fn test_application_update_puts_only_the_fields_set() {
    let mock_server = setup_mock_server().await;
    let mut saved = view_json();
    saved["ignoredFields"] = json!(["status"]);
    Mock::given(method("PUT"))
        .and(path("/short_codes/application"))
        .and(body_json(json!({
            "useCase": "Delivery alerts for Acme orders",
            "messageFrequency": "4 messages per month",
            "contentProviderSameAsBrand": true,
            "sampleMessages": ["Acme: your order shipped. Reply STOP to opt out."]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(saved))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let view = client
        .short_codes()
        .application()
        .update(
            ShortCodeApplicationInput::new()
                .use_case("Delivery alerts for Acme orders")
                .message_frequency("4 messages per month")
                .content_provider_same_as_brand(true)
                .sample_messages(["Acme: your order shipped. Reply STOP to opt out."]),
        )
        .await
        .unwrap();

    assert_eq!(view.ignored_fields, Some(vec!["status".to_string()]));
    let requests = mock_server.received_requests().await.unwrap();
    assert!(requests[0].headers.contains_key(&"idempotency-key".into()));
}

#[tokio::test]
async fn test_application_update_with_options_sends_the_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PUT"))
        .and(path("/short_codes/application"))
        .and(header("Idempotency-Key", "save-1"))
        .and(body_json(json!({ "campaignKeyword": "JOIN" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(view_json()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    client
        .short_codes()
        .application()
        .update_with_options(
            ShortCodeApplicationInput::new().campaign_keyword("JOIN"),
            IdempotentRequestOptions::new().idempotency_key("save-1"),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_application_update_locked_is_an_api_error() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("PUT"))
        .and(path("/short_codes/application"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "short_code_locked",
            "message": "Sendly is reviewing this application."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client
        .short_codes()
        .application()
        .update(ShortCodeApplicationInput::new().use_case("x"))
        .await
        .unwrap_err();

    assert_eq!(err.code(), Some("short_code_locked"));
}

#[tokio::test]
async fn test_application_check_posts_unsaved_answers_and_decodes_issues() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/short_codes/application/preflight"))
        .and(body_json(
            json!({ "helpResponse": "Acme: help at acme.example" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": false,
            "issues": [{ "path": "helpResponse", "message": "Add how to stop." }],
            "requiredDocuments": ["order_brief"],
            "missingDocuments": ["order_brief"],
            "documents": []
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let preflight = client
        .short_codes()
        .application()
        .check(ShortCodeApplicationInput::new().help_response("Acme: help at acme.example"))
        .await
        .unwrap();

    assert!(!preflight.ok);
    assert_eq!(preflight.issues[0].path, "helpResponse");
    assert_eq!(preflight.issues[0].message, "Add how to stop.");
    assert_eq!(preflight.missing_documents, vec!["order_brief"]);
    assert!(preflight.ignored_fields.is_none());
}

#[tokio::test]
async fn test_application_check_with_nothing_posts_an_empty_object() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/short_codes/application/preflight"))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "issues": [],
            "requiredDocuments": [],
            "missingDocuments": [],
            "documents": []
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let preflight = client
        .short_codes()
        .application()
        .check(ShortCodeApplicationInput::new())
        .await
        .unwrap();
    assert!(preflight.ok);
}

#[tokio::test]
async fn test_application_submit_sends_accept_terms_with_final_answers() {
    let mock_server = setup_mock_server().await;
    let mut submitted = view_json();
    submitted["submitted"] = json!(true);
    submitted["alreadySubmitted"] = json!(false);
    submitted["notified"] = json!(true);
    submitted["payment"] = json!({ "status": "paid", "charged": true });
    Mock::given(method("POST"))
        .and(path("/short_codes/application/submit"))
        .and(body_json(json!({
            "campaignKeyword": "JOIN",
            "acceptTerms": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(submitted))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .short_codes()
        .application()
        .submit(
            SubmitShortCodeApplicationRequest::new()
                .application(ShortCodeApplicationInput::new().campaign_keyword("JOIN"))
                .accept_terms(true),
        )
        .await
        .unwrap();

    assert_eq!(result.submitted, Some(true));
    assert_eq!(result.already_submitted, Some(false));
    assert_eq!(result.notified, Some(true));
    let payment = result.payment.unwrap();
    assert_eq!(payment.status, "paid");
    assert!(payment.charged);
}

#[tokio::test]
async fn test_application_submit_with_options_sends_the_key() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/short_codes/application/submit"))
        .and(header("Idempotency-Key", "submit-1"))
        .and(body_json(json!({ "acceptTerms": true })))
        .respond_with(ResponseTemplate::new(200).set_body_json(view_json()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    client
        .short_codes()
        .application()
        .submit_with_options(
            SubmitShortCodeApplicationRequest::new().accept_terms(true),
            IdempotentRequestOptions::new().idempotency_key("submit-1"),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_short_codes_not_enabled_is_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/short_codes"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "short_codes_not_enabled",
            "message": "Short codes are not enabled for this account."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let err = client.short_codes().list().await.unwrap_err();
    assert!(matches!(err, Error::NotFound { .. }));
}
