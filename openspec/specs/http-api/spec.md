# HTTP API

## Purpose
Define the public HTTP surface exposed by the image processing service.

## Requirements

### Requirement: Health endpoint
The service MUST expose `GET /health` that returns HTTP 200 with the body `ok`.

#### Scenario: liveness check
- GIVEN the service is running
- WHEN a client sends `GET /health`
- THEN the response status is 200 and the body is exactly `ok`

### Requirement: Image processing endpoint
The service MUST expose `GET /img` that takes a source image and a transform
specification as query parameters, and returns the resulting image bytes.

#### Scenario: required source
- GIVEN a request to `/img` with no `src` query parameter
- WHEN the handler validates the request
- THEN the response status is 400 and the error code is `bad_request`

#### Scenario: content-type matches format
- GIVEN a successful transformation with `format=jpeg`
- WHEN the response is returned
- THEN the `Content-Type` header is `image/jpeg` and the body is a valid JPEG byte stream

#### Scenario: zero dimension rejected
- GIVEN a request with `w=0` or `h=0`
- WHEN the handler validates the request
- THEN the response status is 400

### Requirement: Response caching
Successful image responses MUST be cacheable at HTTP edges for 86400 seconds.

#### Scenario: cache header
- GIVEN any successful `/img` response
- WHEN the client inspects headers
- THEN `Cache-Control: public, max-age=86400` is present

### Requirement: Structured error envelope
All non-2xx responses MUST be JSON of the shape
`{"error":{"code":"<stable_code>","message":"<human readable>"}}`.

#### Scenario: error body shape
- GIVEN any 4xx or 5xx response
- WHEN the client parses the body
- THEN it contains the `error.code` and `error.message` fields
