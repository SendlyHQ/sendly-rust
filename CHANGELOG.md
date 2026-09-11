# sendly (Rust)

## 4.0.1

### Security

- **Path parameters are percent-encoded.** Every id you pass is now encoded (`urlencoding::encode`) before it goes into the request path. An id containing `/`, `?` or `#` used to change which endpoint the request reached: an id of `../../account/keys` left its collection and hit another endpoint carrying your API key. Ordinary ids are sent byte-for-byte as before.

No public type changed. The crate is on 4.0.1 because 4.0.0 was already published; the rest of the fleet is on 4.0.0.

## 4.0.0

The crate moves to 4.0.0 while the rest of the SDK fleet stays on 3.x. The
changes below are breaking by [cargo's semver rules](https://doc.rust-lang.org/cargo/reference/semver.html),
and shipping them inside a minor would break `cargo update` for anyone on
`^3.39`. The fleet version number is not worth that.

### Breaking Changes

- **`WebhookEvent::data` is now `Option<WebhookMessageData>`.** It is `Some` for
  `message.*` events and `None` for every event whose payload is not
  message-shaped.

  This is the point of the release rather than a side effect of it. Through
  3.39.0, `parse_event` decoded `data.object` straight into
  `WebhookMessageData`, whose `id`, `status`, `to` and `from` are all required,
  so a payload that is not a message could not survive the decode. In this crate
  it failed loudly: the whole `parse_event` call returned
  `Err(WebhookError::ParseError(..))` and your handler never saw the event at
  all. Measured against the payloads the API really sends:

  - `rcs_agent.live` → ``ParseError("missing field `id`")``
  - `contact.auto_flagged` → ``ParseError("missing field `status`")``
  - `call.completed` → ``ParseError("unknown variant `completed`, expected one of `queued`, `sent`, `delivered`, ...")``
  - `verification.verified` → ``ParseError("unknown variant `verified`, expected one of `queued`, `sent`, `delivered`, ...")``

  That covered every `rcs_*`, `whatsapp_*`, `call.*`, `brand.*`, `campaign.*`,
  `assignment.*`, `number.*`, `port*`, `contact*` and `verification.*` webhook.
  The dynamically typed SDKs in the fleet carried the same bug in a quieter
  form — they handed back a message object with every field at its default and
  raised nothing, and `contact.auto_flagged` reported the *contact* id as the
  message id, so a handler keyed on it acted on the wrong record. Rust at least
  refused to guess.

  After the upgrade those events parse. `data` is `None` for them, and reading a
  message field off one no longer compiles. The compile error is the migration
  prompt: it is what sends you to `object`, which has the real payload.

  ```rust
  // 3.39.0 — `data` was a plain WebhookMessageData
  let event = Webhooks::parse_event(body, sig, secret, Some(ts))?;
  // ^ for a lifecycle event this never reached the next line: it was
  //   Err(ParseError("missing field `id`")) and `?` bailed out of the handler
  let to = event.data.to;

  // 4.0.0 — the same call returns Ok, and the message view is optional
  let event = Webhooks::parse_event(body, sig, secret, Some(ts))?;
  let to = event.data.as_ref().map(|m| m.to.clone()); // None for lifecycle events
  let agent_id = event.object["agent_id"].as_str();   // the payload that did arrive
  ```

  **The edit:** every `event.data.<field>` becomes a `match` or an `if let` on
  the `Option`. A handler that only ever cared about `message.*` keeps its old
  behaviour with one line at the top —
  `let Some(message) = &event.data else { return Ok(()) };` — and a handler that
  needs the lifecycle payloads now has them, on `object`.

- **`WebhookEventType` gained the 20 event types the API actually emits** —
  `message.read`, `conversation.*`, `draft.*`, `rcs_brand.*`, `rcs_agent.*`,
  `whatsapp_account.*`, `whatsapp_template.*` and `call.*` — plus an
  `Unknown(String)` fallback so a type added later can never fail a parse again.
  It is now `#[non_exhaustive]`. Adding variants and adding `#[non_exhaustive]`
  are each a major change on their own.

  **The edit:** an exhaustive `match` on `WebhookEventType` stops compiling —
  ``error[E0004]: non-exhaustive patterns: `_` not covered``. Add a `_ => {}`
  arm. It is not busywork: that arm is where
  `Unknown("something.invented_later")` lands the day the API grows an event,
  instead of a `ParseError`.

- **`message.queued` and `message.undelivered` are removed.** The API has never
  emitted either one and rejects both with a 400 when you subscribe, so any code
  matching on them was dead. Other SDKs in the fleet keep them as deprecated for
  one more cycle; this crate drops them now because it is taking a major anyway.

  **The edit:** delete the `WebhookEventType::MessageQueued` and
  `WebhookEventType::MessageUndelivered` arms, and drop the two strings from any
  `webhooks().create(...)` / `update(...)` event list — the API answers 400 for
  them. Note that `WebhookMessageStatus::Queued` and
  `WebhookMessageStatus::Undelivered` are untouched: those are message
  *statuses*, which the API does send, and they are a different enum.

- **`WebhookEvent` is `#[non_exhaustive]`**, so later fields are additive.
  Construct one only by parsing a payload; a struct literal
  (`WebhookEvent { .. }`) no longer compiles outside this crate.

- **Minimum `serde` is now 1.0.185** (was `1.0`). `WebhookEventType` is
  `#[non_exhaustive]` with a data-carrying variant, and deriving `Serialize` on
  that shape fails to compile with *"cannot move out of a shared reference"* on
  serde 1.0.166 through 1.0.184. Verified by building against each boundary
  version. **The edit:** none, unless a lockfile pins you below 1.0.185, in
  which case `cargo update -p serde` resolves it.

### Added

- **`WebhookEvent::object`** carries `data.object` exactly as it arrived, for
  every event type — message events included — and **`object_as::<T>()`**
  deserializes it into a type of your choosing. This is the supported way to
  read a lifecycle payload.

  ```rust
  use sendly::webhooks::{WebhookEventType, WebhookVerificationData, Webhooks};

  #[derive(serde::Deserialize)]
  struct RcsAgentLive { agent_id: String, name: String, stage: String }

  let event = Webhooks::parse_event(body, sig, secret, Some(ts))?;
  match event.event_type {
      WebhookEventType::RcsAgentLive => {
          let agent: RcsAgentLive = event.object_as()?;
          println!("{} ({}) is {}", agent.name, agent.agent_id, agent.stage);
      }
      WebhookEventType::VerificationVerified => {
          // WebhookVerificationData was already declared in this crate, with
          // nothing that could produce one. `object_as` is that thing.
          let v: WebhookVerificationData = event.object_as()?;
          println!("{} verified after {} attempts", v.phone, v.attempts);
      }
      // No struct needed for a single field: `object` is a serde_json::Value.
      WebhookEventType::ContactAutoFlagged => {
          let contact_id = event.object["id"].as_str().unwrap_or_default();
          let message_id = event.object["message_id"].as_str(); // not the same id
          println!("contact {contact_id} flagged by {message_id:?}");
      }
      _ => {}
  }
  ```

- Webhook handling is now documented in the README, under **Webhooks →
  Receiving events**.

## 3.39.0

### Minor Changes

- **`RcsAgent` is now `#[non_exhaustive]`.** It is a deserialize-only response type, so this costs nothing to read it, and it means later fields (such as the `stage` added in this release) arrive as minor releases instead of breaking struct-literal construction. If you were building an `RcsAgent` by hand, construct it from a deserialized response instead.


- **RCS registration is self-serve from the SDK.** `client.rcs()` gains four sub-resources that mirror the dashboard's registration flow: `registration().get()` (the workspace's brand, agent, devices and `stage` at a glance), `dossier().get()` (business details already on file, shaped as an `RcsBrandInput` you can pass straight to `brands().create`), `brands().create` / `brands().update`, and on `agents()`: `create`, `get`, `update`, `set_test_devices`, `submit` and `request_launch`. Sendly reviews a submission first, then the carrier network; poll `agents().get` (or `registration().get`) and read `RcsCustomerStage` as it moves through review, testing and launch. Logo, hero and call-to-action media must already be public `https://` URLs; uploading assets is dashboard-only. Reads need the `rcs:read` scope and writes `rcs:write`. While RCS registration isn't enabled for an account these calls answer 404 (`Error::NotFound`).

  ```rust
  use sendly::{CreateRcsAgentRequest, RcsAgentBasicsInput, RcsBrandAddressInput, RcsBrandInput, Sendly};

  let brand = client.rcs().brands().create(
      RcsBrandInput::new()
          .display_name("Acme Coffee")
          .legal_name("Acme Coffee LLC")
          .ein("12-3456789")
          .address(RcsBrandAddressInput::new().line1("100 Main St").city("Chicago").state("IL").postal_code("60601").country_code("US")),
  ).await?.brand;
  let agent = client.rcs().agents().create(
      CreateRcsAgentRequest::new(&brand.id)
          .display_name("Acme Coffee")
          .use_case("MULTI_USE")
          .basics(RcsAgentBasicsInput::new().logo_url("https://acme.example/rcs/logo.png")),
  ).await?.agent;
  let review = client.rcs().agents().submit(&agent.id).await?;
  println!("{}", review.stage); // in_review
  ```
- **Typed registration models.** `RcsCustomerStage` and `RcsReviewStatus` enums (with an `Unknown` fallback so a stage this version doesn't know never fails decoding), `RcsBrand`, `RcsBrandAddress`, `RcsBrandContact`, `RcsAgentDetail`, `RcsAgentBasics`, `RcsTestDevice`, `RcsRegistration`, `RcsDossier`, and the response envelopes `RcsBrandResponse`, `RcsAgentResponse`, `RcsAgentDetailResponse`, `RcsTestDeviceListResponse`, `RcsAgentReviewResponse`. Inputs are builders: `RcsBrandInput` (+ `RcsBrandAddressInput`, `RcsBrandContactInput`), `RcsAgentBasicsInput` (+ `RcsAgentPhoneContact`, `RcsAgentWebsiteContact`, `RcsAgentEmailContact`), `RcsCampaign` (+ `RcsInteraction`, `RcsConsentSettings`, `RcsOptInMethod`), `RcsTesting`, `CreateRcsAgentRequest`, `UpdateRcsAgentRequest` (with `clear_campaign()` / `clear_testing()` to send `null`), `RcsTestDeviceInput` and `RcsRequestLaunchRequest`.
- **`RcsAgent::stage`.** `agents().list()` now reads the `stage` the API reports on each agent, as `Option<RcsCustomerStage>` (`None` when a payload doesn't carry it).
- **Idempotency keys on registration writes.** Every registration write carries an `Idempotency-Key`, generated per call like other POSTs and, new for this crate, on the PATCH and PUT calls too (`brands().update`, `agents().update`, `agents().set_test_devices`). Each write has a `*_with_options` twin that takes `IdempotentRequestOptions` for your own key; pass one to `submit_with_options` so a retried submit returns the original result instead of notifying reviewers again.
- **`Error::Api::code` is populated from `{ error, message }` bodies.** When an error body carries both an `error` code and a human `message` and no separate `code` field, the `error` value is now surfaced as `code`, so a 409 `rcs_field_locked`, `rcs_brand_not_verified` or `rcs_launch_not_ready` can be matched on instead of parsed out of the message. Bodies with only an `error` string are unchanged (`code` stays `None`).

## 3.38.0

### Minor Changes

- **Every JSON write call now reaches the API.** `reqwest`'s `.json()` already sets `Content-Type`, and the client set it a second time by hand. `RequestBuilder::header` appends rather than replaces, so every POST, PUT and PATCH went out carrying the header twice, the edge joined the two values into `application/json, application/json`, and the origin's body parser did not match it. The handler then saw an empty body and answered `400 "Missing required fields"`. If you have been getting `Error::Validation` back from sends, batches, contact and campaign writes, template writes, verification submits or anything else that posts JSON, this was the cause, and it applied to every write in the crate. GET and DELETE were never affected. No code change needed on your side, but calls that used to fail will now really execute, so re-check any retry loop or fallback you built around them.
- **Templates are live for the first time.** `client.templates()` targeted `/verify/templates`, `/verify/templates/{id}` and `/verify/templates/{id}/publish`. The API does not serve those paths, so `list`, `get`, `create`, `update`, `delete` and `publish` could not succeed. All six now call `/templates*` and work. Two related fixes came with them: `publish` used to post a bare `null` body and now posts `{}`, and `delete` used to try to decode an empty `204 No Content` as JSON and now returns `DeleteTemplateResponse { success: true, message: None }`.
- **`Template` now matches what the API actually returns.** The model described a template that the service never sends: `body`, `type`, `locale`, `isDefault`, `isPublished`. The real payload is `text`, `variables` (objects with `key`, `type`, `fallback`), `is_preset`, `preset_slug`, `status`, `version`, `published_at`, `created_at`, `updated_at`, and those are now the primary fields. `is_custom()` is unchanged, and `is_published()` still answers correctly: it reads `status == "published"`, falling back to a legacy `isPublished` flag if that is all the payload carries.
- **Variable specs are on a new field.** Each variable's type and fallback now live on `Template::variable_specs` (`Vec<TemplateVariable>`). The old `Template::variables` is still there and still `Vec<String>`, holding just the variable names, so code that read it keeps compiling and keeps returning the same thing, now with a deprecation warning. Numeric fallbacks (`"fallback": 1`) decode to `Some("1")` instead of failing the whole response, and a payload that sends `variables` as plain strings still decodes.
- **Restored and deprecated on `Template`.** These were all removed while the model was corrected and have been put back so existing code compiles. Each emits a deprecation warning:
  - `body` (use `text`; carries the same value)
  - `template_type` (use the `is_preset` field or `is_custom()`)
  - `variables` (use `variable_specs` for type and fallback)
  - `is_published` field (use `status` or the `is_published()` method)
  - `is_preset()` method (use the `is_preset` field of the same name)
  - `locale` and `is_default` are the honest exceptions: the templates API does not return either one, so unless you are decoding some older payload that still carries them, `locale` is permanently `None` and `is_default` is permanently `false`. Use `is_preset` to tell a built-in template from your own.

  A recorded 3.37.1-shaped payload still decodes in full, so stored responses and test fixtures written against the old shape keep working: `body`, `type`, `locale`, `isDefault` and `isPublished` are all still read when they are present. Serializing a `Template` emits the current API shape, so the deprecated fields are not written back out.
- **Restored and deprecated on the template request builders.** `CreateTemplateRequest::body` and `UpdateTemplateRequest::body` are back as deprecated fields, and `UpdateTemplateRequest::body()` is back as a deprecated builder method that sets the same value `text()` does, so whichever you call last wins. One caveat worth knowing: the request is serialized from `text`, so assigning to the `body` field directly no longer changes what is sent. `CreateTemplateRequest::new()` and `UpdateTemplateRequest::text()` keep both in step for you.
- **Automatic idempotency keys on POST.** The client now generates a key per logical POST and sends it as `Idempotency-Key`, reusing the same key across its own retries of a timeout or connection failure. That means a send that timed out after the server had already accepted it is recognized as a retry instead of going out twice. The server records a key only once the first attempt has finished, so this narrows the duplicate-send window rather than closing it: a retry that fires while the original is still running is not seen as a repeat. This covers single sends, group MMS, WhatsApp, RCS, scheduled sends and the multipart upload paths (media, verification documents, business upgrade). GET and DELETE do not carry a key.
- **`IdempotentRequestOptions` for supplying your own key.** New `*_with_options` variants sit alongside the existing methods and take one: `send_with_options`, `send_whatsapp_with_options`, `send_rcs_with_options`, `send_group_with_options`, `schedule_with_options` and `send_batch_with_options`. Supply your own key when you need dedupe across process restarts or your own retry loop. Repeating a request with the same key inside 24 hours returns the original recorded response rather than executing again, and that includes recorded failures, so use a fresh key when you actually want to re-run. Reusing a key with a different payload comes back as `Error::Validation`. Keys are validated locally as 1 to 255 printable ASCII characters and rejected before any network call; an empty or whitespace-only key falls back to the automatic one.

  ```rust
  use sendly::{IdempotentRequestOptions, Sendly, SendMessageRequest};

  let message = client.messages()
      .send_with_options(
          SendMessageRequest::new("+15551234567", "Your order #4821 has shipped!"),
          IdempotentRequestOptions::new().idempotency_key("order-4821-shipped"),
      )
      .await?;
  ```
- **Batch sends deliberately carry no automatic key.** The batch endpoint dedupes header-less retries by hashing the content of the request, which also catches an identical re-run from a different process. An auto-generated key would defeat that, so `send_batch` only sends a key you supply yourself via `send_batch_with_options`.
- **`send_batch` can decode its response.** `BatchMessageResponse::queued` was a required field that the batch response has never carried, so the call failed to parse even once the request itself was accepted. It is now optional and reads `0`; use `total`, `sent` and `failed`.
- **Credit history and API key management were pointed at paths the server does not serve.** `account().transactions()` called `/account/transactions` and now calls `/credits/transactions`. `account().revoke_api_key()` sent `DELETE /account/keys/{id}`, which reaches no revoke handler, so the key was never actually revoked; it now sends `PATCH /account/keys/{id}/revoke` and the key really is revoked, so be sure the ids you pass are the ones you mean.
- **API keys decode instead of coming back blank.** `account().api_keys()` looked for `apiKeys` or `data` in the response envelope, but the API answers `{"keys": [...]}`, so it quietly returned an empty list however many keys you had. `account().get_api_key()` expected the key wrapped in `apiKey` or `data`, but the API returns the object unwrapped, so it quietly returned an all-empty `ApiKey` with `is_active: false`. Both now read the real shape, and both still accept the older envelopes.
- Not fixed, and worth knowing before you rely on them: the list endpoint ignores the `limit`, `type` and `locale` filters that `ListTemplatesOptions` sends, and returns every template you can see. `CreateTemplateRequest::locale`, `UpdateTemplateRequest::locale` and `published()` are also ignored by the service, so `published(true)` on a create does not publish anything; call `templates().publish(id)` instead. Neither request type can send variable types or fallbacks yet, so the service derives the variable list from the template text.
- New dependency: `uuid` 1.x with the `v4` feature, used to generate idempotency keys.

## 3.32.0

### Minor Changes

- New **`business_upgrade()`** resource on the client — the toll-free entity-upgrade ("fork-with-new-number") flow. When a customer forms a new legal entity (e.g. an LLC), this resource reserves a new toll-free number under the new entity, submits it for carrier review, and atomically swaps to it on approval — without disrupting outbound SMS during the 1-2 week review window. Mirrors the same resource on our Node, Python, Ruby, Go, and C# SDKs.

  ```rust
  use sendly::{Sendly, business_upgrade::{StartUpgradeRequest, BrnType, EntityType, EinDocument}};

  let client = Sendly::new("sk_live_v1_xxx");

  // 1) Preview validation (no writes)
  let report = client.business_upgrade().preflight(/* PreflightCandidate { ... } */).await?;

  // 2) Submit the upgrade with the IRS letter
  let pdf = std::fs::read("./CP-575.pdf")?;
  let result = client
      .business_upgrade()
      .start(
          "ws_abc",
          StartUpgradeRequest::new(
              "Acme Holdings LLC",
              "12-3456789",
              BrnType::Ein,
              "US",
              EntityType::PrivateProfit,
          ),
          Some(EinDocument::new(pdf).filename("CP-575.pdf")),
      )
      .await?;

  // 3) Poll status, cancel, resubmit, or set disposition once approved
  let status = client.business_upgrade().status("ws_abc").await?;
  ```

  Seven methods: `preflight`, `best_prefill`, `start`, `status`, `cancel`, `resubmit`, `set_disposition`. File upload uses `reqwest::multipart` via the SDK's existing `post_multipart` helper. New types (`PreflightCandidate`, `PreflightReport`, `StartUpgradeRequest`, `ResubmitUpgradeRequest`, `EinDocument`, `Disposition`, `SetDispositionRequest`, plus response structs) live in the `business_upgrade` module; `BusinessUpgradeResource` is re-exported from the crate root.

## 3.31.0

### Minor Changes

- New method **`conversations().suggest_replies(id)`** — returns AI-generated reply suggestions for a conversation based on its recent message history. Mirrors the same method on our Node, Python, Ruby, Go, and C# SDKs (closes a feature gap).

  ```rust
  let response = client.conversations().suggest_replies("conv_abc").await?;
  for s in response.suggestions {
      println!("{} ({})", s.text, s.tone);
  }
  ```

  New types `SuggestedReply` and `SuggestRepliesResponse` are re-exported from the crate root.

## 3.30.0

### Minor Changes

- `enterprise.workspaces().submit_verification(workspace_id, data)`: rewritten to match the actual API shape (camelCase top-level via `serde(rename_all = "camelCase")`, nested `address`/`contact` objects, `entity_type` + `brn`/`brn_type`/`brn_country` instead of the prior shape). The previous shape didn't match the server endpoint and was returning 400s.
- **Partial-update friendly:** for resubmits on existing workspaces, send only the fields you want to change — everything else is filled from the existing record. Hosted page URLs (`/biz/`, `/opt-in/`, `/legal/`) generated during provision are auto-preserved.
- `enterprise.workspaces().resubmit_verification(workspace_id, partial)`: convenience alias for resubmits — same as `submit_verification` but reads more naturally for one-field-change use cases.
- New `VerificationSubmitInput` struct — type-safe payload shape with all fields as `Option<...>` so `None` = omit. Implements `Default`, so partial updates are ergonomic via struct-update syntax. `SubmitVerificationRequest` is kept as a type alias for backwards compatibility.
- `VerificationAddress` and `VerificationContact` fields are now all `Option<...>` to support the partial-update model. Both implement `Default`.

### Server-side fixes paired with this release

- `/api/v1/enterprise/workspaces/:id/verification/submit` now returns specific missing-field errors (e.g. `"Missing required fields: website"`) instead of listing every required field whether present or not.
- Endpoint accepts both flat and `{ verification: {...} }` wrapped shapes (matches `/enterprise/provision`).
- `useCase` validation expanded from 23 entries to the full 43-value carrier use-case enum.

## 3.29.0

### Minor Changes

- `contacts.bulk_mark_valid(BulkMarkValidRequest::of_ids(...))` / `BulkMarkValidRequest::of_list_id(...)`: clear the invalid flag on many contacts at once (up to 10,000 per call). Escape hatch for when auto-mark misclassifies at scale.
- Four new list-health `WebhookEventType` variants: `ContactAutoFlagged`, `ContactMarkedValid`, `ContactsLookupCompleted`, `ContactsBulkMarkedValid`.
- New `ListHealthEventSource` enum (frozen): `SendFailure | CarrierLookup | UserAction | BulkMarkValid` — the `source` field on auto-flag and mark-valid webhooks.
- `Contact` gains `user_marked_valid_at` — when a user manually cleared an auto-flag. Carrier re-checks respect this timestamp and leave the contact clean.
- `CheckNumbersResponse` gains `already_running` so the client knows when a rapid re-trigger was collapsed against an in-flight lookup.

## 3.28.0

### Minor Changes

- `contacts.mark_valid(id)`: clear the auto-exclusion flag on a contact.
- `contacts.check_numbers(CheckNumbersRequest { list_id, force })`: trigger a background carrier lookup.
- `Contact` gains `opted_out`, `line_type`, `carrier_name`, `line_type_checked_at`, `invalid_reason`, `invalidated_at` (with snake_case and camelCase deserialize aliases).

## 3.18.1

### Patch Changes

- fix: webhook signature verification and payload parsing now match server implementation
  - `verify_signature()` accepts `timestamp: Option<&str>` for HMAC on `timestamp.payload` format
  - `parse_event()` handles `data.object` nesting (with flat `data` fallback for backwards compat)
  - `WebhookEvent` adds `livemode: bool`, `created: Value` fields
  - `WebhookMessageData` renamed `message_id` to `id` (with `message_id()` method alias)
  - Added `direction`, `organization_id`, `text`, `message_format`, `media_urls` fields
  - `generate_signature()` accepts `timestamp: Option<&str>` parameter
  - Added `MessageReceived`, `MessageOptOut`, `MessageOptIn` event types
  - 5-minute timestamp tolerance check prevents replay attacks

## 3.18.0

### Minor Changes

- Add MMS support for US/CA domestic messaging

## 3.17.0

### Minor Changes

- Add structured error classification and automatic message retry
- New `error_code` field with 13 structured codes (E001-E013, E099)
- New `retry_count` field tracks retry attempts
- New `Retrying` status variant and `message.retrying` webhook event

## 3.16.0

### Minor Changes

- Add `transfer_credits()` for moving credits between workspaces

## 3.15.2

### Patch Changes

- Fix flaky network error test, add metadata to batch items

## 3.13.0

### Minor Changes

- Campaigns, Contacts & Contact Lists resources with full CRUD
- Template clone method
