# Design: api-response

## Decisions

### Numeric `code` equals HTTP status on errors

Clients can read either the HTTP status line or `body.code` — both match.
Success always uses `code: 0` with HTTP 200.

### `err.kind` replaces legacy `error.code`

The stable snake_case identifier from `AppError::code()` lives at
`err.kind`. Human text stays in `message`.

### Base64 only on JSON image path

Protobuf `ImageData.image` carries raw bytes to avoid double encoding.

## File map

- `src/response.rs` — all envelope rendering
- `src/middleware/middleware.rs` — `trace_id` injection
