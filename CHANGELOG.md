# sendly (Rust)

## 6.0.0

The crate ships this with fleet release 4.3.0. It takes a major because several fixes change a public type or enum; each one below says what to edit. Every other change is additive. Each fix was checked against the API handler it calls.

### Major Changes

- **Errors keep the API's code and body.** `Error::Authentication`, `RateLimit`, `InsufficientCredits`, `Validation` and `NotFound` gain `code: Option<String>` and `body: Option<serde_json::Value>`, and `Error::Api` gains `body`. Read them from any variant with `Error::code()`, `Error::body()` and `Error::remaining_attempts()`. Before this, a wrong `verify().check()` code lost its `remaining_attempts` (the API reports it only on the 400 `invalid_code` error, never on a success), and the three 429s `max_attempts_exceeded`, `rate_limit_exceeded` and `too_many_failed_key_attempts` could not be told apart. `Error` is now `#[non_exhaustive]`.

  **The edit:** a pattern that names every field, such as `Error::Validation { message }` or `Error::RateLimit { message, retry_after }`, stops compiling; add `..` (`Error::Validation { message, .. }`). An exhaustive `match` on `Error` needs a `_` arm. Code that builds an `Error` by hand adds `code: None, body: None` (`body: None` for `Error::Api`).

- **Rate limits carry their code, and only a transient refusal is retried.** `Error::is_retryable()` is now false for a 429 whose code says waiting cannot clear it: `too_many_failed_key_attempts` (the key is wrong, and the API refuses keys from that address for up to 300 seconds), `max_attempts_exceeded` (the verification has failed), `quota_exceeded` (the workspace's monthly message quota is used up, from `messages().schedule()`, `send_batch()` and `campaigns().send()`, until the month resets or an admin raises it), `daily_call_limit` (`calls().create()`, until tomorrow), `hold_limit_reached` and `whatsapp_signup_limit_reached`. `rate_limit_exceeded` and `provision_rate_limit` stay retryable. `Error::RateLimit::retry_after` falls back to the `retryAfter` in the body when there is no `Retry-After` header, so the one-time-password limits, which answer with up to 600 or 86,400 seconds and no header, now say how long to wait. The client itself retries one 429: `too_many_concurrent_verifications`, after its `Retry-After` (1 second), within `max_retries` and with the same idempotency key, because that request never ran. File uploads retry it too (listed under Patch Changes). It never waits more than 60 seconds for a 429; every other 429 is returned at once, as before.

- **`SenderType` has the values the API sends.** Every live send that the API did not simulate returned `Err` (``unknown variant `number_pool` ``) after the message was sent and charged, because the enum only knew `user`, `api`, `system` and `campaign`, which the API never sends. That covered `messages().send()`, `send_with_options()`, `send_to()` and `conversations().reply()`. `SenderType` now has `NumberPool`, `Alphanumeric`, `Sandbox`, `Explicit` and an `Unknown` fallback, and is `#[non_exhaustive]`; the four old variants stay, deprecated.

  **The edit:** match the new variants, and give an exhaustive `match` a `_` arm.

- **`TransactionType` keeps every ledger type.** `account().transactions()` failed to decode whenever the page held a `transfer`, `admin_grant` or `admin_seed` row. The enum gains `Transfer`, `AdminGrant`, `AdminSeed` and an `Unknown` fallback, and is `#[non_exhaustive]`. `Adjustment` is deprecated: the API never records it (auto-recharges are recorded as `Purchase`). `ListTransactionsOptions::transaction_type` filters by the new types, and filtering by `Unknown` sends no filter. `TransactionType` now implements `Display`.

  **The edit:** give an exhaustive `match` a `_` arm.

- **`ScheduledMessageStatus` has `Delivered`, `Bounced` and an `Unknown` fallback**, and is `#[non_exhaustive]`. A scheduled message takes its delivery receipt, so `messages().list_scheduled()` and `get_scheduled()` failed to decode once any was delivered or bounced. `ScheduledMessage::is_sent()` is also true for a delivered message. Filtering by `Unknown` sends no filter.

  **The edit:** give an exhaustive `match` a `_` arm.

- **`CampaignStatus` has `Completed`**, the status a campaign ends in once it has sent, and is `#[non_exhaustive]`. `Sent` and `Paused` are deprecated: the API never gives a campaign either status. It reads a `Sent` filter on `campaigns().list()` as `Completed`, and a `Paused` filter returned nothing. `CampaignStatus` now implements `Display`.

  **The edit:** filter by `CampaignStatus::Completed` where you filtered by `Sent`, and give an exhaustive `match` a `_` arm.

- **The WhatsApp enums have an `Unknown` fallback** and are `#[non_exhaustive]`: `WhatsAppSignupStatus`, `WhatsAppSenderStatus`, `WhatsAppTemplateCategory`, `WhatsAppTemplateStatus`, `WhatsAppMessageKind` and `WhatsAppMessageCategory`. The API passes template statuses and categories through from WhatsApp as they come, so one value this SDK did not know (such as `IN_APPEAL`) failed `whatsapp().templates().list()` for the whole workspace, and an unknown kind or category failed `messages().send_whatsapp()` after the message was sent and charged. It now decodes as `Unknown` and the rest of the response is kept.

  **The edit:** give an exhaustive `match` on any of them a `_` arm.

- **`conversations().add_labels()` returns `Vec<Label>` and `remove_label()` returns `()`.** Both returned `Err` on every call after the change was applied: the API answers `add_labels` with the conversation's labels (`{ data: [...] }`) and `remove_label` with an empty 204, and the SDK tried to read a `Conversation` from both. `add_labels` now returns every label the conversation has.

  **The edit:** drop code that read a `Conversation` from these calls; call `conversations().get()` if you need it.

- **`campaigns().send()` returns the batch it created**, a `BatchMessageResponse`. It failed on every call: its body was the JSON literal `null`, which the API refused with a 400 before the route ran, and the batch result it answers with would not have decoded as a `Campaign`. The new `send_from(id, from)` sends from a number of yours (400 `invalid_from_number` if the workspace cannot send from it).

  **The edit:** read `batch_id` from the result and follow it with `messages().get_batch()`; call `campaigns().get()` for the campaign.

- **`enterprise().workspaces().list()` returns an `EnterpriseWorkspaceList`.** It failed on every call: the API returns an object (`workspaces`, `pagination`, `summary`, `max_workspaces`, `workspaces_used`), not a bare list, and its items carry `creditBalance`, not `credits`. Items are `EnterpriseWorkspaceListItem` (status, verification status and type, toll-free number, `credit_balance`, key count, 30-day message counts, quota and tags). The new `list_with_options(ListWorkspacesOptions)` pages, searches, filters by status, verification and tags, sorts, and asks for the compact list of every workspace; `ListWorkspacesOptions::limit()` keeps the page size between 1 and 100, and a limit of 0 is never sent, because the API cannot page by 0 and answers with a `totalPages` of `null`. `get()` still returns an `EnterpriseWorkspaceDetail`.

  **The edit:** iterate `list.workspaces`, and read `credit_balance` where you read `credits`.

- **`OwnedNumber::monthly_cost_cents` is `Option<i64>`.** A number with no recorded price, such as the toll-free number that comes with a verification, has a `null` cost, so `numbers().list()` failed for any workspace that had one, and `get()` and `update()` failed for that number.

  **The edit:** `number.monthly_cost_cents.unwrap_or(0)` where you need a number.

- **Rule conditions and actions are objects: `RuleConditions` and `RuleActions`.** The API stores and evaluates a rule's conditions as one object (`intent`, `sentiment`, `intentConfidenceMin`, `sentimentConfidenceMin`) and its actions as one object (`addLabels`, `closeConversation`). The SDK sent lists, which the API stored but never evaluated, so such a rule matched every message and applied nothing; and `rules().list()` failed to decode any rule created elsewhere. `RuleMatch` holds one value or a list (`intent` can be either), and converts from a string or a `Vec` of strings or string literals (`.intent(vec!["complaint", "refund"])`). A rule stored as a list by an older SDK reads back as its first entry. `Rule` gains `enabled`, and `UpdateRuleRequest` gains `enabled` to switch a rule off. `rules().create()` no longer rejects empty conditions or actions, which the API accepts.

  **The edit:** `CreateRuleRequest { name, conditions: RuleConditions::new().intent("complaint"), actions: RuleActions::new().add_labels(vec!["lbl_1"]), priority: None }`.

- **`CreateApiKeyRequest` can set the key's type and scopes.** It gains `key_type` (`"test"` or `"live"`) and `scopes`, derives `Default`, and has builders: `CreateApiKeyRequest::new("Production").key_type("live").scopes(vec!["sms:send"])`. Without a type the API creates a test key, so a live key could not be created from the SDK. `expires_at` is now sent as `expiresAt`, which the API also reads.

  **The edit:** build the request with `CreateApiKeyRequest::new(name)` and the builders instead of a struct literal.

- **`StartUpgradeResponse` decodes the 202 `business_upgrade().start()` returns.** It failed on every call because the response has no verification id, toll-free number or messaging profile id yet: the new number is provisioned after the call returns. `toll_free_number` and the two provider id fields are now `Option<String>` and stay `None` (the id fields and `ein_doc_stored` are deprecated); the response gains `status` (`provisioning`).

  **The edit:** follow the upgrade with `business_upgrade().status()` for the new number.

- **`messages().send_group()` and `send_group_with_options()` decode a live send.** Every live group send returned `Err` (`invalid type: map, expected a string`) after the message had been sent and charged, because the API lists a live send's recipients as objects (`{ phoneNumber, status }`) and the SDK read `to` as strings; only a simulated send lists plain numbers. `GroupMessageResponse::to` still holds the phone numbers either way, and the new `recipients` lists each `GroupRecipient` (`phone_number`, `status`) on a live send and is empty on a simulated one.

  **The edit:** code that builds a `GroupMessageResponse` with a struct literal adds `recipients`.

- **Response types gained fields** (listed under Minor Changes). Code that builds `Message`, `MessageList`, `BatchMessageResponse`, `BatchPreviewResponse`, `Credits`, `Account`, `AccountLimits`, `ApiKey`, `ApiKeyUsage`, `CreateApiKeyResponse`, `Rule`, `UpdateRuleRequest`, `WebhookTestResult`, `EnterpriseWebhook`, `EnterpriseWebhookTestResult`, `WorkspaceWebhookTestResult`, `CreditsAnalytics`, `ProvisionWorkspaceResponse`, `InheritVerificationResponse`, `WhatsAppSender`, `WhatsAppSignup` or `WhatsAppSignupSession` with a struct literal must add them.

### Minor Changes

- **`Message::simulated`, `simulated_reason` and `action_url`.** A send that the API simulated (a test key, a sandbox number, or a live key whose account is not yet set up for the destination) says so, and a live key's simulated send says why and where to finish setting up. `is_sandbox` is read from `get()` and `list()`; the send response does not report it.
- **`MessageList::pagination`** (`MessagePagination`: `total`, `limit`, `offset`, `page`, `total_pages`, `has_more`), and `MessageList::total()` returns the number of messages matching the query across all pages instead of the page size. `count` is the size of the page.
- **`BatchMessageResponse` gains `delivered`, `credits_reserved` and `credits_refunded`.**
- **`BatchPreviewResponse` gains `duplicates`, `key_type`, `has_write_scope`, `pooled`, `blocked_messages` (`BatchPreviewBlockedMessage`) and `warnings`.** `messages` and `block_reasons` are deprecated: the API sends neither.
- **`Credits::billing_mode`** (`"prepaid"` or `"pooled"`).
- **`Account::organization`** (`AccountOrganization`: the workspace's `id`, `name` and `is_personal`), **`Account::business_verification`** (`AccountBusinessVerification`: `status`, `verification_type`, `region`, `submitted_at`, `updated_at`) and **`AccountLimits::messages_per_minute`**. `Account::verification`, `name` and `company_name`, the `AccountVerification` type, and `AccountLimits::messages_per_second` and `max_batch_size` are deprecated: the API sends none of them.
- **`ApiKey::key_type` and `ApiKey::scopes`**, on every key the account endpoints return, and `CreateApiKeyResponse` gains the flat `id`, `name`, `key_prefix` and `key_type`.
- **`ApiKeyUsage::recent_requests`** (`ApiKeyRequestRecord`) and **`endpoint_breakdown`** (`ApiKeyEndpointCount`). The usage figures cover the key's last 100 requests. `successful_requests` and `failed_requests` are deprecated: the API does not report them. `ApiKeyUsage`, `ApiKeyRequestRecord` and `ApiKeyEndpointCount` are exported from the crate root, so they can be named in your own code.
- **`RedeliverOptions` and `BackfillOptions` are exported**, so `webhooks().redeliver()` and `backfill()` can be given a window, event types, statuses and a limit instead of only `Default::default()`.
- **`WebhookTestResult::message` and `delivery`**, and `webhooks().test()` documents that an endpoint that refuses the test event comes back as `Error::Validation`.
- **`CreditsAnalytics` gains `total_balance`, `total_lifetime`, `total_used` and `workspace_count`.** `data` and `CreditDataPoint` are deprecated: the endpoint reports totals, not a daily series.
- **`EnterpriseWebhook` gains `events`, `workspaces`, `signing_secret` and `rotated_at`**, and `EnterpriseWebhookTestResult` gains `status_text` and `error`. `WorkspaceWebhookTestResult` gains `delivery`.
- **`ProvisionWorkspaceResponse` gains `opt_in_page` and `business_page` (`ProvisionedPage`), `legal_pages` (`ProvisionedLegalPages`), `api_base_url` and `dashboard_url`.**
- **WhatsApp: add a number by code.** `whatsapp().signup().create_with_options(CreateWhatsAppSignupRequest)` takes a `business_account_id` (the WhatsApp Business Account id from `senders().list()`), a `verification_method` (`WhatsAppVerificationMethod::Sms`, the default, or `Voice`) and a `display_name`, and adds the number to that already-connected account with no Facebook step. It charges the same $19 one-time fee (refunded automatically if the connection fails) and returns a session with status `Verifying` and no `connect_url`. `signup().verify(id, code)` submits the 6-digit code and `signup().resend(id, method)` asks for another (at least 30 seconds apart). Without a `business_account_id`, `create_with_options` is `create`: it starts the Facebook connection, with the same retries and automatic idempotency key. A `business_account_id` that is present but empty or whitespace-only is `Error::Validation` before anything is sent, instead of starting a paid Facebook signup. `verify()` and `create_with_options()` with a non-empty `business_account_id` are sent once and never retried, not after a 5xx, a timeout or a network error: every submitted code uses up one of five attempts, and a failed start has already failed the session and refunded the fee, so a retry would be a new $19 session. `resend()` keeps the usual retries. `WhatsAppSignupStatus` gains `Verifying`; `WhatsAppSignup` gains `verification_method`, `verification_attempts_remaining` and `verification_code` (the code once its text has arrived on the number, set by `get()`), and `WhatsAppSignupSession` gains `phone_number`, `business_account_id`, `verification_method`, `verification_attempts_remaining` and `updated_at`; `connect_url` is empty for a number added by code. New failure reasons: `verification_start_failed`, `verification_failed` and `verification_expired`. New refusals: 404 `whatsapp_business_account_not_found`, 400 `display_name_required` and `invalid_verification_code`, 409 `whatsapp_signup_in_progress`, `whatsapp_verification_in_progress`, `whatsapp_verification_failed`, `whatsapp_verification_busy` and `signup_not_active`, 422 `whatsapp_verification_code_invalid` (`Error::remaining_attempts()` reads its `attemptsRemaining`), 422 or 502 `whatsapp_verification_start_failed` and `whatsapp_verification_resend_failed`, 429 `whatsapp_verification_resend_too_soon` (`Error::RateLimit`, retryable, with `retry_after`), and 502 `whatsapp_verification_unavailable` and `whatsapp_activation_pending`.
- **WhatsApp sender extras.** `whatsapp().senders()` gains `upload_profile_photo(phone, path)` and `upload_profile_photo_bytes(phone, data, filename, content_type)` (a JPEG or PNG of at most 5 MB, sent as the multipart field `file`, once and never retried after a 5xx, a timeout or a network error), `delete_profile_photo(phone)`, `get_conversational_components(phone)` and `update_conversational_components(phone, UpdateWhatsAppConversationalComponentsRequest)` for ice breakers and `WhatsAppCommand`s (each list given replaces the stored one; an empty list clears it), and `set_calling(phone, enabled)`, which returns a `WhatsAppSenderCalling`. `WhatsAppSender` gains `business_account_id`, `business_name`, `calling_enabled` and `outbound_calling_allowed` (false for every +1 number and for +20, +84 and +234). New refusals: 400 `file_required` and `whatsapp_profile_photo_invalid`, 413 `whatsapp_profile_photo_too_large`, 502 `whatsapp_profile_update_failed`, `whatsapp_conversational_components_fetch_failed`, `whatsapp_conversational_components_update_failed` and `whatsapp_calling_update_failed`, 409 `voice_not_enabled` and 422 `whatsapp_calling_unavailable`.
- **`Call::channel`** (`CallChannel`: `Phone`, `WhatsApp`, `Browser`, with an `Unknown` fallback) says how a call travelled. The `call.started`, `call.completed` and `call.recording.ready` webhooks carry `channel` too, as a field of `data.object`: read it from `event.object["channel"]`, or decode `data.object` with `object_as()` into a struct with a `channel: CallChannel` field. Calls to WhatsApp users are placed from the dashboard only; there is no API for them.
- **`enterprise().workspaces().inherit_verification_with_options(id, InheritVerificationOptions)`.** `InheritVerificationOptions::new(source).purchase_new_number(true)` copies only the business details and orders the workspace its own toll-free number instead of sharing the source's; `InheritVerificationResponse::new_number` reports it.

### Patch Changes

- **Eighteen calls that failed on every call now reach their route.** They posted the JSON literal `null`, which the API's JSON parser refuses with a 400 before any route runs; they now post `{}`: `conversations().close()`, `reopen()` and `mark_read()`, `drafts().approve()`, `campaigns().send()`, `cancel()` and `clone()`, `templates().clone()`, `verify().resend()`, `webhooks().test()`, `reset_circuit()`, `rotate_secret()` and `retry_delivery()`, `enterprise().workspaces().test_webhook()` and `resume()`, and `enterprise().webhooks().test()` and `rotate_secret()`. `templates().unpublish()`, deprecated because the API has no such route, now fails with the 404 its note promises instead of a 400.
- **`drafts().approve()` sends the draft.** With the API update released alongside, approving sends the message in the same call and returns the approved draft with the sent message's `message_id`; a send refused with a 4xx puts the draft back to pending.
- **`messages().get_batch()` and `list_batches()` failed on every call**, because they read the batch id from `batchId` and those responses call it `id`. `batch_id` now reads either.
- **`messages().preview_batch()` failed on every call**: it required a `can_send` field the API does not send, and read most figures from keys the API does not use. It now reads `total`, `sendable`, `creditBalance` and `hasSufficientCredits` into `total_messages`, `will_send`, `current_balance` and `has_enough_credits`, and `can_send` is true when nothing the preview found stops a send: the key can send, the batch has at most 10,000 messages (a send of more is rejected with 400 `batch_too_large`), at least one message is sendable, nothing is blocked for a reason other than an opt-out (a live send rejects the whole batch then) and, for a live key, the balance covers it. A test key's send skips the destination and verification checks, so it can go through while `can_send` is false.
- **`webhooks().list()` failed whenever the account had a webhook**, because the API returns a bare list.
- **`account().get()` returned an `Account` with an empty `id` and `email`**, because the API nests them under `user`; it now reads them, and returns an error when the response has no account at all.
- **`account().credits()` read `reserved_credits` as 0**; it now reads the API's `reservedBalance`. `pending_credits` is deprecated: the API has no pending balance.
- **`account().get_api_key_usage()` returned zeros**; it reads the figures from the response's `summary`.
- **Campaigns.** `campaigns().schedule()` failed on every call because it sent `scheduled_at` and the API reads only `scheduledAt`. `get()`, `list()`, `create()`, `update()`, `schedule()`, `cancel()` and `clone()` failed on every call: the response carries each timestamp under both `created_at` and `createdAt` (and `updated_at`), which the decoder treated as a duplicate field. `recipient_count` and `started_at` now read the API's `totalRecipients` and `sentAt`, and `contact_list_ids` falls back to the campaign's `targetListId`.
- **`contacts().check_numbers()` read `already_running` as false**; it now reads the API's `alreadyRunning`.
- **`webhooks().test()` reported `status_code` and `response_time_ms` as 0**, and `enterprise().workspaces().test_webhook()` reported no `status_code`; both are read from the test delivery.
- **`enterprise().webhooks().set()` and `rotate_secret()` dropped the signing secret**, which the API shows only once; it is `EnterpriseWebhook::signing_secret`.
- **`enterprise().analytics().credits()` returned an empty series and no totals**, and **`enterprise().provision()` dropped the generated pages and URLs**; both now read what the API sends.
- **`conversations().reply()` accepts a media-only reply.** It required `text` although the API accepts `text` or `media_urls`; an empty `text` is left out of the request.
- **Ids are percent-encoded in every webhook, API key and enterprise path**, as 4.0.1 said they were. An id containing `/` could still send a request, carrying your API key, to a different endpoint from `webhooks()`, `account().get_api_key()`, `get_api_key_usage()` and `revoke_api_key()`, and every `enterprise().workspaces()` call; the `webhook_id` filter of `delete_webhooks()` is encoded too.
- **An id that is empty, `.` or `..` is refused before any request is sent.** Every method that puts an id in a path returns `Error::Validation` for one. Percent-encoding cannot protect such an id, because the URL parser reads it (and `%2E%2E`) as a dot-segment: `enterprise().workspaces().revoke_key("ws_1", "..")`, `delete_opt_in_page()` and `cancel_invitation()` sent `DELETE /enterprise/workspaces/ws_1/`, the route that deletes the workspace, and `contacts().lists().remove_contact("list_1", "..")` sent the request that deletes the list. An id with dots inside it, such as `whk..1`, is sent as before.
- **File uploads retry the busy key-check refusal.** `media().upload()`, `upload_bytes()`, `business_upgrade().start()`, `resubmit()` and `enterprise().upload_verification_document()` sent once, so the 429 `too_many_concurrent_verifications` came straight back. They now wait out its `Retry-After` and send the same file again with the same idempotency key, within `max_retries`, like every other request. An upload is still not sent again after a timeout or a network error.
- **WhatsApp send outcomes.** `messages().send_whatsapp()` documents the 409 `whatsapp_send_unconfirmed` (`Error::Api`): the outcome is unknown, the message was marked failed and refunded but may still be delivered, so check before sending it again. It is cached under the idempotency key and the client does not retry it. A 502 `whatsapp_send_failed` was not sent and is safe to send again.
- **Message text over 1600 characters is counted in characters.** `messages().send()`, `schedule()`, `send_batch()` and `preview_batch()` refused text written outside ASCII as over 1600 characters when it was under that, because the check counted UTF-8 bytes: 801 accented letters, 534 CJK characters or 401 emoji were refused before anything was sent. It counts characters now.
- **List counts say what they count.** `MessageList::count`, `ScheduledMessageList::count` and `BatchList::count` are documented as the page size, and `ScheduledMessageList::total()` and `BatchList::total()` are deprecated (the API reports no total for them). `WebhookDeliveryList` and `CreditTransactionList` `total` and `has_more` are deprecated: the API sends neither.
- **Docs.** `webhooks().backfill()` said synthesized events have fresh ids and to dedupe on `data.object.id`; they carry the same event id the original dispatch used, so dedupe on `event.id` (a message's sent and delivered events share `data.object.id`). The `CallsResource` error list gains 400 `from_number_not_supported`, a call from a number outside the US and Canada.

## 5.1.0

### Minor Changes

- **Voice configuration: `client.voice()`.** Everything a call depends on can now be set up from code instead of only in the dashboard. Three sub-resources:
  - `voice().numbers()`: `list` (`GET /voice/numbers`), `get` (`GET /voice/numbers/{number}`), `update` (`PATCH /voice/numbers/{number}`: switch voice on or off, choose `VoiceMode::RingDashboard` or `VoiceMode::Agent`, point the number at an agent) and `register_emergency_address` (`POST /voice/numbers/{number}/emergency-address`). `{number}` is the number's id or its E.164 phone number; the SDK percent-encodes it, so `+15555550188` goes out as `%2B15555550188`. A US or Canadian number needs an emergency address before it can place calls; the first registration adds $1.50 a month to the number.
  - `voice().agents()`: `list`, `create` (201), `get`, `update` and `delete` on `/voice/agents`. Each agent gets its own scoped sending key so it can text callers (`can_send_sms`), and a workspace can have up to 20 (409 `agent_limit`). An agent that still answers a number can't be deleted (409 `agent_in_use`): point those numbers at another agent or back to the team first.
  - `voice().voices()`: `list` (`GET /voice/voices`), the voices an agent can speak with.

  ```rust
  use sendly::{
      CreateVoiceAgentRequest, RegisterEmergencyAddressRequest, UpdateVoiceNumberRequest, VoiceMode,
  };

  let agent = client
      .voice()
      .agents()
      .create(CreateVoiceAgentRequest::new("Front desk").greeting("Thanks for calling Acme, how can I help?"))
      .await?;
  client
      .voice()
      .numbers()
      .register_emergency_address(
          "+15555550188",
          RegisterEmergencyAddressRequest::new("500 Example Ave", "Austin", "TX", "78701"),
      )
      .await?;
  client
      .voice()
      .numbers()
      .update(
          "+15555550188",
          UpdateVoiceNumberRequest::new().voice_enabled(true).voice_mode(VoiceMode::Agent).agent_id(&agent.id),
      )
      .await?;
  ```
- **Typed voice models.** `VoiceNumber`, `VoiceNumberEmergencyAddress` (with `is_active()`), `EmergencyAddress`, `VoiceNumberRates`, `VoiceNumberListResponse`, `VoiceAgent`, `VoiceAgentTools`, `VoiceAgentListResponse`, `DeletedVoiceAgent`, `Voice`, `VoiceListResponse` and the `VoiceMode` enum, which has an `Unknown` fallback so a mode this version doesn't know never fails decoding. The response types and `VoiceMode` are `#[non_exhaustive]`. Requests are builders: `UpdateVoiceNumberRequest`, `RegisterEmergencyAddressRequest`, `CreateVoiceAgentRequest`, `UpdateVoiceAgentRequest` and `VoiceAgentToolsInput`. Unset fields are left out of the request, and `clear_agent_id()` / `clear_transfer_to()` send `null`.
- **Idempotency keys on every voice write.** `update`, `register_emergency_address`, `create` and `delete` carry an auto-generated `Idempotency-Key`; each has a `*_with_options` variant that takes `IdempotentRequestOptions` for your own key.
- Reads need the `calls:read` scope, writes `calls:write` and a live API key. Refusals map as they do for calls: 404s (`voice_not_enabled`, `number_not_found`, `agent_not_found`) are `Error::NotFound`; 400s (`invalid_request`, `invalid_voice_mode`, `agent_required`, `invalid_address`, `e911_not_applicable`) and 422 `invalid_address` (the address couldn't be validated) are `Error::Validation`; everything else (403 `forbidden` / `live_key_required`, 409 `agent_disabled` / `agent_limit` / `agent_in_use`, 502 `voice_attach_failed` / `carrier_refused`, 503 `voice_unavailable`) is `Error::Api` with `code` set.

### Patch Changes

- `WebhookEvent::data` is now `None` for every event that is not a `message.*` event. Call events whose `from` and `to` are phone numbers, such as `call.completed`, used to come back with a fabricated message view (`segments` 1, `credits_used` 0); read them through `WebhookEvent::object` or `object_as` instead.

- **Recording channels.** The `CallRecording` docs had the channels the wrong way round. Agent calls are recorded with the agent on the left channel and the other party on the right.
- The `CallsResource` and `OwnedNumber::voice_enabled` docs no longer send you to the dashboard to configure voice; they point at `client.voice()`.

## 5.0.0

### Major Changes

- **Open enums.** `MessageStatus`, `WebhookMessageStatus` and `OwnedNumber` are now `#[non_exhaustive]`, and both status enums gain an `Unknown` variant that absorbs any value this build does not know. Before this, `MessageStatus` had no `Received`, so `messages().list()` failed to deserialise for any workspace with inbound traffic (the API lists inbound rows by default), and `WebhookMessageStatus` had no `Read`, so every RCS and WhatsApp `message.read` webhook lost its typed message view. Adding variants to a closed enum is a breaking change under Cargo's semver rules, which is why this is a major: exhaustive `match` statements on these enums need a wildcard arm, and `OwnedNumber` can no longer be constructed or destructured exhaustively outside the crate. No method signatures changed.
- `MessageStatus::Received` and `WebhookMessageStatus::Read` are new variants.

### Minor Changes

- **Voice calls: `client.calls()`.** Place a phone call that one of your workspace's AI agents handles, follow it, end it early, and fetch the recording. Five methods: `create` (`POST /calls`), `list` (`GET /calls`, filters and paging), `get` (`GET /calls/{id}`, with the `transcript` on agent-handled calls), `hangup` (`POST /calls/{id}/hangup`; a ringing call becomes `cancelled`, an active one `completed`, an ended one is returned unchanged) and `recording` (`GET /calls/{id}/recording`; the URL is signed and valid for five minutes). Calls go to US and Canadian numbers from a number that is voice-enabled in the dashboard and has an emergency address registered; they are billed per started minute from your credit balance (10 credits/min for an agent-handled outbound call; unanswered calls cost nothing). Reads need the `calls:read` scope, writes `calls:write` and a live API key. Voice is being enabled workspace by workspace; until it is on for yours the routes answer 404 `voice_not_enabled` (`Error::NotFound`).

  ```rust
  use sendly::{CreateCallRequest, Sendly};

  let call = client
      .calls()
      .create(
          CreateCallRequest::new("+15555550123", "3c4d5e6f-7081-4293-a4b5-c6d7e8f90a1b")
              .from_number("+15555550188")
              .context("You are calling Jordan to confirm the 3pm appointment on Tuesday.")
              .metadata_entry("crmId", "lead_8812"),
      )
      .await?;
  let call = client.calls().get(&call.id).await?;
  for line in call.transcript.unwrap_or_default() {
      println!("{}: {}", line.speaker, line.text);
  }
  ```
- **Typed call models.** `Call` (with `is_live()` / `is_ended()`), `CallTranscriptLine`, `CallListResponse` + `CallPagination` (`total`, `limit`, `offset`, `has_more`), `CallRecording`, the request builders `CreateCallRequest` and `ListCallsOptions`, and the enums `CallStatus`, `CallDirection`, `CallKind`, `CallHandledBy`, `CallBilling`, `CallRecordingStatus` and `CallTranscriptSpeaker`, each with an `Unknown` fallback so a value this version doesn't know never fails decoding. Because `from` is a Rust keyword, the wire field `from` is `from_number` on `Call`, `CreateCallRequest` and `ListCallsOptions`. The response types are `#[non_exhaustive]`, so fields added later arrive as minor releases.
- **Idempotency keys on call writes.** `create` and `hangup` carry an auto-generated `Idempotency-Key` like every other POST; `create_with_options` and `hangup_with_options` take `IdempotentRequestOptions` for your own key, so a retried request never places a second call.
- **`OwnedNumber::voice_enabled` and `OwnedNumber::voice_mode`.** `numbers().list()` and `numbers().get()` now read the voice fields the API reports on each number (`voice_mode` is `"none"`, `"ring_dashboard"` or `"agent"`), so you can find a number to call from. `OwnedNumber` is now `#[non_exhaustive]`; it is a deserialize-only response type, so this costs nothing to read it and means later fields arrive as minor releases instead of breaking struct-literal construction.
- **Call webhooks.** The `call.started`, `call.completed` and `call.recording.ready` objects now carry `billing` and `metadata` (the pairs you attached on create). The crate's `WebhookEvent::object` already exposes the raw object, so both read as `event.object["billing"]` and `event.object["metadata"]`; no type changed.

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
