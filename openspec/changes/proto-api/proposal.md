# Change: proto-api

## Why

Backend callers need a compact binary API for `/img` with the same semantics
as the GET query-string path.

## What Changes

- `proto/api.proto` with `ImageRequest` and `ApiResponse`.
- `build.rs` compiles proto via `prost-build` + vendored `protoc`.
- `POST /img` handler in `src/controller/img.rs`.

## Capabilities

- `proto-api` — protobuf schema and POST handler.

## Impact

- **Code:** `proto/api.proto`, `build.rs`, `src/proto.rs`, `src/controller/img.rs`
- **Dependencies:** `prost`, `prost-build`, `protoc-bin-vendored`

## Non-goals

- gRPC services, proto-over-HTTP for `/health`
- JSON transcoding of protobuf messages

## Affected domains

- **proto-api** (created)
- **http-api** (POST route)
- **api-response** (shared envelope fields)
