# Change: http-api

## Why

The service needs a public HTTP surface that clients (browsers, image CDNs,
internal callers) can hit to transform a source image. Without an HTTP
entrypoint there is no way to deliver the value of the image-processing
pipeline implemented elsewhere in this crate.

## What Changes

- Add a single image-processing endpoint at `GET /img` driven by query-string
  parameters (`src`, `w`, `h`, `fit`, `crop`, `filters`, `watermark`,
  `format`).
- Add a liveness endpoint at `GET /health` that returns `200 ok`.
- Return successful image bytes with `Content-Type` matching the chosen
  output format and a `Cache-Control: public, max-age=86400` header.
- Return all non-2xx responses as a single stable JSON envelope
  `{"error":{"code","message"}}` whose `code` is part of the public API
  contract.
- Reject requests that omit `src` or specify `w=0` / `h=0` with status 400.

## Capabilities

- `http-api`: Health endpoint, image endpoint contract, response caching,
  and the structured error envelope. (No other specs are touched by this
  change.)

## Impact

- **Code:** new `src/handler.rs` and `src/error.rs`. `src/main.rs` wires
  the router into the axum listener.
- **Dependencies:** `axum 0.7`, `tower 0.5`, `tower-http 0.6`
  (TraceLayer, CorsLayer), `serde 1`, `serde_json 1`, `thiserror 1`.
- **API:** introduces the public HTTP surface; nothing else depends on
  this yet so there are no breaking changes.
- **Config:** none — runtime configuration is owned by the `runtime-config`
  change.

## Non-goals

- Path versioning (e.g. `/v1/img`) — single endpoint, query-driven.
- Authentication, signed URLs, per-tenant rate limits.
- In-memory or HTTP-cache of the transformed image (the 24h
  `Cache-Control` is enough).
- Streaming response bodies for very large images — the full encoded
  image fits in one `Bytes` body.
- A binary/protobuf wire format — that lives in a separate change
  (`proto-api`).

## Affected domains

- **http-api** (created) — the only spec modified or created by this
  change.
