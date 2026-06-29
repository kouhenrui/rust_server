# Tasks: api-response

## 1. Response types

- [x] 1.1 Define `ApiSuccess<T>`, `ApiErrorBody`, `ErrBody`, `ImageData`,
  `HealthData` in `src/response.rs`.
- [x] 1.2 Implement `api_success`, `api_error`, `ImageOutcome` with JSON and
  protobuf paths.

## 2. Trace injection

- [x] 2.1 Middleware injects `trace_id` into JSON objects and `ApiResponse`
  protobuf messages. (`src/middleware/middleware.rs`)
- [x] 2.2 Set `X-Trace-Id` response header on every response.

## 3. Verification

- [x] 3.1 Unit tests for success/error JSON shape in `src/response.rs`.
- [x] 3.2 Integration tests in `tests/integration.rs` assert envelope fields.
