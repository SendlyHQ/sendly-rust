//! Numbers Resource — Phone Number Discovery & Provisioning
//!
//! Browse the countries and number types Sendly can provision, search
//! available numbers (already priced for your account), list the numbers you
//! own, and buy a new one.
//!
//! Buying a number is asynchronous. [`NumbersResource::buy`] returns `202`
//! with a [`BuyNumberResponse::status`]:
//!
//! - `provisioning` — the number is being set up; poll [`NumbersResource::list`]
//!   until it appears as active.
//! - `documents_required` / `payment_required` — the purchase needs the user
//!   to finish something on a hosted Sendly page first. The response carries an
//!   [`NumberBuyAction`] with a `url` (the hosted page) and a short `code` the
//!   user types to prove they have terminal access. Hand the user the URL +
//!   code, wait for the action to complete, then call `buy()` again with the
//!   SAME body plus `action_code` set to the completed action's code.
//!
//! See <https://sendly.live/docs/numbers> for the full flow.

use serde::{Deserialize, Serialize};

use crate::client::Sendly;
use crate::error::{Error, Result};

/// A country Sendly can provision numbers in.
#[derive(Debug, Clone, Deserialize)]
pub struct NumberCountry {
    /// ISO 3166-1 alpha-2 country code (e.g. `GB`).
    pub code: String,
    /// Human-readable country name.
    pub name: String,
    /// Number types available in this country (e.g. `["mobile", "local"]`).
    #[serde(default, alias = "numberTypes")]
    pub number_types: Vec<String>,
}

/// Response from [`NumbersResource::list_countries`].
#[derive(Debug, Clone, Deserialize)]
pub struct NumberCountriesResponse {
    #[serde(default)]
    pub countries: Vec<NumberCountry>,
}

/// A number available to buy. `monthly_cost` is already priced for your
/// account (a display string).
#[derive(Debug, Clone, Deserialize)]
pub struct AvailableNumber {
    /// Phone number in E.164 format.
    #[serde(alias = "phoneNumber")]
    pub phone_number: String,
    /// ISO 3166-1 alpha-2 country code.
    pub country: String,
    /// Number type (e.g. `mobile`, `local`, `toll_free`).
    #[serde(alias = "numberType")]
    pub number_type: String,
    /// Customer-facing monthly cost (already marked up), as a string.
    #[serde(alias = "monthlyCost")]
    pub monthly_cost: String,
    /// ISO 4217 currency code for `monthly_cost` (e.g. `USD`).
    pub currency: String,
}

/// Response from [`NumbersResource::list_available`].
#[derive(Debug, Clone, Deserialize)]
pub struct AvailableNumbersResponse {
    #[serde(default)]
    pub numbers: Vec<AvailableNumber>,
}

/// A phone number you own.
#[derive(Debug, Clone, Deserialize)]
pub struct OwnedNumber {
    /// Unique number identifier.
    pub id: String,
    /// Phone number in E.164 format.
    #[serde(alias = "phoneNumber")]
    pub phone_number: String,
    /// Provisioning / lifecycle status.
    pub status: String,
    /// How the number was acquired (e.g. `purchased`, `ported`).
    ///
    /// Absent on the trimmed `number` returned by a buy response.
    #[serde(default)]
    pub source: Option<String>,
    /// ISO 3166-1 alpha-2 country code.
    ///
    /// Absent on the trimmed `number` returned by a buy response.
    #[serde(default, alias = "countryCode")]
    pub country_code: Option<String>,
    /// Number type (e.g. `mobile`, `local`, `toll_free`).
    ///
    /// Absent on the trimmed `number` returned by a buy response.
    #[serde(default, alias = "phoneNumberType")]
    pub phone_number_type: Option<String>,
    /// Monthly cost in cents.
    #[serde(default, alias = "monthlyCostCents")]
    pub monthly_cost_cents: i64,
}

/// Response from [`NumbersResource::list`].
#[derive(Debug, Clone, Deserialize)]
pub struct OwnedNumbersResponse {
    #[serde(default)]
    pub numbers: Vec<OwnedNumber>,
}

/// Hosted-page hand-off returned when a buy needs the user to finish a step.
///
/// Hand the user the [`url`](Self::url) and the short [`code`](Self::code)
/// (the code proves they have terminal access). Once they complete the hosted
/// page, re-call [`NumbersResource::buy`] with the same body plus
/// `action_code` set to this code.
#[derive(Debug, Clone, Deserialize)]
pub struct NumberBuyAction {
    /// Hosted Sendly page URL to send the user to.
    pub url: String,
    /// Short user code the user enters to prove terminal access. Display only —
    /// never pass this where `action_code` is expected.
    pub code: String,
    /// The action identifier (32-hex). Use this to poll the action's status and
    /// to re-call `buy()` (set it on [`BuyNumberRequest::action_code`]).
    #[serde(default, alias = "actionCode")]
    pub action_code: Option<String>,
    /// When this action expires, as epoch milliseconds.
    #[serde(default, alias = "expiresAt")]
    pub expires_at: Option<i64>,
}

/// Response from [`NumbersResource::buy`].
///
/// When `status` is `documents_required` or `payment_required`, `action`
/// carries the hosted-page hand-off (URL + code). Hand it to the user, wait
/// for completion, then call `buy()` again with the same body plus
/// `action_code` set to the completed action's code.
#[derive(Debug, Clone, Deserialize)]
pub struct BuyNumberResponse {
    /// Buy outcome: `provisioning`, `documents_required`, or `payment_required`.
    pub status: String,
    /// The provisioned number, when available.
    #[serde(default)]
    pub number: Option<OwnedNumber>,
    /// Outstanding requirements, when `status` is `documents_required`.
    #[serde(default)]
    pub requirements: Option<Vec<serde_json::Value>>,
    /// Hosted-page hand-off, when `status` requires user action.
    #[serde(default)]
    pub action: Option<NumberBuyAction>,
}

/// Options for [`NumbersResource::list_available`].
#[derive(Debug, Clone, Default)]
pub struct ListAvailableNumbersOptions {
    /// ISO 3166-1 alpha-2 country code to search in (e.g. `GB`).
    pub country: String,
    /// Number type to search for (e.g. `mobile`).
    pub r#type: String,
    /// Optional substring the number must contain (digits only).
    pub contains: Option<String>,
}

impl ListAvailableNumbersOptions {
    pub fn new(country: impl Into<String>, r#type: impl Into<String>) -> Self {
        Self {
            country: country.into(),
            r#type: r#type.into(),
            contains: None,
        }
    }

    pub fn contains(mut self, contains: impl Into<String>) -> Self {
        self.contains = Some(contains.into());
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("country".to_string(), self.country.clone()),
            ("type".to_string(), self.r#type.clone()),
        ];
        if let Some(ref contains) = self.contains {
            params.push(("contains".to_string(), contains.clone()));
        }
        params
    }
}

/// Request body for [`NumbersResource::buy`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuyNumberRequest {
    /// Phone number to buy, in E.164 format.
    pub phone_number: String,
    /// ISO 3166-1 alpha-2 country code.
    pub country_code: String,
    /// Number type (e.g. `mobile`, `local`, `toll_free`).
    pub phone_number_type: String,
    /// Customer-facing monthly cost from the available-numbers listing.
    pub monthly_cost: String,
    /// The code from a COMPLETED action. Set this only when re-calling `buy()`
    /// after the user finished a `documents_required` / `payment_required`
    /// hosted-page step.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_code: Option<String>,
}

impl BuyNumberRequest {
    pub fn new(
        phone_number: impl Into<String>,
        country_code: impl Into<String>,
        phone_number_type: impl Into<String>,
        monthly_cost: impl Into<String>,
    ) -> Self {
        Self {
            phone_number: phone_number.into(),
            country_code: country_code.into(),
            phone_number_type: phone_number_type.into(),
            monthly_cost: monthly_cost.into(),
            action_code: None,
        }
    }

    /// Set the code from a completed documents/payment action before
    /// re-calling `buy()`.
    pub fn action_code(mut self, action_code: impl Into<String>) -> Self {
        self.action_code = Some(action_code.into());
        self
    }
}

/// Numbers resource — discover, buy, and list phone numbers.
///
/// # Example
///
/// ```rust,no_run
/// use sendly::{Sendly, ListAvailableNumbersOptions, BuyNumberRequest};
///
/// # async fn run() -> Result<(), sendly::Error> {
/// let client = Sendly::new("sk_live_v1_xxx");
///
/// // 1) Browse what's available
/// let countries = client.numbers().list_countries().await?;
/// let available = client
///     .numbers()
///     .list_available(ListAvailableNumbersOptions::new("GB", "mobile"))
///     .await?;
///
/// // 2) Buy one
/// let first = &available.numbers[0];
/// let result = client
///     .numbers()
///     .buy(BuyNumberRequest::new(
///         &first.phone_number,
///         &first.country,
///         &first.number_type,
///         &first.monthly_cost,
///     ))
///     .await?;
///
/// if result.status == "provisioning" {
///     println!("Number is being set up");
/// } else if let Some(action) = result.action {
///     // documents_required / payment_required: send the user to the hosted page
///     println!("Finish at {} (code {})", action.url, action.code);
///     // ...after they complete it, re-call buy() with action_code set.
/// }
///
/// // 3) List the numbers you own
/// let owned = client.numbers().list().await?;
/// # Ok(()) }
/// ```
pub struct NumbersResource<'a> {
    client: &'a Sendly,
}

impl<'a> NumbersResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// List the countries Sendly can provision numbers in, with the number
    /// types available in each.
    pub async fn list_countries(&self) -> Result<NumberCountriesResponse> {
        let response = self.client.get("/numbers/countries", &[]).await?;
        Ok(response.json().await?)
    }

    /// Search numbers available to buy in a country. Prices are already
    /// customer-priced for your account.
    pub async fn list_available(
        &self,
        options: ListAvailableNumbersOptions,
    ) -> Result<AvailableNumbersResponse> {
        if options.country.is_empty() {
            return Err(Error::Validation {
                message: "list_available requires a country".to_string(),
            });
        }
        if options.r#type.is_empty() {
            return Err(Error::Validation {
                message: "list_available requires a type".to_string(),
            });
        }
        let params = options.to_query_params();
        let response = self.client.get("/numbers/available", &params).await?;
        Ok(response.json().await?)
    }

    /// List the phone numbers you own.
    pub async fn list(&self) -> Result<OwnedNumbersResponse> {
        let response = self.client.get("/numbers", &[]).await?;
        Ok(response.json().await?)
    }

    /// Buy a phone number. Asynchronous — returns `202` with a status.
    ///
    /// When the status is `documents_required` or `payment_required`, the
    /// response carries an [`NumberBuyAction`] hand-off (hosted-page URL +
    /// short code). Hand it to the user, wait for them to complete it, then
    /// call `buy()` again with the SAME body plus `action_code` set to the
    /// completed action's code.
    pub async fn buy(&self, request: BuyNumberRequest) -> Result<BuyNumberResponse> {
        let response = self.client.post("/numbers/buy", &request).await?;
        Ok(response.json().await?)
    }
}
