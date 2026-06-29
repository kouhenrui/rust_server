# Design: http-api

## Context

This change defines the public HTTP surface. The implementation has evolved
from a single `handler.rs` to `router.rs` + `controller/`, with unified
response envelopes in `response.rs` and access logging in `middleware/`.

## Goals

- `/health` for liveness without touching the image pipeline.
- `GET /img` for cache-friendly query-string transforms (JSON envelope).
- `POST /img` for backend-to-backend callers (Protobuf envelope).
- Stable machine-readable error kinds via `err.kind`.
- Per-request `trace_id` for log correlation.

## Non-goals (recap)

No path versioning, no auth, no streaming bodies. See `proposal.md`.

## Decisions

### Router + controller split

`src/router.rs` registers routes and applies middleware.
`src/controller/health.rs` and `src/controller/img.rs` own handler logic.
This keeps routing wiring separate from business logic.

### Unified envelope instead of raw image bytes on GET

`GET /img` returns `{ code, message, data: { image: "<base64>", content_type } }`
so every endpoint shares one client parsing path. Raw image bytes appear only
inside the Protobuf `ImageData.image` field on `POST /img`.

**Trade-off:** browsers cannot use `<img src="/img?...">` directly without a
wrapper; API consistency is prioritized.

### Removed `Cache-Control: public, max-age=86400`

The previous design cached raw image responses at HTTP edges. With JSON
envelopes, edge caching is no longer applicable on the default GET path.

### `ImageOutcome` bridges pipeline and wire format

`controller/img.rs` runs `process_image` and converts the result to either
`into_json_response()` or `into_proto_response()` via `ImageOutcome` in
`response.rs`.

## File map

- `src/router.rs` — route registration, middleware layer
- `src/controller/health.rs` — `GET /health`
- `src/controller/img.rs` — `GET /img`, `POST /img`, processing pipeline
- `src/response.rs` — `ApiSuccess`, `ApiErrorBody`, `ImageOutcome`
- `src/error.rs` — `AppError`, status / kind mapping
- `src/main.rs` — binary entry, logger init, serve router

## Related specs

- `api-response` — envelope shape and `trace_id`
- `proto-api` — `POST /img` protobuf contract
- `observability` — access logging middleware
