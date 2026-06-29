# API Response Envelope

## Purpose

Define the unified success and error response shapes shared by JSON and
Protobuf wire formats.

## Requirements

### Requirement: Success envelope

Successful responses MUST use HTTP 200 and a body of the form:

```json
{ "code": 0, "message": "success", "data": <object>, "trace_id": "<id>" }
```

The `data` field type depends on the endpoint (`HealthData` for `/health`,
`ImageData` for `/img`).

#### Scenario: JSON success shape

- GIVEN any successful JSON endpoint
- WHEN the client parses the body
- THEN `code` is 0, `message` is `"success"`, `data` is present, and `err`
  is absent

#### Scenario: image data encoding

- GIVEN a successful `GET /img` response
- WHEN the client reads `data.image`
- THEN the value is standard base64 encoding of the raw image bytes

### Requirement: Error envelope

Error responses MUST use an HTTP status matching the failure class and a JSON
body of the form:

```json
{ "code": <http_status>, "message": "<human readable>", "err": { "kind": "<stable_code>" }, "trace_id": "<id>" }
```

The numeric `code` MUST equal the HTTP status code. The stable string
identifier previously exposed as `error.code` is now `err.kind`.

#### Scenario: error body shape

- GIVEN any 4xx or 5xx JSON response
- WHEN the client parses the body
- THEN `code` equals the HTTP status, `message` is non-empty, and
  `err.kind` is a stable snake_case string from `AppError::code()`

#### Scenario: protobuf error parity

- GIVEN a failed `POST /img`
- WHEN the client decodes `ApiResponse`
- THEN `code` and `message` match the JSON semantics and `err.kind` carries
  the same stable identifier

### Requirement: Trace identifier

Every JSON and Protobuf response MUST include a non-empty `trace_id`. The
service MUST echo an incoming `X-Trace-Id` request header when provided;
otherwise it MUST generate one (nanoid).

#### Scenario: trace id in response

- GIVEN any HTTP response from the service
- WHEN the client inspects headers and body
- THEN `X-Trace-Id` is set and the parsed body contains the same `trace_id`

### Requirement: Implementation location

Envelope rendering MUST be centralized in `src/response.rs`. Controllers MUST
return `Response` via `api_success`, `api_error`, `ImageOutcome`, or the
protobuf equivalents — not ad-hoc JSON in handlers.
