//! Account resource for managing account information and credits.

use crate::client::Sendly;
use crate::error::Result;
use crate::models::{
    Account, ApiKey, CreateApiKeyRequest, CreateApiKeyResponse, CreditTransactionList, Credits,
    ListTransactionsOptions,
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
        let response = self.client.get("/account/api-keys", &[]).await?;
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
        let response = self.client.post("/account/api-keys", &request).await?;
        let result: CreateApiKeyResponse = response.json().await?;
        Ok(result)
    }

    /// Revokes an API key.
    ///
    /// # Arguments
    ///
    /// * `id` - API key ID
    pub async fn revoke_api_key(&self, id: impl AsRef<str>) -> Result<()> {
        let path = format!("/account/api-keys/{}", id.as_ref());
        self.client.delete(&path).await?;
        Ok(())
    }
}
