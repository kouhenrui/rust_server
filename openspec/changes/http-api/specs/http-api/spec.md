# HTTP API (delta)

## Purpose

Define the public HTTP surface exposed by the image processing service.

## MODIFIED Requirements

### Requirement: Health endpoint

The service MUST expose `GET /health` that returns HTTP 200 with the unified
JSON success envelope. The `data` field MUST contain `{ "status": "ok" }`.

#### Scenario: liveness check

- GIVEN the service is running
- WHEN a client sends `GET /health`
- THEN the response status is 200
- AND `Content-Type` is `application/json`
- AND the body matches `{ "code": 0, "message": "success", "data": { "status": "ok" }, "trace_id": "<non-empty>" }`

### Requirement: Image processing endpoint

The service MUST expose `GET /img` that accepts transform parameters as query
strings and returns the unified JSON success envelope. The processed image
bytes MUST appear base64-encoded in `data.image`.

#### Scenario: required source

- GIVEN a request to `/img` with no `src` query parameter
- WHEN the handler validates the request
- THEN the response status is 400 and `err.kind` is `bad_request`

#### Scenario: successful JSON image response

- GIVEN a successful transformation with `format=jpeg`
- WHEN the client parses the JSON body
- THEN `code` is 0, `data.content_type` is `image/jpeg`, and `data.image`
  decodes to a valid JPEG byte stream

#### Scenario: zero dimension rejected

- GIVEN a request with `w=0` or `h=0`
- WHEN the handler validates the request
- THEN the response status is 400 and `err.kind` is `bad_request`

## REMOVED Requirements

### Requirement: Response caching

**Reason:** Responses are now JSON/protobuf envelopes, not raw cacheable image
bytes. CDN caching of transformed images is a future concern.

### Requirement: Structured error envelope (legacy)

**Reason:** Replaced by `api-response` spec (`code` / `message` / `err.kind` /
`trace_id`).

## ADDED Requirements

### Requirement: Image processing endpoint (POST)

The service MUST expose `POST /img` with `Content-Type:
application/x-protobuf`. See `proto-api` spec.

### Requirement: Route registration

HTTP routes MUST be registered in `src/router.rs` with handlers in
`src/controller/` and logging middleware applied globally.
