# sendly (Rust)

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
