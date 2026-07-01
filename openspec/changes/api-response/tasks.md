# Tasks: api-response

## 1. Response types

- [x] 1.1 Define `ApiSuccess<T>`, `ApiErrorBody`, `ErrBody`, `ImageData`,
  `HealthData`, `ComponentHealth`, `ImageOutcome` in `src/response.rs`.
- [x] 1.2 Implement `api_success`, `api_error`, `ImageOutcome` with JSON and
  protobuf paths; `err.kind` from `AppError::code()`.
- [x] 1.3 Export `ok!` / `err!` macros delegating to api_success/api_error.

## 2. Trace injection

- [x] 2.1 Middleware injects `trace_id` into JSON objects and `ApiResponse`
  protobuf messages. (`src/middleware/middleware.rs`)
- [x] 2.2 Set `X-Trace-Id` response header on every response.

## 3. Verification

- [x] 3.1 Unit tests for success/error JSON shape in `src/response.rs`.
- [x] 3.2 Integration tests in `tests/integration.rs` assert envelope fields,
  `x-trace-id` header, health `data.cache`/`data.database`, and image base64.
