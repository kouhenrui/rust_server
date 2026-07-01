# Change: shared-infra

## Why

Reduce duplication across config loaders and standardize HTTP fetching,
response macros, and handler ergonomics.

## What Changes

- `src/util/` — `parse_or_warn`, `redact_url`
- `src/http_client.rs` — `HttpClient::fetch`
- `logger/macros.rs` — `span!`, `ok!`, `err!`
- `error.rs` — `AppResultExt`, `AppResultMapExt`
- `middleware` — `TraceId` `FromRequestParts`
- `state.check_health` — dependency ping for `/health`

## Capabilities

- `shared-infra`

## Non-goals

- Generic DI container
- Third-party config library

## Affected domains

- **shared-infra** (created)
- **observability** (span macro, TraceId)
- **api-response** (ok/err macros, health payload)
- **runtime-config** (parse_or_warn usage)
