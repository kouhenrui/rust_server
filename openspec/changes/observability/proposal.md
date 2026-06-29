# Change: observability

## Why

Operators need structured logs with request parameters, latency, and a
correlatable trace id across access logs and API responses.

## What Changes

- `logger/` module: config, formatter, init, encapsulated macros.
- `middleware/logging_middleware`: trace id, request summary, completion log.
- Application code uses `crate::info!` / `warn!` / `error!` instead of raw
  `tracing::*`.

## Capabilities

- `observability` — logging and HTTP access middleware.

## Impact

- **Code:** `src/logger/`, `src/middleware/`, call sites across `src/`
- **Config:** `RUST_LOG`, `THUMBOR_LOG_LEVEL`

## Non-goals

- OpenTelemetry export, log shipping agents, metrics/prometheus

## Affected domains

- **observability** (created)
- **http-api** (middleware on router)
