# Image Source Loading

## Purpose
Define how the service obtains the bytes of the source image referenced by the
`src` query parameter.

## Requirements

### Requirement: HTTP(S) sources
The service MUST accept `http://` and `https://` URLs as the `src` value when
remote sources are enabled.

#### Scenario: fetch remote URL
- GIVEN remote sources are enabled
- AND `src=https://example.com/cat.jpg`
- WHEN the handler loads the source
- THEN the bytes are downloaded and used as the source image

### Requirement: Local sources via `file://`
The service MUST accept `file://` URIs as the `src` value.

#### Scenario: load local file
- GIVEN `src=file:///tmp/cat.jpg` and the file exists
- WHEN the handler loads the source
- THEN the file's bytes are used

### Requirement: Relative local sources
The service MUST resolve a non-URL `src` value:
- by prepending `THUMBOR_LOCAL_SOURCE_ROOT` when set, OR
- as a literal filesystem path otherwise.

#### Scenario: resolve relative path
- GIVEN `THUMBOR_LOCAL_SOURCE_ROOT=/var/img` and `src=cat.jpg`
- WHEN the handler loads the source
- THEN the bytes are read from `/var/img/cat.jpg`

### Requirement: Remote sources can be disabled
The service MUST reject `http(s)://` sources when remote loading is disabled.

#### Scenario: rejection
- GIVEN `THUMBOR_ALLOW_REMOTE=false` and `src=https://...`
- WHEN the handler loads the source
- THEN the response status is 502 and `err.kind` is `remote_disabled`

### Requirement: Source size cap
The service MUST reject any source whose body exceeds the configured maximum
(`THUMBOR_MAX_SOURCE_BYTES`).

#### Scenario: oversized source
- GIVEN a source larger than the configured cap
- WHEN the handler loads the source
- THEN the response status is 413 and `err.kind` is `source_too_large`

### Requirement: Fetch timeout
Remote fetches MUST be bounded by `THUMBOR_FETCH_TIMEOUT_MS`. Timeouts surface
as a 502 with error code `upstream_failed`.

#### Scenario: timeout
- GIVEN a remote server that does not respond within the timeout
- WHEN the handler loads the source
- THEN the request fails with status 502 and `err.kind` is `upstream_failed`
