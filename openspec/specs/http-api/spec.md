# HTTP API

## Purpose

Define the public HTTP surface exposed by the image processing service.

## Requirements

### Requirement: Health endpoint

The service MUST expose `GET /health` that returns HTTP 200 with the unified
JSON success envelope. The `data` field MUST contain `status`, `cache`, and
`database` dependency health objects.

#### Scenario: liveness check

- GIVEN the service is running
- WHEN a client sends `GET /health`
- THEN the response status is 200
- AND `Content-Type` is `application/json`
- AND `data.status` is `"ok"` when cache and database ping succeed
- AND `data.cache` contains `{ "backend": "<name>", "ok": true|false }`
- AND `data.database` contains `{ "backend": "<name>", "ok": true|false }`
- AND `trace_id` is non-empty

### Requirement: Image processing endpoint (GET)

The service MUST expose `GET /img` that accepts transform parameters as query
strings and returns the unified JSON success envelope defined in `api-response`.
The processed image bytes MUST appear base64-encoded in `data.image` with
`data.content_type` set to the output MIME type.

#### Scenario: required source

- GIVEN a request to `/img` with no `src` query parameter
- WHEN the handler validates the request
- THEN the response status is 400 and `err.kind` is `bad_request`

#### Scenario: zero dimension rejected

- GIVEN a request with `w=0` or `h=0`
- WHEN the handler validates the request
- THEN the response status is 400 and `err.kind` is `bad_request`

#### Scenario: successful JSON image response

- GIVEN a successful transformation with `format=jpeg`
- WHEN the client parses the JSON body
- THEN `code` is 0, `data.content_type` is `image/jpeg`, and `data.image`
  decodes to a valid JPEG byte stream

### Requirement: Image processing endpoint (POST)

The service MUST expose `POST /img` that accepts an `ImageRequest` protobuf
body (`Content-Type: application/x-protobuf`) and returns an `ApiResponse`
protobuf envelope with the same semantic fields as the JSON envelope.

#### Scenario: protobuf success

- GIVEN a valid `ImageRequest` body
- WHEN the handler completes successfully
- THEN the response status is 200, `Content-Type` is
  `application/x-protobuf`, and the decoded `ApiResponse` has `code = 0` with
  populated `data.image` and `data.content_type`

#### Scenario: invalid protobuf body

- GIVEN a POST body that is not a valid `ImageRequest`
- WHEN the handler validates the request
- THEN the response status is 400 and the protobuf `ApiResponse.err.kind` is
  `bad_request`

### Requirement: Route registration

HTTP routes MUST be registered in `src/router.rs`. Handlers MUST live under
`src/controller/`. Access logging middleware MUST wrap all routes.

### Requirement: Processing pipeline

Image handlers MUST delegate transformation to `controller::img::process_image`
after parsing parameters (see `image-pipeline` and `params` specs).

#### Scenario: router wiring

- GIVEN the application starts
- WHEN `router::router(state)` is built
- THEN `/health`, `GET /img`, and `POST /img` are registered and the logging
  middleware is applied as an outer layer
