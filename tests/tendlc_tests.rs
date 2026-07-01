mod common;

use common::{create_test_client, setup_mock_server, TEST_API_KEY};
use sendly::{CreateTenDlcBrandRequest, CreateTenDlcCampaignRequest, Error};
use serde_json::json;
use wiremock::matchers::{body_json, body_partial_json, header, method, path};
use wiremock::{Mock, ResponseTemplate};

// ==================== list_brands() Tests ====================

#[tokio::test]
async fn test_list_brands_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/tendlc/brands"))
        .and(header(
            "Authorization",
            format!("Bearer {}", TEST_API_KEY).as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "brd_abc123",
                    "legalName": "Acme Holdings LLC",
                    "dba": "Acme",
                    "entityType": "PRIVATE_PROFIT",
                    "ein": "12-3456789",
                    "vertical": "RETAIL",
                    "website": "https://acme.example",
                    "status": "verified",
                    "identityStatus": "VERIFIED",
                    "failureReasons": null,
                    "createdAt": "2026-06-01T10:00:00Z",
                    "updatedAt": "2026-06-02T10:00:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.ten_dlc().list_brands().await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 1);
    let b = &response.data[0];
    assert_eq!(b.id, "brd_abc123");
    assert_eq!(b.legal_name, "Acme Holdings LLC");
    assert_eq!(b.dba.as_deref(), Some("Acme"));
    assert_eq!(b.entity_type, "PRIVATE_PROFIT");
    assert_eq!(b.ein.as_deref(), Some("12-3456789"));
    assert_eq!(b.vertical.as_deref(), Some("RETAIL"));
    assert_eq!(b.website.as_deref(), Some("https://acme.example"));
    assert_eq!(b.status, "verified");
    assert_eq!(b.identity_status.as_deref(), Some("VERIFIED"));
    assert!(b.failure_reasons.is_none());
    assert_eq!(b.created_at, "2026-06-01T10:00:00Z");
    assert_eq!(b.updated_at, "2026-06-02T10:00:00Z");
}

// ==================== create_brand() Tests ====================

#[tokio::test]
async fn test_create_brand_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/tendlc/brands"))
        .and(body_partial_json(json!({
            "legalName": "Acme Holdings LLC",
            "ein": "12-3456789",
            "entityType": "PRIVATE_PROFIT",
            "website": "https://acme.example",
            "email": "ops@acme.example",
            "mobilePhone": "+15551234567",
            "postalCode": "94105",
            "verificationId": "ver_abc123"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "data": {
                "id": "brd_abc123",
                "legalName": "Acme Holdings LLC",
                "dba": null,
                "entityType": "PRIVATE_PROFIT",
                "ein": "12-3456789",
                "vertical": null,
                "website": "https://acme.example",
                "status": "pending",
                "identityStatus": null,
                "failureReasons": null,
                "createdAt": "2026-06-01T10:00:00Z",
                "updatedAt": "2026-06-01T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .ten_dlc()
        .create_brand(
            CreateTenDlcBrandRequest::new("Acme Holdings LLC")
                .ein("12-3456789")
                .entity_type("PRIVATE_PROFIT")
                .website("https://acme.example")
                .email("ops@acme.example")
                .mobile_phone("+15551234567")
                .postal_code("94105")
                .verification_id("ver_abc123"),
        )
        .await;

    assert!(result.is_ok());
    let brand = result.unwrap().data;
    assert_eq!(brand.id, "brd_abc123");
    assert_eq!(brand.status, "pending");
    assert!(brand.dba.is_none());
    assert!(brand.identity_status.is_none());
}

#[tokio::test]
async fn test_create_brand_minimal_body_omits_unset_fields() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/tendlc/brands"))
        .and(body_json(json!({ "legalName": "Acme Holdings LLC" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "data": {
                "id": "brd_abc123",
                "legalName": "Acme Holdings LLC",
                "dba": null,
                "entityType": "PRIVATE_PROFIT",
                "ein": null,
                "vertical": null,
                "website": null,
                "status": "pending",
                "identityStatus": null,
                "failureReasons": null,
                "createdAt": "2026-06-01T10:00:00Z",
                "updatedAt": "2026-06-01T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .ten_dlc()
        .create_brand(CreateTenDlcBrandRequest::new("Acme Holdings LLC"))
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().data.status, "pending");
}

#[tokio::test]
async fn test_create_brand_carrier_rejection() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/tendlc/brands"))
        .respond_with(ResponseTemplate::new(502).set_body_json(json!({
            "error": "brand_create_failed",
            "message": "The carrier network could not process the registration"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .ten_dlc()
        .create_brand(CreateTenDlcBrandRequest::new("Acme Holdings LLC"))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api {
            message,
            status_code,
            ..
        } => {
            assert_eq!(status_code, 502);
            assert!(message.contains("carrier network"));
        }
        _ => panic!("Expected Api error"),
    }
}

// ==================== get_brand() Tests ====================

#[tokio::test]
async fn test_get_brand_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/tendlc/brands/brd_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "id": "brd_abc123",
                "legalName": "Acme Holdings LLC",
                "dba": null,
                "entityType": "PRIVATE_PROFIT",
                "ein": null,
                "vertical": null,
                "website": null,
                "status": "failed",
                "identityStatus": "UNVERIFIED",
                "failureReasons": ["Business address could not be confirmed"],
                "createdAt": "2026-06-01T10:00:00Z",
                "updatedAt": "2026-06-03T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.ten_dlc().get_brand("brd_abc123").await;

    assert!(result.is_ok());
    let brand = result.unwrap().data;
    assert_eq!(brand.status, "failed");
    assert_eq!(
        brand.failure_reasons,
        Some(vec!["Business address could not be confirmed".to_string()])
    );
}

#[tokio::test]
async fn test_get_brand_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/tendlc/brands/brd_missing"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "not_found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.ten_dlc().get_brand("brd_missing").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NotFound { .. }));
}

// ==================== qualify() Tests ====================

#[tokio::test]
async fn test_qualify_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/tendlc/brands/brd_abc123/qualify/MIXED"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "useCase": "MIXED",
                "qualified": true,
                "reason": null,
                "throughput": {
                    "tier": "Standard",
                    "carriersReady": 3
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.ten_dlc().qualify("brd_abc123", "MIXED").await;

    assert!(result.is_ok());
    let check = result.unwrap().data;
    assert_eq!(check.use_case, "MIXED");
    assert!(check.qualified);
    assert!(check.reason.is_none());
    let throughput = check.throughput.expect("throughput should be present");
    assert_eq!(throughput.tier, "Standard");
    assert_eq!(throughput.carriers_ready, 3);
}

#[tokio::test]
async fn test_qualify_not_qualified() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/tendlc/brands/brd_abc123/qualify/MARKETING"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "useCase": "MARKETING",
                "qualified": false,
                "reason": "This use case is not available for the brand",
                "throughput": null
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.ten_dlc().qualify("brd_abc123", "MARKETING").await;

    assert!(result.is_ok());
    let check = result.unwrap().data;
    assert!(!check.qualified);
    assert_eq!(
        check.reason.as_deref(),
        Some("This use case is not available for the brand")
    );
    assert!(check.throughput.is_none());
}

// ==================== list_campaigns() Tests ====================

#[tokio::test]
async fn test_list_campaigns_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/tendlc/campaigns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "cmp_abc123",
                    "brandId": "brd_abc123",
                    "useCase": "MIXED",
                    "subUseCases": ["ACCOUNT_NOTIFICATION"],
                    "description": "Order updates and support replies",
                    "status": "active",
                    "sampleMessages": ["Your order #123 has shipped!"],
                    "throughput": {
                        "tier": "Standard",
                        "carriersReady": 4
                    },
                    "failureReasons": null,
                    "createdAt": "2026-06-01T10:00:00Z",
                    "updatedAt": "2026-06-05T10:00:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.ten_dlc().list_campaigns().await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 1);
    let c = &response.data[0];
    assert_eq!(c.id, "cmp_abc123");
    assert_eq!(c.brand_id, "brd_abc123");
    assert_eq!(c.use_case, "MIXED");
    assert_eq!(c.sub_use_cases, vec!["ACCOUNT_NOTIFICATION"]);
    assert_eq!(c.description.as_deref(), Some("Order updates and support replies"));
    assert_eq!(c.status, "active");
    assert_eq!(c.sample_messages, vec!["Your order #123 has shipped!"]);
    assert_eq!(c.throughput.as_ref().unwrap().carriers_ready, 4);
}

// ==================== create_campaign() Tests ====================

#[tokio::test]
async fn test_create_campaign_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/tendlc/campaigns"))
        .and(body_partial_json(json!({
            "brandId": "brd_abc123",
            "useCase": "MIXED",
            "description": "Order updates and support replies",
            "messageFlow": "Customers opt in at checkout",
            "sampleMessages": ["Your order #123 has shipped!"],
            "optOutKeywords": "STOP",
            "embeddedLink": true
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "data": {
                "id": "cmp_abc123",
                "brandId": "brd_abc123",
                "useCase": "MIXED",
                "subUseCases": [],
                "description": "Order updates and support replies",
                "status": "pending",
                "sampleMessages": ["Your order #123 has shipped!"],
                "throughput": null,
                "failureReasons": null,
                "createdAt": "2026-06-01T10:00:00Z",
                "updatedAt": "2026-06-01T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .ten_dlc()
        .create_campaign(
            CreateTenDlcCampaignRequest::new(
                "brd_abc123",
                "MIXED",
                "Order updates and support replies",
                "Customers opt in at checkout",
                vec!["Your order #123 has shipped!".to_string()],
            )
            .opt_out_keywords("STOP")
            .embedded_link(true),
        )
        .await;

    assert!(result.is_ok());
    let campaign = result.unwrap().data;
    assert_eq!(campaign.id, "cmp_abc123");
    assert_eq!(campaign.status, "pending");
    assert!(campaign.throughput.is_none());
}

#[tokio::test]
async fn test_create_campaign_brand_not_verified() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/tendlc/campaigns"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "brand_not_verified",
            "message": "The brand must be verified before creating a campaign"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .ten_dlc()
        .create_campaign(CreateTenDlcCampaignRequest::new(
            "brd_abc123",
            "MIXED",
            "Order updates",
            "Customers opt in at checkout",
            vec!["Your order #123 has shipped!".to_string()],
        ))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("verified")),
        _ => panic!("Expected Validation error"),
    }
}

// ==================== get_campaign() Tests ====================

#[tokio::test]
async fn test_get_campaign_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/tendlc/campaigns/cmp_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "id": "cmp_abc123",
                "brandId": "brd_abc123",
                "useCase": "MIXED",
                "subUseCases": [],
                "description": "Order updates and support replies",
                "status": "active",
                "sampleMessages": ["Your order #123 has shipped!"],
                "throughput": {
                    "tier": "High volume",
                    "carriersReady": 5
                },
                "failureReasons": null,
                "createdAt": "2026-06-01T10:00:00Z",
                "updatedAt": "2026-06-05T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.ten_dlc().get_campaign("cmp_abc123").await;

    assert!(result.is_ok());
    let campaign = result.unwrap().data;
    assert_eq!(campaign.status, "active");
    let throughput = campaign.throughput.expect("throughput should be present");
    assert_eq!(throughput.tier, "High volume");
    assert_eq!(throughput.carriers_ready, 5);
}

#[tokio::test]
async fn test_get_campaign_not_found() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/tendlc/campaigns/cmp_missing"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "not_found"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.ten_dlc().get_campaign("cmp_missing").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NotFound { .. }));
}

// ==================== assign_number() Tests ====================

#[tokio::test]
async fn test_assign_number_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/tendlc/campaigns/cmp_abc123/assign"))
        .and(body_json(json!({ "phoneNumber": "+15551234567" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "data": {
                "id": "asg_abc123",
                "campaignId": "cmp_abc123",
                "phoneNumber": "+15551234567",
                "status": "Under review",
                "assignedAt": null
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .ten_dlc()
        .assign_number("cmp_abc123", "+15551234567")
        .await;

    assert!(result.is_ok());
    let assignment = result.unwrap().data;
    assert_eq!(assignment.id, "asg_abc123");
    assert_eq!(assignment.campaign_id, "cmp_abc123");
    assert_eq!(assignment.phone_number, "+15551234567");
    assert_eq!(assignment.status, "Under review");
    assert!(assignment.assigned_at.is_none());
}

#[tokio::test]
async fn test_assign_number_already_assigned() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/tendlc/campaigns/cmp_abc123/assign"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "number_already_assigned",
            "message": "This number is already assigned to another campaign"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .ten_dlc()
        .assign_number("cmp_abc123", "+15551234567")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Api {
            message,
            status_code,
            ..
        } => {
            assert_eq!(status_code, 409);
            assert!(message.contains("already assigned"));
        }
        _ => panic!("Expected Api error"),
    }
}

#[tokio::test]
async fn test_assign_number_campaign_not_active() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/tendlc/campaigns/cmp_abc123/assign"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "campaign_not_active",
            "message": "The campaign must be active before assigning numbers"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client
        .ten_dlc()
        .assign_number("cmp_abc123", "+15551234567")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Validation { message } => assert!(message.contains("active")),
        _ => panic!("Expected Validation error"),
    }
}

// ==================== list_assignments() Tests ====================

#[tokio::test]
async fn test_list_assignments_success() {
    let mock_server = setup_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/tendlc/assignments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "asg_abc123",
                    "campaignId": "cmp_abc123",
                    "phoneNumber": "+15551234567",
                    "status": "Active",
                    "assignedAt": "2026-06-05T10:00:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let result = client.ten_dlc().list_assignments().await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 1);
    let a = &response.data[0];
    assert_eq!(a.id, "asg_abc123");
    assert_eq!(a.campaign_id, "cmp_abc123");
    assert_eq!(a.phone_number, "+15551234567");
    assert_eq!(a.status, "Active");
    assert_eq!(a.assigned_at.as_deref(), Some("2026-06-05T10:00:00Z"));
}
