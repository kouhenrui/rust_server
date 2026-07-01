# Tasks: proto-api

## 1. Schema and build

- [x] 1.1 Define `ImageRequest`, `ImageData`, `ErrInfo`, `ApiResponse` in
  `proto/api.proto`.
- [x] 1.2 `build.rs` runs `prost-build` with vendored `protoc` and
  `config.bytes(["."])` for `Bytes` fields. (`build.rs`)
- [x] 1.3 `src/proto.rs` includes generated code from `OUT_DIR`.

## 2. POST handler

- [x] 2.1 `img_post` decodes `ImageRequest`, converts via `img_request_to_params`.
  (`src/controller/img.rs`)
- [x] 2.2 Success/error rendered as `ApiResponse` protobuf via `ImageOutcome`.

## 3. Verification

- [x] 3.1 Integration tests: protobuf success, invalid body, error propagation.
  (`tests/integration.rs`)
- [x] 3.2 Middleware unit test summarizes proto request fields.
