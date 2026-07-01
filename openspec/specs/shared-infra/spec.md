# Shared Infrastructure

## Purpose

Cross-cutting helpers that are not domain-specific: env parsing utilities,
HTTP client wrapper, response/error ergonomics, and supplemental macros.

## Requirements

### Requirement: Environment parse helper

`util::parse_or_warn` MUST parse a string as `T: FromStr`, log `crate::warn!`
on failure, and return `None` without panicking.

#### Scenario: used in config loaders

- GIVEN invalid `THUMBOR_BIND`
- WHEN `Config::from_env()` runs
- THEN a warning is logged and the default bind address is kept

### Requirement: URL redaction

`util::redact_url` MUST mask credentials in connection URLs for logging.

#### Scenario: redis url

- GIVEN `redis://user:secret@127.0.0.1:6379/0`
- WHEN `redact_url` is called for a log line
- THEN the output does not contain `secret`

### Requirement: HTTP client wrapper

`http_client::HttpClient` MUST wrap `reqwest` with project timeout and expose
`fetch(url, max_bytes)` for remote image sources.

#### Scenario: size cap enforced

- GIVEN a response body larger than `max_bytes`
- WHEN `fetch` completes
- THEN the error is `AppError::SourceTooLarge`

### Requirement: Response macros

Crate-root `ok!` and `err!` MUST delegate to `response::api_success` and
`response::api_error`.

### Requirement: Span macro

Crate-root `span!` MUST create an `info_span` with `module`, `file`, and `line`.

### Requirement: TraceId extractor

`middleware::TraceId` MUST implement `FromRequestParts` so handlers can
extract the trace id set by `logging_middleware`.

### Requirement: AppState lifecycle

`AppState::connect` MUST wire `config`, `http`, `fonts` (`FontCache`),
`cache`, `db`, and `jwt` from runtime configuration.

#### Scenario: connect succeeds

- GIVEN valid cache and database env configuration
- WHEN `AppState::connect(config)` runs
- THEN all backends are connected and logged with `backend` name

### Requirement: Result extensions

`AppResultExt` and `AppResultMapExt` MUST provide `bad_request` /
`map_bad_request` helpers on `Result<T, E>`.
