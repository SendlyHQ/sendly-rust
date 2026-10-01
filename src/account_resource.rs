//! Account resource for managing account information and credits.

use crate::client::{path_id, Sendly};
use crate::error::Error;
use crate::error::Result;
use crate::models::{
    Account, AccountBusinessVerification, AccountLimits, AccountOrganization, ApiKey,
    CreateApiKeyRequest, CreateApiKeyResponse, CreditTransactionList, Credits,
    ListTransactionsOptions, RotateApiKeyRequest, RotateApiKeyResponse, TransferCreditsRequest,
    TransferCreditsResponse,
};
use serde::Deserialize;

/// Account resource for managing account information and credits.
pub struct AccountResource<'a> {
    client: &'a Sendly,
}

#[derive(Debug, Deserialize)]
struct AccountResponse {
    #[serde(default)]
    account: Option<Account>,
    #[serde(default)]
    data: Option<Account>,
    #[serde(default)]
    user: Option<AccountUserWire>,
    #[serde(default)]
    organization: Option<AccountOrganization>,
    #[serde(default)]
    verification: Option<serde_json::Value>,
    #[serde(default)]
    limits: Option<AccountLimits>,
}

#[derive(Debug, Deserialize)]
struct AccountUserWire {
    id: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default, alias = "createdAt")]
    created_at: Option<String>,
}

impl AccountResponse {
    #[allow(deprecated)]
    fn into_account(self) -> Result<Account> {
        if let Some(account) = self.account.or(self.data) {
            return Ok(account);
        }
        let user = self.user.ok_or_else(|| {
            Error::Json(<serde_json::Error as serde::de::Error>::custom(
                "the account response has no user",
            ))
        })?;
        Ok(Account {
            id: user.id,
            email: user.email.unwrap_or_default(),
            name: None,
            company_name: None,
            verification: Default::default(),
            limits: self.limits.unwrap_or_default(),
            created_at: user.created_at,
            organization: self.organization,
            business_verification: self
                .verification
                .and_then(|v| serde_json::from_value::<AccountBusinessVerification>(v).ok()),
        })
    }
}

#[derive(Debug, Deserialize)]
struct CreditsResponse {
    #[serde(default)]
    credits: Option<Credits>,
    #[serde(default)]
    data: Option<Credits>,
    #[serde(flatten)]
    flat: Option<Credits>,
}

#[derive(Debug, Deserialize)]
struct ApiKeyListResponse {
    /// The API answers `{"keys": [...]}`; the other spellings are kept so an
    /// older or aliased response still decodes.
    #[serde(default)]
    keys: Option<Vec<ApiKey>>,
    #[serde(default, alias = "apiKeys")]
    api_keys: Option<Vec<ApiKey>>,
    #[serde(default)]
    data: Option<Vec<ApiKey>>,
}

#[derive(Debug, Deserialize)]
struct ApiKeyResponse {
    #[serde(default, alias = "apiKey")]
    api_key: Option<ApiKey>,
    #[serde(default)]
    data: Option<ApiKey>,
}

/// Usage statistics for an API key, over its most recent 100 requests.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ApiKeyUsage {
    /// Number of requests made with this key, counting at most its last 100.
    #[serde(default, alias = "totalRequests")]
    pub total_requests: i64,
    /// Number of successful requests.
    #[deprecated(
        note = "The API does not report this; it is always 0. Count `recent_requests` by `status_code`."
    )]
    #[serde(default, alias = "successfulRequests")]
    pub successful_requests: i64,
    /// Number of failed requests.
    #[deprecated(
        note = "The API does not report this; it is always 0. Count `recent_requests` by `status_code`."
    )]
    #[serde(default, alias = "failedRequests")]
    pub failed_requests: i64,
    /// When the key was last used.
    #[serde(default, alias = "lastRequestAt", alias = "lastUsed")]
    pub last_request_at: Option<String>,
    /// Credits the counted requests used.
    #[serde(default, alias = "creditsUsed", alias = "totalCredits")]
    pub credits_used: i64,
    /// The key's 20 most recent requests, newest first.
    #[serde(default, alias = "recentRequests")]
    pub recent_requests: Vec<ApiKeyRequestRecord>,
    /// How many of the counted requests went to each endpoint, busiest
    /// first.
    #[serde(default, alias = "endpointBreakdown")]
    pub endpoint_breakdown: Vec<ApiKeyEndpointCount>,
}

/// One request made with an API key.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ApiKeyRequestRecord {
    /// The path requested, such as `/api/v1/messages`.
    #[serde(default)]
    pub endpoint: String,
    /// The HTTP method.
    #[serde(default)]
    pub method: String,
    /// The HTTP status the API answered with.
    #[serde(default)]
    pub status_code: Option<i32>,
    /// Credits the request used.
    #[serde(default)]
    pub credits_used: i64,
    /// When the request was made.
    #[serde(default)]
    pub created_at: Option<String>,
}

/// How many requests an API key made to one endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
#[non_exhaustive]
pub struct ApiKeyEndpointCount {
    /// The method and path, such as `POST /api/v1/messages`.
    #[serde(default)]
    pub endpoint: String,
    /// Number of requests.
    #[serde(default)]
    pub count: i64,
}

#[derive(Debug, Deserialize)]
struct ApiKeyUsageResponse {
    #[serde(default)]
    usage: Option<ApiKeyUsage>,
    #[serde(default)]
    data: Option<ApiKeyUsage>,
    #[serde(default)]
    summary: Option<ApiKeyUsage>,
    #[serde(default, alias = "recentRequests")]
    recent_requests: Option<Vec<ApiKeyRequestRecord>>,
    #[serde(default, alias = "endpointBreakdown")]
    endpoint_breakdown: Option<Vec<ApiKeyEndpointCount>>,
}

impl<'a> AccountResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Gets current account information.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let account = client.account().get().await?;
    /// println!("Account: {} ({})", account.id, account.email);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self) -> Result<Account> {
        let response = self.client.get("/account", &[]).await?;
        let result: AccountResponse = response.json().await?;

        result.into_account()
    }

    /// Gets current credit balance.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let credits = client.account().credits().await?;
    /// println!("Balance: {} credits", credits.available_balance);
    /// # Ok(())
    /// # }
    /// ```
    #[allow(deprecated)]
    pub async fn credits(&self) -> Result<Credits> {
        let response = self.client.get("/account/credits", &[]).await?;
        let result: CreditsResponse = response.json().await?;

        Ok(result
            .credits
            .or(result.data)
            .or(result.flat)
            .unwrap_or_else(|| Credits {
                balance: 0,
                available_balance: 0,
                pending_credits: 0,
                reserved_credits: 0,
                billing_mode: None,
                currency: "USD".to_string(),
            }))
    }

    /// Lists credit transactions.
    ///
    /// # Arguments
    ///
    /// * `options` - Query options
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, ListTransactionsOptions};
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let transactions = client.account().transactions(
    ///     Some(ListTransactionsOptions::new().limit(10))
    /// ).await?;
    ///
    /// for tx in transactions.data {
    ///     println!("{}: {} credits", tx.id, tx.amount);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn transactions(
        &self,
        options: Option<ListTransactionsOptions>,
    ) -> Result<CreditTransactionList> {
        let query = options.unwrap_or_default().to_query_params();
        let response = self.client.get("/credits/transactions", &query).await?;
        let result: CreditTransactionList = response.json().await?;
        Ok(result)
    }

    pub async fn transfer_credits(
        &self,
        request: TransferCreditsRequest,
    ) -> Result<TransferCreditsResponse> {
        let response = self.client.post("/credits/transfer", &request).await?;
        let result: TransferCreditsResponse = response.json().await?;
        Ok(result)
    }

    /// Lists API keys.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let keys = client.account().api_keys().await?;
    /// for key in keys {
    ///     println!("Key: {} ({})", key.name, key.prefix);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn api_keys(&self) -> Result<Vec<ApiKey>> {
        let response = self.client.get("/account/keys", &[]).await?;
        let result: ApiKeyListResponse = response.json().await?;

        Ok(result
            .keys
            .or(result.api_keys)
            .or(result.data)
            .unwrap_or_default())
    }

    /// Creates a new API key.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for the API key
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let response = client.account().create_api_key("Production").await?;
    /// println!("New key: {}", response.key);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_api_key(&self, name: impl Into<String>) -> Result<CreateApiKeyResponse> {
        self.create_api_key_with_options(CreateApiKeyRequest::new(name))
            .await
    }

    /// Creates a new API key with full options: its type (`"test"` or
    /// `"live"`), scopes and expiry.
    ///
    /// Without a type the API creates a test key. A live key needs a verified
    /// business and a credit balance: the API answers 403
    /// `verification_required` or 402 `credits_required` otherwise. A key
    /// can only grant scopes it has itself (403 `insufficient_permissions`),
    /// and without scopes the new key gets the calling key's.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{CreateApiKeyRequest, Sendly};
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let created = client
    ///     .account()
    ///     .create_api_key_with_options(
    ///         CreateApiKeyRequest::new("Production")
    ///             .key_type("live")
    ///             .scopes(vec!["sms:send", "sms:read"]),
    ///     )
    ///     .await?;
    /// println!("New key: {}", created.key);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_api_key_with_options(
        &self,
        request: CreateApiKeyRequest,
    ) -> Result<CreateApiKeyResponse> {
        let response = self.client.post("/account/keys", &request).await?;
        let result: CreateApiKeyResponse = response.json().await?;
        Ok(result)
    }

    /// Gets a specific API key by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - API key ID
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let key = client.account().get_api_key("key_abc123").await?;
    /// println!("Key: {} ({})", key.name, key.prefix);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_api_key(&self, id: impl AsRef<str>) -> Result<ApiKey> {
        let path = format!("/account/keys/{}", path_id(id.as_ref())?);
        let response = self.client.get(&path, &[]).await?;
        // The API returns the key object unwrapped; older shapes wrapped it in
        // apiKey/data, so try those first and fall back to the bare object.
        let body: serde_json::Value = response.json().await?;
        if let Ok(result) = serde_json::from_value::<ApiKeyResponse>(body.clone()) {
            if let Some(key) = result.api_key.or(result.data) {
                return Ok(key);
            }
        }
        Ok(serde_json::from_value(body).unwrap_or_default())
    }

    /// Gets usage statistics for a specific API key.
    ///
    /// # Arguments
    ///
    /// * `id` - API key ID
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let usage = client.account().get_api_key_usage("key_abc123").await?;
    /// println!("Requests: {}", usage.total_requests);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_api_key_usage(&self, id: impl AsRef<str>) -> Result<ApiKeyUsage> {
        let path = format!("/account/keys/{}/usage", path_id(id.as_ref())?);
        let response = self.client.get(&path, &[]).await?;
        let result: ApiKeyUsageResponse = response.json().await?;
        let mut usage = result
            .usage
            .or(result.data)
            .or(result.summary)
            .unwrap_or_default();
        if let Some(recent_requests) = result.recent_requests {
            usage.recent_requests = recent_requests;
        }
        if let Some(endpoint_breakdown) = result.endpoint_breakdown {
            usage.endpoint_breakdown = endpoint_breakdown;
        }
        Ok(usage)
    }

    /// Revokes an API key.
    ///
    /// # Arguments
    ///
    /// * `id` - API key ID
    pub async fn revoke_api_key(&self, id: impl AsRef<str>) -> Result<()> {
        let path = format!("/account/keys/{}/revoke", path_id(id.as_ref())?);
        self.client.patch(&path, &serde_json::json!({})).await?;
        Ok(())
    }

    /// Rotates an API key, using the default 24-hour grace period.
    ///
    /// Issues a new key and keeps the old one working for a grace period so you
    /// can roll callers over without downtime. The new secret's raw value is on
    /// [`RotatedApiKey::secret`](crate::RotatedApiKey::secret) and is shown only
    /// once — store it now.
    ///
    /// # Arguments
    ///
    /// * `id` - API key ID to rotate
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let result = client.account().rotate_api_key("key_abc123").await?;
    /// println!("New key (save it!): {}", result.new_key.secret);
    /// println!("{}", result.message);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rotate_api_key(&self, id: impl AsRef<str>) -> Result<RotateApiKeyResponse> {
        self.rotate_api_key_with_options(id, RotateApiKeyRequest::default())
            .await
    }

    /// Rotates an API key with a custom grace period (24-168 hours).
    ///
    /// # Arguments
    ///
    /// * `id` - API key ID to rotate
    /// * `request` - Rotation options (grace period)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, RotateApiKeyRequest};
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let result = client.account().rotate_api_key_with_options(
    ///     "key_abc123",
    ///     RotateApiKeyRequest::new().grace_period_hours(72),
    /// ).await?;
    /// println!("New key: {}", result.new_key.secret);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rotate_api_key_with_options(
        &self,
        id: impl AsRef<str>,
        request: RotateApiKeyRequest,
    ) -> Result<RotateApiKeyResponse> {
        let id = id.as_ref();
        if id.is_empty() {
            return Err(Error::validation("API key ID is required"));
        }
        let path = format!("/account/keys/{}/rotate", path_id(id)?);
        let response = self.client.post(&path, &request).await?;
        let result: RotateApiKeyResponse = response.json().await?;
        Ok(result)
    }
}
