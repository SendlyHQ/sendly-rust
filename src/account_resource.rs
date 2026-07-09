//! Account resource for managing account information and credits.

use crate::client::Sendly;
use crate::error::Result;
use crate::error::Error;
use crate::models::{
    Account, ApiKey, CreateApiKeyRequest, CreateApiKeyResponse, CreditTransactionList, Credits,
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

/// Usage statistics for an API key.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ApiKeyUsage {
    /// Total number of requests made with this key.
    #[serde(default, alias = "totalRequests")]
    pub total_requests: i64,
    /// Number of successful requests.
    #[serde(default, alias = "successfulRequests")]
    pub successful_requests: i64,
    /// Number of failed requests.
    #[serde(default, alias = "failedRequests")]
    pub failed_requests: i64,
    /// Last request timestamp.
    #[serde(default, alias = "lastRequestAt")]
    pub last_request_at: Option<String>,
    /// Credits used by this key.
    #[serde(default, alias = "creditsUsed")]
    pub credits_used: i64,
}

#[derive(Debug, Deserialize)]
struct ApiKeyUsageResponse {
    #[serde(default)]
    usage: Option<ApiKeyUsage>,
    #[serde(default)]
    data: Option<ApiKeyUsage>,
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

        Ok(result.account.or(result.data).unwrap_or_else(|| Account {
            id: String::new(),
            email: String::new(),
            name: None,
            company_name: None,
            verification: Default::default(),
            limits: Default::default(),
            created_at: None,
        }))
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
        let response = self.client.get("/account/transactions", &query).await?;
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

        Ok(result.api_keys.or(result.data).unwrap_or_default())
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
        let request = CreateApiKeyRequest {
            name: name.into(),
            expires_at: None,
        };

        self.create_api_key_with_options(request).await
    }

    /// Creates a new API key with full options.
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
        let path = format!("/account/keys/{}", id.as_ref());
        let response = self.client.get(&path, &[]).await?;
        let result: ApiKeyResponse = response.json().await?;
        Ok(result.api_key.or(result.data).unwrap_or_default())
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
        let path = format!("/account/keys/{}/usage", id.as_ref());
        let response = self.client.get(&path, &[]).await?;
        let result: ApiKeyUsageResponse = response.json().await?;
        Ok(result.usage.or(result.data).unwrap_or_default())
    }

    /// Revokes an API key.
    ///
    /// # Arguments
    ///
    /// * `id` - API key ID
    pub async fn revoke_api_key(&self, id: impl AsRef<str>) -> Result<()> {
        let path = format!("/account/keys/{}", id.as_ref());
        self.client.delete(&path).await?;
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
    pub async fn rotate_api_key(
        &self,
        id: impl AsRef<str>,
    ) -> Result<RotateApiKeyResponse> {
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
            return Err(Error::Validation {
                message: "API key ID is required".to_string(),
            });
        }
        let path = format!("/account/keys/{}/rotate", id);
        let response = self.client.post(&path, &request).await?;
        let result: RotateApiKeyResponse = response.json().await?;
        Ok(result)
    }
}
