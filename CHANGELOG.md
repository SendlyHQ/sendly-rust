# sendly (Rust)

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
- `useCase` validation expanded from 23 entries to the full 43-value Telnyx enum.

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
