# Tasks: image-pipeline

## 1. Orchestration

- [x] 1.1 Implement `process_image` in `src/controller/img.rs`.
- [x] 1.2 Order: load → decode → transform → filters → watermark → encode.
- [x] 1.3 Export pipeline via `controller::img::process_image` for tests.

## 2. Encoding

- [x] 2.1 `encode` supports PNG, JPEG (quality 85), WebP lossless.
- [x] 2.2 `OutputFormat::content_type()` drives response MIME type.

## 3. Shared state helpers

- [x] 3.1 `AppState::sniff_format` in `src/state.rs` for magic-byte detection.
- [x] 3.2 `FontCache` lazy TTF loading on `AppState.fonts`.

## 4. ImageOutcome wiring

- [x] 4.1 `ImageOutcome::from_result` maps `process_image` output to JSON or
  protobuf envelope with base64 `data.image`.

## 5. Verification

- [x] 5.1 Integration tests: GET resize (`w=4&h=4`), POST protobuf success,
  format override (`format=jpeg`). (`tests/integration.rs`)
