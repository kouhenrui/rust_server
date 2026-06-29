# Protobuf API

## Purpose

Define the binary wire format for `POST /img` and the shared `ApiResponse`
message used on that path.

## Requirements

### Requirement: Schema location

API messages MUST be defined in `proto/api.proto` (`package thumbor.v1`) and
compiled at build time via `build.rs` + `prost-build`. Rust types are exposed
through `src/proto.rs`.

#### Scenario: proto compiles on build

- GIVEN a clean checkout
- WHEN `cargo build` runs
- THEN `api::ImageRequest` and `api::ApiResponse` are available from
  `thumbor::proto::api`

### Requirement: ImageRequest fields

`ImageRequest` MUST carry the same semantic parameters as `GET /img` query
strings: `src`, optional `w`/`h`, `fit` enum, `crop` rect, `filters` string,
`watermark` oneof (text or image), and `format` enum.

#### Scenario: enum defaults match GET defaults

- GIVEN `fit` and `format` are unspecified (0)
- WHEN `controller::img::img_request_to_params` converts the request
- THEN fit defaults to `cover` and format defaults to `png`

### Requirement: ApiResponse envelope

`ApiResponse` MUST mirror the JSON envelope:

| field | success | error |
|---|---|---|
| `code` | `0` | HTTP status as `int32` |
| `message` | `"success"` | human-readable error |
| `data` | `ImageData` populated | absent |
| `err` | absent | `ErrInfo { kind }` |
| `trace_id` | non-empty string | non-empty string |

`ImageData` carries raw `bytes image` (not base64) and `content_type`.

#### Scenario: protobuf image bytes are raw

- GIVEN a successful `POST /img`
- WHEN the client reads `data.image`
- THEN the bytes are the encoded image file directly (not base64)

### Requirement: Content-Type

`POST /img` requests MUST use `Content-Type: application/x-protobuf`.
Responses MUST use the same content type.

#### Scenario: wrong content type on GET path

- GIVEN a client sends `GET /img`
- WHEN the handler responds
- THEN `Content-Type` is `application/json` regardless of output image format
