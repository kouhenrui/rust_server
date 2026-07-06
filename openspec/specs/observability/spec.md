# Observability

## Purpose

Define structured logging, HTTP access logs, and distributed trace identifiers.

## Requirements

### Requirement: Global tracing subscriber

The service MUST initialize a global `tracing-subscriber` registry at startup
via `logger::init()`, using `LoggerConfig::from_env()` for filter directives
and a local-time formatted output layer.

#### Scenario: logger init at startup

- GIVEN the binary starts
- WHEN `main` runs
- THEN `logger::init()` is called before the axum server accepts connections

### Requirement: Log level configuration

Log filtering MUST be read from `RUST_LOG` when set; otherwise from
`THUMBOR_LOG_LEVEL` (applied as `{level},thumbor={level}`).

#### Scenario: RUST_LOG takes precedence

- GIVEN `RUST_LOG=debug,thumbor=trace` is set
- WHEN `LoggerConfig::from_env()` runs
- THEN the filter directive equals `RUST_LOG` verbatim

### Requirement: Encapsulated log macros

Application code MUST use the crate-root macros `trace!`, `debug!`, `info!`,
`warn!`, and `error!` from `src/logger/macros.rs` instead of calling
`tracing::*` directly. `info!`, `warn!`, and `error!` MUST attach
`module`, `file`, and `line` fields.

#### Scenario: structured info log

- GIVEN application code calls `crate::info!(key = %value, "message")`
- WHEN the log line is emitted
- THEN it includes `module`, `file`, `line`, and the provided fields

### Requirement: HTTP access middleware

All routes MUST pass through `middleware::logging_middleware`, which:

1. Resolves or generates `trace_id` (header `X-Trace-Id`)
2. Logs request method, path, query, and a parameter summary (not the full
   response body)
3. Logs completion with HTTP status and `latency_ms`
4. Injects `trace_id` into JSON and Protobuf response bodies and sets the
   `X-Trace-Id` response header

#### Scenario: request and completion logs

- GIVEN a `GET /health` request
- WHEN the middleware runs
- THEN an `http request received` log and an `http request completed` log are
  emitted with the same `trace_id`

#### Scenario: protobuf request summary

- GIVEN a `POST /img` with a valid `ImageRequest` body
- WHEN the middleware summarizes the request
- THEN the log includes `src`, dimensions, `fit`, `filters`, and `format`
  fields decoded from protobuf

### Requirement: No response-body logging

The access middleware MUST NOT log response bodies.

#### Scenario: completion log omits body

- GIVEN a successful `GET /img` response with a large base64 payload
- WHEN the access middleware emits `http request completed`
- THEN the log line does not include response body content

### Requirement: Supplemental macros

The service MUST provide crate-root `span!`, `ok!`, and `err!` macros for
spans and API responses (see `shared-infra` spec).

#### Scenario: ok macro builds envelope

- GIVEN handler code calls `ok!(data, trace_id)`
- WHEN the macro expands
- THEN it delegates to `response::api_success`

### Requirement: TraceId extractor

`middleware::TraceId` MUST implement axum `FromRequestParts` so handlers can
read the trace id injected by the logging middleware.

#### Scenario: handler reads trace id

- GIVEN a request passes through `logging_middleware`
- WHEN a handler extracts `TraceId` from request parts
- THEN the value matches `X-Trace-Id` on the response
