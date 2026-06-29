# Change: api-response

## Why

Clients need one predictable success/error shape across `/health` and `/img`,
with stable error kinds and trace correlation.

## What Changes

- Introduce `ApiSuccess`, `ApiErrorBody`, `ImageOutcome` in `src/response.rs`.
- JSON: base64 `data.image` on success; `err.kind` on failure.
- Middleware injects `trace_id` into JSON and Protobuf bodies.

## Capabilities

- `api-response` — envelope contract and rendering helpers.

## Impact

- **Code:** `src/response.rs`, `src/middleware/middleware.rs`
- **Tests:** `src/response.rs` unit tests, `tests/integration.rs`

## Non-goals

- gRPC, GraphQL, or versioning headers
- Per-endpoint custom error shapes

## Affected domains

- **api-response** (created)
- **http-api** (consumes envelope)
- **proto-api** (protobuf mirror)
