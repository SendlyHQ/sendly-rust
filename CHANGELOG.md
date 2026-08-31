# sendly (Rust)

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
