# Change: http-api

## Why

The service needs a public HTTP surface for image transformation and health
checks. Clients need predictable JSON and Protobuf envelopes with stable error
kinds and trace correlation.

## What Changes

- `GET /health` returns unified JSON `{ code: 0, data: { status: "ok" } }`.
- `GET /img` accepts query parameters and returns JSON with base64 image data.
- `POST /img` accepts `ImageRequest` protobuf and returns `ApiResponse`.
- Errors use `{ code, message, err: { kind }, trace_id }`.
- Routes live in `router.rs`; handlers in `controller/`.

## Capabilities

- `http-api` — routes, handlers, middleware wiring
- `api-response` — shared envelope (see separate change)
- `proto-api` — POST wire format (see separate change)
- `observability` — trace_id and access logs (see separate change)

## Impact

- **Code:** `src/router.rs`, `src/controller/`, `src/response.rs`,
  `src/middleware/`, `src/error.rs`, `src/main.rs`
- **Dependencies:** axum, tower, tower-http, serde, serde_json, prost, nanoid
- **Breaking:** legacy raw-image GET responses and `{"error":{...}}` envelope
  are replaced by the unified format

## Non-goals

- Path versioning, authentication, signed URLs
- HTTP edge caching of JSON responses
- Streaming encoded image bodies

## Affected domains

- **http-api** (modified)
- **api-response** (created)
- **proto-api** (created)
- **observability** (created)
