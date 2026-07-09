use crate::client::Sendly;
use crate::error::{Error, Result};
use crate::models::{
    CreateShortLinkRequest, CreateShortLinkResponse, ListShortLinksOptions, ShortLinkListResponse,
    UpdateShortLinkRequest, UpdateShortLinkResponse,
};

/// Links resource for branded URL shortening.
///
/// Mint branded short links for a destination URL, list the links your
/// workspace has created (with click analytics), and enable or disable an
/// individual link (a per-link kill switch).
///
/// URL shortening is gated behind the `url_shortener` rollout flag (currently
/// founder-only); while the flag is off for your account these calls resolve as
/// [`Error::NotFound`](crate::Error::NotFound) — the same as an absent feature.
#[derive(Debug, Clone)]
pub struct LinksResource<'a> {
    client: &'a Sendly,
}

impl<'a> LinksResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Mints a branded short link for an http/https destination URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The destination URL to shorten (must be http/https)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let link = client.links().create("https://example.com/spring-sale").await?;
    /// println!("{}", link.short_url);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, url: impl Into<String>) -> Result<CreateShortLinkResponse> {
        let url = url.into();
        validate_url(&url)?;

        let body = CreateShortLinkRequest { url };
        let response = self.client.post("/links", &body).await?;
        let result: CreateShortLinkResponse = response.json().await?;

        Ok(result)
    }

    /// Lists the short links your workspace has created, newest first, with
    /// click counts and a 14-day daily click histogram.
    ///
    /// # Arguments
    ///
    /// * `options` - Optional pagination options (`limit` 1-200, `offset`)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, ListShortLinksOptions};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let listing = client.links().list(Some(ListShortLinksOptions::new().limit(50))).await?;
    /// for link in &listing.links {
    ///     println!("{} -> {} ({} clicks)", link.short_url, link.destination_url, link.click_count);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(
        &self,
        options: Option<ListShortLinksOptions>,
    ) -> Result<ShortLinkListResponse> {
        let query = options.map(|o| o.to_query_params()).unwrap_or_default();

        let response = self.client.get("/links", &query).await?;
        let result: ShortLinkListResponse = response.json().await?;

        Ok(result)
    }

    /// Enables or disables a short link (a per-link kill switch). A disabled
    /// link's redirect returns 404 until it is re-enabled.
    ///
    /// # Arguments
    ///
    /// * `code` - The short link code
    /// * `disabled` - `true` to disable, `false` to re-enable
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// client.links().set_disabled("Ab3xY7", true).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_disabled(
        &self,
        code: &str,
        disabled: bool,
    ) -> Result<UpdateShortLinkResponse> {
        if code.is_empty() {
            return Err(Error::Validation {
                message: "Link code is required".to_string(),
            });
        }

        let encoded = urlencoding::encode(code);
        let path = format!("/links/{}", encoded);
        let body = UpdateShortLinkRequest { disabled };
        let response = self.client.patch(&path, &body).await?;
        let result: UpdateShortLinkResponse = response.json().await?;

        Ok(result)
    }

    /// Disables a short link (its redirect returns 404 until re-enabled).
    /// Convenience wrapper over [`LinksResource::set_disabled`].
    pub async fn disable(&self, code: &str) -> Result<UpdateShortLinkResponse> {
        self.set_disabled(code, true).await
    }

    /// Re-enables a previously disabled short link. Convenience wrapper over
    /// [`LinksResource::set_disabled`].
    pub async fn enable(&self, code: &str) -> Result<UpdateShortLinkResponse> {
        self.set_disabled(code, false).await
    }
}

fn validate_url(url: &str) -> Result<()> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(Error::Validation {
            message: "url must be an http:// or https:// URL".to_string(),
        });
    }
    Ok(())
}
